//! Farming domain — soil tilling, watering, planting, crop growth, harvest.
//!
//! Communicates with other domains exclusively through crate::shared events/resources.

use bevy::prelude::*;
use crate::shared::*;

mod soil;
mod crops;
mod harvest;
mod render;
mod events_handler;
mod sprinkler;

// Internal re-exports used by Bevy queries from outside the module (not currently needed).

/// Marker component for soil tile entities managed by the farming domain.
#[derive(Component, Debug, Clone)]
pub struct SoilTileEntity {
    pub grid_x: i32,
    pub grid_y: i32,
}

/// Marker component for crop sprite entities managed by the farming domain.
#[derive(Component, Debug, Clone)]
pub struct CropTileEntity {
    pub grid_x: i32,
    pub grid_y: i32,
}

/// Tracks which soil/crop entities exist keyed by grid position.
/// This lets systems find ECS entities for a given tile quickly.
#[derive(Resource, Default, Debug)]
pub struct FarmEntities {
    /// (x, y) → soil entity
    pub soil_entities: std::collections::HashMap<(i32, i32), Entity>,
    /// (x, y) → crop entity
    pub crop_entities: std::collections::HashMap<(i32, i32), Entity>,
}

/// Holds the texture atlas handles for farming sprites (soil tiles and plant stages).
/// Loaded once on entering Playing state; render systems use the handles once loaded.
#[derive(Resource, Default)]
pub struct FarmingAtlases {
    pub loaded: bool,
    pub plants_image: Handle<Image>,
    pub plants_layout: Handle<TextureAtlasLayout>,
    pub dirt_image: Handle<Image>,
    pub dirt_layout: Handle<TextureAtlasLayout>,
}

/// A pending harvest interaction from the player pressing Space.
#[derive(Event, Debug, Clone)]
pub struct HarvestAttemptEvent {
    pub grid_x: i32,
    pub grid_y: i32,
}

/// A pending planting interaction (player uses seed on tilled soil).
#[derive(Event, Debug, Clone)]
pub struct PlantSeedEvent {
    pub grid_x: i32,
    pub grid_y: i32,
    pub seed_item_id: ItemId,
}

/// Signal that the morning sprinkler phase should run.
/// Fired by the day-start path before normal gameplay systems run.
#[derive(Event, Debug, Clone)]
pub struct MorningSprinklerEvent;

pub struct FarmingPlugin;

impl Plugin for FarmingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Internal resources
            .init_resource::<FarmEntities>()
            .init_resource::<FarmingAtlases>()
            // Internal events
            .add_event::<HarvestAttemptEvent>()
            .add_event::<PlantSeedEvent>()
            .add_event::<MorningSprinklerEvent>()
            // ------------------------------------------------------------------
            // Atlas loading — runs once on first Playing frame
            // ------------------------------------------------------------------
            .add_systems(
                OnEnter(GameState::Playing),
                load_farming_atlases,
            )
            // ------------------------------------------------------------------
            // Systems that run during Playing
            // ------------------------------------------------------------------
            .add_systems(
                Update,
                (
                    // Tool use responses — soil interaction
                    soil::handle_hoe_tool_use,
                    soil::handle_watering_can_tool_use,
                    // Planting (player presses interact with seed in hand over tilled tile)
                    crops::handle_plant_seed,
                    // Harvest (player presses Space near mature crop)
                    harvest::handle_harvest_attempt,
                    // Keyboard shortcut: Space bar → try harvest at player position
                    harvest::detect_harvest_input,
                    // Seed placement detection (player uses seed item)
                    crops::detect_seed_use,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // ------------------------------------------------------------------
            // DayEnd processing — crop growth & overnight logic
            // ------------------------------------------------------------------
            .add_systems(
                Update,
                (
                    sprinkler::apply_sprinklers,
                    events_handler::on_day_end,
                    events_handler::on_season_change,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // ------------------------------------------------------------------
            // Visual sync — runs after all state mutations
            // ------------------------------------------------------------------
            .add_systems(
                PostUpdate,
                (
                    render::sync_soil_sprites,
                    render::sync_crop_sprites,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Atlas loading system
// ─────────────────────────────────────────────────────────────────────────────

/// Loads the farming texture atlases once when the Playing state is entered.
/// After this runs, `FarmingAtlases::loaded` is true and render systems can use
/// `plants_image`/`plants_layout` for crop sprites and `dirt_image`/`dirt_layout`
/// for soil sprites.
///
/// Assets:
///   assets/sprites/plants.png       — 96×32, 16×16 tiles, 6 cols × 2 rows (12 sprites)
///   assets/tilesets/tilled_dirt.png — 176×112, 16×16 tiles, 11 cols × 7 rows (77 sprites)
fn load_farming_atlases(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlases: ResMut<FarmingAtlases>,
) {
    if atlases.loaded {
        return;
    }

    atlases.plants_image = asset_server.load("sprites/plants.png");
    atlases.plants_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        6,
        2,
        None,
        None,
    ));

    atlases.dirt_image = asset_server.load("tilesets/tilled_dirt.png");
    atlases.dirt_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        11,
        7,
        None,
        None,
    ));

    atlases.loaded = true;
}

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers used across submodules
// ─────────────────────────────────────────────────────────────────────────────

/// Convert a grid position to a world-space translation (centre of tile).
pub fn grid_to_world(x: i32, y: i32) -> Vec3 {
    Vec3::new(
        x as f32 * TILE_SIZE,
        y as f32 * TILE_SIZE,
        // Soil tiles at z=1, crop sprites at z=2
        1.0,
    )
}

/// Check whether the given CropDef can grow in this season.
pub fn crop_can_grow_in_season(def: &CropDef, season: Season) -> bool {
    def.seasons.is_empty() || def.seasons.contains(&season)
}

/// Return the colour for a given crop stage index (0 = seedling, max = ripe).
/// Used as a placeholder when no sprite atlas is loaded.
pub fn crop_stage_color(stage: u8, total_stages: u8, dead: bool) -> Color {
    if dead {
        return Color::srgb(0.35, 0.28, 0.20); // dried-out brown
    }
    if total_stages == 0 {
        return Color::srgb(0.3, 0.7, 0.3);
    }
    let progress = stage as f32 / (total_stages.saturating_sub(1).max(1)) as f32;
    // Lerp from pale yellow-green (seedling) to vivid green/orange (mature)
    let r = 0.5 * (1.0 - progress) + 0.2 * progress;
    let g = 0.65 + 0.15 * progress;
    let b = 0.2 * (1.0 - progress);
    Color::srgb(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
}
