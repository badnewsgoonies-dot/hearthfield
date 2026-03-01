//! World domain plugin for Hearthfield.
//!
//! Responsible for:
//! - Loading and rendering tile maps
//! - Tracking collision/walkability
//! - Map transitions between areas
//! - World objects (trees, rocks, etc.) and tool interactions
//! - Forageable spawning per season/day
//! - Seasonal visual changes

use bevy::prelude::*;
use std::collections::HashSet;

use crate::shared::*;

pub mod maps;
pub mod objects;
pub mod chests;
pub mod seasonal;
pub mod lighting;
pub mod weather_fx;
pub mod ysort;

use maps::{generate_map, MapDef};
use objects::{
    handle_forageable_pickup, handle_tool_use_on_objects, spawn_forageables, spawn_world_objects,
    handle_weed_scythe, spawn_daily_weeds, regrow_trees_on_season_change,
    spawn_shipping_bin, spawn_crafting_bench, spawn_carpenter_board, spawn_building_signs,
    spawn_building_sprites, spawn_interior_decorations,
    WorldObject,
};
use seasonal::{
    SeasonalTintApplied, LeafSpawnAccumulator,
    apply_seasonal_tint, spawn_falling_leaves, update_falling_leaves,
};
use lighting::{
    spawn_day_night_overlay, despawn_day_night_overlay,
    update_day_night_tint, update_lightning_flash,
    LightningFlash,
};
use weather_fx::{
    spawn_weather_particles, update_weather_particles,
    cleanup_weather_on_change, cleanup_all_weather_particles,
    PreviousWeather,
};

// ═══════════════════════════════════════════════════════════════════════
// PLUGIN
// ═══════════════════════════════════════════════════════════════════════

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToastEvent>()
            .init_resource::<WorldMap>()
            .init_resource::<CurrentMapId>()
            .init_resource::<TerrainAtlases>()
            .init_resource::<objects::ObjectAtlases>()
            .init_resource::<objects::FurnitureAtlases>()
            .init_resource::<chests::ChestInteraction>()
            .init_resource::<chests::ChestSpriteData>()
            .init_resource::<SeasonalTintApplied>()
            .init_resource::<LeafSpawnAccumulator>()
            // Day/night + weather resources
            .init_resource::<DayNightTint>()
            .init_resource::<LightningFlash>()
            .init_resource::<PreviousWeather>()
            // Spawn overlay + initial map when entering Playing state
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_initial_map, spawn_day_night_overlay, chests::load_chest_sprites),
            )
            // Despawn overlay + weather particles when leaving Playing state
            .add_systems(
                OnExit(GameState::Playing),
                (despawn_day_night_overlay, cleanup_all_weather_particles),
            )
            // Gameplay systems: tool interactions, transitions, forageables
            .add_systems(
                Update,
                (
                    handle_map_transition,
                    handle_tool_use_on_objects,
                    handle_forageable_pickup,
                    chests::place_chest,
                    chests::interact_with_chest,
                    chests::close_chest_on_escape,
                    // Seasonal tinting and leaf particles
                    apply_seasonal_tint,
                    spawn_falling_leaves,
                    update_falling_leaves,
                    // Day/night ambient tint
                    update_day_night_tint,
                    update_lightning_flash,
                    // Weather particle effects
                    spawn_weather_particles,
                    update_weather_particles,
                    cleanup_weather_on_change,
                    // Weed scythe clearing
                    handle_weed_scythe,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    // Interactable object spawning (shipping bin, crafting bench, etc.)
                    spawn_shipping_bin,
                    spawn_crafting_bench,
                    spawn_carpenter_board,
                    spawn_building_signs,
                    spawn_building_sprites,
                    spawn_interior_decorations,
                    // Sync solid tiles from WorldMap into CollisionMap after map loads
                    sync_collision_map,
                    // Subtle pulse on nearby interactable objects
                    highlight_nearby_interactables,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Listen for day-end events (forageable respawn + weed spawning) in any state
            // so we don't miss the event
            .add_systems(Update, (handle_day_end_forageables, spawn_daily_weeds))
            // Listen for season changes for visual updates + tree regrowth.
            // This handles season-switch atlas swaps (index-based).
            // apply_seasonal_tint handles multiplicative colour tinting.
            .add_systems(Update, (handle_season_change, regrow_trees_on_season_change))
            // Y-sort + pixel-snap: runs after all movement, writes Transform
            .add_systems(PostUpdate, ysort::sync_position_and_ysort);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TERRAIN ATLAS RESOURCE
// ═══════════════════════════════════════════════════════════════════════

/// Caches loaded texture atlas handles for terrain tiles.
/// Loaded lazily on first map spawn.
#[derive(Resource, Default)]
pub struct TerrainAtlases {
    pub loaded: bool,
    pub grass_image: Handle<Image>,
    pub grass_layout: Handle<TextureAtlasLayout>,
    pub dirt_image: Handle<Image>,
    pub dirt_layout: Handle<TextureAtlasLayout>,
    pub water_image: Handle<Image>,
    pub water_layout: Handle<TextureAtlasLayout>,
    pub paths_image: Handle<Image>,
    pub paths_layout: Handle<TextureAtlasLayout>,
    pub bridge_image: Handle<Image>,
    pub bridge_layout: Handle<TextureAtlasLayout>,
    pub hills_image: Handle<Image>,
    pub hills_layout: Handle<TextureAtlasLayout>,
}

/// Loads all terrain atlas assets on first use. Subsequent calls are no-ops.
fn ensure_atlases_loaded(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlases: &mut TerrainAtlases,
) {
    if atlases.loaded {
        return;
    }

    // grass.png: 176x112px -> 16x16 tiles, 11 columns x 7 rows
    atlases.grass_image = asset_server.load("tilesets/grass.png");
    atlases.grass_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        11,
        7,
        None,
        None,
    ));

    // tilled_dirt.png: 176x112px -> 16x16 tiles, 11 columns x 7 rows
    atlases.dirt_image = asset_server.load("tilesets/tilled_dirt.png");
    atlases.dirt_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        11,
        7,
        None,
        None,
    ));

    // water.png: 64x16px -> 16x16 tiles, 4 columns x 1 row
    atlases.water_image = asset_server.load("tilesets/water.png");
    atlases.water_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        1,
        None,
        None,
    ));

    // paths.png: 64x64px -> 16x16 tiles, 4 columns x 4 rows
    atlases.paths_image = asset_server.load("sprites/paths.png");
    atlases.paths_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        4,
        None,
        None,
    ));

    // wood_bridge.png: 80x48px -> 16x16 tiles, 5 columns x 3 rows
    atlases.bridge_image = asset_server.load("sprites/wood_bridge.png");
    atlases.bridge_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        5,
        3,
        None,
        None,
    ));

    // hills.png: 176x144px -> 16x16 tiles, 11 columns x 9 rows
    atlases.hills_image = asset_server.load("tilesets/hills.png");
    atlases.hills_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        11,
        9,
        None,
        None,
    ));

    atlases.loaded = true;
}

// ═══════════════════════════════════════════════════════════════════════
// TILE ATLAS MAPPING
// ═══════════════════════════════════════════════════════════════════════

/// Maps a TileKind (and optionally season) to (image_handle, layout_handle, atlas_index).
/// Returns None for Void tiles, which use a plain colored sprite instead.
fn tile_atlas_info(
    kind: TileKind,
    _season: Season,
    atlases: &TerrainAtlases,
    map_id: MapId,
) -> Option<(Handle<Image>, Handle<TextureAtlasLayout>, usize)> {
    match kind {
        // Grass: use grass.png atlas. Index 5 is a nice center grass tile.
        // Different rows could represent seasonal variants; for now we pick
        // a reasonable base index per season.
        TileKind::Grass => {
            // Row 0 = basic grass. Index 5 is middle of the first row.
            let index = match _season {
                Season::Spring => 5,  // lush green center
                Season::Summer => 16, // row 1, col 5 — slightly different shade
                Season::Fall => 27,   // row 2, col 5
                Season::Winter => 38, // row 3, col 5
            };
            Some((
                atlases.grass_image.clone(),
                atlases.grass_layout.clone(),
                index,
            ))
        }

        // Dirt: use tilled_dirt.png atlas. Index 5 = plain dirt tile.
        TileKind::Dirt => Some((
            atlases.dirt_image.clone(),
            atlases.dirt_layout.clone(),
            5,
        )),

        // Tilled soil: tilled_dirt.png with a hoed-looking tile (index 12, row 1 col 1).
        TileKind::TilledSoil => Some((
            atlases.dirt_image.clone(),
            atlases.dirt_layout.clone(),
            12,
        )),

        // Watered soil: tilled_dirt.png with a darker index (index 16, row 1 col 5).
        TileKind::WateredSoil => Some((
            atlases.dirt_image.clone(),
            atlases.dirt_layout.clone(),
            16,
        )),

        // Water: water.png atlas, index 0.
        TileKind::Water => Some((
            atlases.water_image.clone(),
            atlases.water_layout.clone(),
            0,
        )),

        // Sand: use grass.png atlas with a sandy tile (row 4, col 2 = index 46).
        TileKind::Sand => Some((
            atlases.grass_image.clone(),
            atlases.grass_layout.clone(),
            46,
        )),

        // Stone: use tilled_dirt.png with a stone-looking tile (index 22, row 2 col 0).
        TileKind::Stone => Some((
            atlases.dirt_image.clone(),
            atlases.dirt_layout.clone(),
            22,
        )),

        // Wood floor: tilled_dirt.png with a wood-colored tile (index 33, row 3 col 0).
        TileKind::WoodFloor => Some((
            atlases.dirt_image.clone(),
            atlases.dirt_layout.clone(),
            33,
        )),

        // Path: paths.png atlas, index 0.
        TileKind::Path => Some((
            atlases.paths_image.clone(),
            atlases.paths_layout.clone(),
            5, // center path tile (row 1, col 1)
        )),

        // Bridge: wood_bridge.png atlas, center plank tile (row 1, col 2 = index 7).
        TileKind::Bridge => Some((
            atlases.bridge_image.clone(),
            atlases.bridge_layout.clone(),
            7,
        )),

        // Void: hills for outdoor maps, wall tile for indoor maps.
        TileKind::Void => {
            let is_indoor = matches!(
                map_id,
                MapId::PlayerHouse | MapId::GeneralStore | MapId::AnimalShop | MapId::Blacksmith
            );
            if is_indoor {
                // Use tilled_dirt as a dark wall texture (index 0 = top-left corner)
                Some((
                    atlases.dirt_image.clone(),
                    atlases.dirt_layout.clone(),
                    0,
                ))
            } else {
                // Outdoor: use hills for natural cliff edge
                Some((
                    atlases.hills_image.clone(),
                    atlases.hills_layout.clone(),
                    60,
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Tracks the currently loaded map and provides collision/walkability queries.
#[derive(Resource, Debug, Clone)]
pub struct WorldMap {
    /// The current map definition.
    pub map_def: Option<MapDef>,
    /// Set of solid tile positions (not walkable).
    pub solid_tiles: HashSet<(i32, i32)>,
    /// Map width in tiles.
    pub width: usize,
    /// Map height in tiles.
    pub height: usize,
}

impl Default for WorldMap {
    fn default() -> Self {
        Self {
            map_def: None,
            solid_tiles: HashSet::new(),
            width: 0,
            height: 0,
        }
    }
}

impl WorldMap {
    /// Check if a tile position is walkable.
    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        // Out of bounds is not walkable
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }

        // Check solid objects
        if self.solid_tiles.contains(&(x, y)) {
            return false;
        }

        // Check tile type
        if let Some(ref map_def) = self.map_def {
            let tile = map_def.get_tile(x, y);
            !matches!(tile, TileKind::Water | TileKind::Void)
        } else {
            false
        }
    }

    /// Check if a tile is solid (object or terrain).
    pub fn is_solid(&self, x: i32, y: i32) -> bool {
        !self.is_walkable(x, y)
    }

    /// Mark a tile as solid or clear it.
    pub fn set_solid(&mut self, x: i32, y: i32, solid: bool) {
        if solid {
            self.solid_tiles.insert((x, y));
        } else {
            self.solid_tiles.remove(&(x, y));
        }
    }

    /// Check if a tile is water.
    #[allow(dead_code)]
    pub fn is_water(&self, x: i32, y: i32) -> bool {
        if let Some(ref map_def) = self.map_def {
            matches!(map_def.get_tile(x, y), TileKind::Water)
        } else {
            false
        }
    }

    /// Get the tile kind at a position.
    #[allow(dead_code)]
    pub fn get_tile(&self, x: i32, y: i32) -> TileKind {
        if let Some(ref map_def) = self.map_def {
            map_def.get_tile(x, y)
        } else {
            TileKind::Void
        }
    }

    /// Get the list of map transitions for the current map.
    #[allow(dead_code)]
    pub fn transitions(&self) -> &[MapTransition] {
        if let Some(ref map_def) = self.map_def {
            &map_def.transitions
        } else {
            &[]
        }
    }
}

/// Simple resource to track the currently loaded map ID.
#[derive(Resource, Debug, Clone)]
pub struct CurrentMapId {
    pub map_id: MapId,
}

impl Default for CurrentMapId {
    fn default() -> Self {
        Self {
            map_id: MapId::Farm,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker component for tile sprite entities (for bulk despawn).
#[derive(Component, Debug)]
pub struct MapTile;

/// Marker component for transition zone entities.
#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct TransitionZone {
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
    pub rect_x: i32,
    pub rect_y: i32,
    pub rect_w: i32,
    pub rect_h: i32,
}

// ═══════════════════════════════════════════════════════════════════════
// TILE COLORS (fallback for Void tiles and season change)
// ═══════════════════════════════════════════════════════════════════════

/// Get the color for a tile kind, optionally adjusted for season.
/// Kept as a fallback for Void tiles and the season change system.
fn tile_color(kind: TileKind, season: Season) -> Color {
    match kind {
        TileKind::Grass => match season {
            Season::Spring => Color::srgb(0.3, 0.72, 0.32),
            Season::Summer => Color::srgb(0.35, 0.68, 0.28),
            Season::Fall => Color::srgb(0.6, 0.5, 0.25),
            Season::Winter => Color::srgb(0.75, 0.82, 0.88),
        },
        TileKind::Dirt => match season {
            Season::Spring => Color::srgb(0.6, 0.45, 0.3),
            Season::Summer => Color::srgb(0.62, 0.47, 0.28),
            Season::Fall => Color::srgb(0.55, 0.4, 0.25),
            Season::Winter => Color::srgb(0.65, 0.6, 0.58),
        },
        TileKind::TilledSoil => Color::srgb(0.45, 0.32, 0.2),
        TileKind::WateredSoil => Color::srgb(0.3, 0.22, 0.15),
        TileKind::Water => match season {
            Season::Spring => Color::srgb(0.2, 0.4, 0.82),
            Season::Summer => Color::srgb(0.22, 0.42, 0.78),
            Season::Fall => Color::srgb(0.18, 0.35, 0.7),
            Season::Winter => Color::srgb(0.5, 0.65, 0.82),
        },
        TileKind::Sand => Color::srgb(0.9, 0.85, 0.6),
        TileKind::Stone => match season {
            Season::Winter => Color::srgb(0.6, 0.62, 0.68),
            _ => Color::srgb(0.5, 0.5, 0.55),
        },
        TileKind::WoodFloor => Color::srgb(0.65, 0.5, 0.3),
        TileKind::Path => match season {
            Season::Winter => Color::srgb(0.78, 0.75, 0.72),
            _ => Color::srgb(0.7, 0.65, 0.5),
        },
        TileKind::Bridge => Color::srgb(0.55, 0.42, 0.25),
        TileKind::Void => Color::srgb(0.08, 0.08, 0.1),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MAP LOADING AND RENDERING
// ═══════════════════════════════════════════════════════════════════════

/// Load a map by ID: populate WorldMap resource and spawn tile entities.
fn load_map(
    commands: &mut Commands,
    map_id: MapId,
    world_map: &mut WorldMap,
    current_map_id: &mut CurrentMapId,
    season: Season,
    day: u8,
    atlases: &TerrainAtlases,
    object_atlases: &objects::ObjectAtlases,
) {
    let map_def = generate_map(map_id);

    // Update tracking
    current_map_id.map_id = map_id;
    world_map.width = map_def.width;
    world_map.height = map_def.height;
    world_map.solid_tiles.clear();

    // Mark inherently solid tiles
    for y in 0..map_def.height {
        for x in 0..map_def.width {
            let tile = map_def.tiles[y * map_def.width + x];
            if matches!(tile, TileKind::Water | TileKind::Void) {
                world_map.solid_tiles.insert((x as i32, y as i32));
            }
        }
    }

    // Spawn tile sprites using texture atlases
    spawn_tile_sprites(commands, &map_def, season, atlases);

    // Spawn transition zone markers
    for transition in &map_def.transitions {
        commands.spawn((
            MapTile, // Despawn with the rest of the map
            TransitionZone {
                to_map: transition.to_map,
                to_x: transition.to_pos.0,
                to_y: transition.to_pos.1,
                rect_x: transition.from_rect.0,
                rect_y: transition.from_rect.1,
                rect_w: transition.from_rect.2,
                rect_h: transition.from_rect.3,
            },
        ));
    }

    // Spawn world objects with atlas sprites
    let object_placements = map_def.objects.clone();
    spawn_world_objects(commands, &object_placements, world_map, object_atlases);

    // Spawn forageables for today
    let forage_points = map_def.forage_points.clone();
    spawn_forageables(commands, &forage_points, season, day, world_map, object_atlases);

    // Store the map definition
    world_map.map_def = Some(map_def);
}

/// Spawn individual tile sprites for the map using texture atlases.
fn spawn_tile_sprites(
    commands: &mut Commands,
    map_def: &MapDef,
    season: Season,
    atlases: &TerrainAtlases,
) {
    for y in 0..map_def.height {
        for x in 0..map_def.width {
            let tile = map_def.tiles[y * map_def.width + x];

            match tile_atlas_info(tile, season, atlases, map_def.id) {
                Some((image, layout, index)) => {
                    // Use texture atlas sprite
                    commands.spawn((
                        {
                            let mut sprite = Sprite::from_atlas_image(
                                image,
                                TextureAtlas {
                                    layout,
                                    index,
                                },
                            );
                            sprite.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
                            sprite
                        },
                        Transform::from_translation(Vec3::new(
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                            Z_GROUND,
                        )),
                        MapTile,
                    ));
                }
                None => {
                    // Void tile: use plain colored sprite (no texture needed)
                    commands.spawn((
                        Sprite {
                            color: tile_color(tile, season),
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                            Z_GROUND,
                        )),
                        MapTile,
                    ));
                }
            }
        }
    }
}

/// Despawn all map tiles and world objects.
fn despawn_map(
    commands: &mut Commands,
    tile_query: &Query<Entity, With<MapTile>>,
    object_query: &Query<Entity, With<WorldObject>>,
) {
    for entity in tile_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in object_query.iter() {
        commands.entity(entity).despawn();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn the initial farm map when the game enters Playing state.
fn spawn_initial_map(
    mut commands: Commands,
    mut world_map: ResMut<WorldMap>,
    mut current_map_id: ResMut<CurrentMapId>,
    calendar: Res<Calendar>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut terrain_atlases: ResMut<TerrainAtlases>,
    mut object_atlases: ResMut<objects::ObjectAtlases>,
    mut furniture_atlases: ResMut<objects::FurnitureAtlases>,
    existing_tiles: Query<Entity, With<MapTile>>,
) {
    // Guard against re-entry (e.g. Playing → Cutscene → Playing).
    if !existing_tiles.is_empty() {
        return;
    }

    // Ensure terrain atlases are loaded
    ensure_atlases_loaded(&asset_server, &mut atlas_layouts, &mut terrain_atlases);
    // Ensure object atlases are loaded
    objects::ensure_object_atlases_loaded(&asset_server, &mut atlas_layouts, &mut object_atlases);
    // Ensure furniture atlases are loaded
    objects::ensure_furniture_atlases_loaded(&asset_server, &mut atlas_layouts, &mut furniture_atlases);

    load_map(
        &mut commands,
        MapId::Farm,
        &mut world_map,
        &mut current_map_id,
        calendar.season,
        calendar.day,
        &terrain_atlases,
        &object_atlases,
    );
}

/// Handle MapTransitionEvent: despawn current map, load new one.
fn handle_map_transition(
    mut commands: Commands,
    mut events: EventReader<MapTransitionEvent>,
    tile_query: Query<Entity, With<MapTile>>,
    object_query: Query<Entity, With<WorldObject>>,
    mut world_map: ResMut<WorldMap>,
    mut current_map_id: ResMut<CurrentMapId>,
    mut player_state: ResMut<PlayerState>,
    calendar: Res<Calendar>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut terrain_atlases: ResMut<TerrainAtlases>,
    mut object_atlases: ResMut<objects::ObjectAtlases>,
    mut furniture_atlases: ResMut<objects::FurnitureAtlases>,
) {
    for event in events.read() {
        // Don't transition to the same map
        if event.to_map == current_map_id.map_id {
            continue;
        }

        // Despawn current map
        despawn_map(&mut commands, &tile_query, &object_query);

        // Update player's current map
        player_state.current_map = event.to_map;

        // Ensure atlases are loaded (in case they weren't yet)
        ensure_atlases_loaded(&asset_server, &mut atlas_layouts, &mut terrain_atlases);
        objects::ensure_object_atlases_loaded(
            &asset_server,
            &mut atlas_layouts,
            &mut object_atlases,
        );
        objects::ensure_furniture_atlases_loaded(
            &asset_server,
            &mut atlas_layouts,
            &mut furniture_atlases,
        );

        // Load the new map
        load_map(
            &mut commands,
            event.to_map,
            &mut world_map,
            &mut current_map_id,
            calendar.season,
            calendar.day,
            &terrain_atlases,
            &object_atlases,
        );
    }
}

/// Handle DayEndEvent: despawn old forageables and spawn new ones.
fn handle_day_end_forageables(
    mut commands: Commands,
    mut day_events: EventReader<DayEndEvent>,
    forageable_query: Query<Entity, With<objects::Forageable>>,
    world_map: Res<WorldMap>,
    object_atlases: Res<objects::ObjectAtlases>,
) {
    for event in day_events.read() {
        // Despawn existing forageables
        for entity in forageable_query.iter() {
            commands.entity(entity).despawn();
        }

        // Spawn new forageables for the new day
        if let Some(ref map_def) = world_map.map_def {
            let forage_points = map_def.forage_points.clone();
            spawn_forageables(
                &mut commands,
                &forage_points,
                event.season,
                event.day,
                &world_map,
                &object_atlases,
            );
        }
    }
}

/// Keep CollisionMap in sync with WorldMap whenever solid_tiles change
/// (initial load, object destruction, map transition, etc.).
pub fn sync_collision_map(
    world_map: Res<WorldMap>,
    mut collision_map: ResMut<crate::player::CollisionMap>,
) {
    if !world_map.is_changed() {
        return;
    }
    collision_map.solid_tiles.clone_from(&world_map.solid_tiles);
    if !world_map.solid_tiles.is_empty() {
        collision_map.initialised = true;
    }
}

/// Handle SeasonChangeEvent: update tile sprites for the new season.
/// For atlas-based tiles, we swap the atlas index to the seasonal variant.
/// For Void tiles (plain colored), we leave them as-is.
///
/// Also resets `SeasonalTintApplied` so that `apply_seasonal_tint` will
/// re-apply the new season's colour tint on the next frame (after the atlas
/// indices have been updated).
fn handle_season_change(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut tile_query: Query<(&Transform, &mut Sprite), With<MapTile>>,
    world_map: Res<WorldMap>,
    terrain_atlases: Res<TerrainAtlases>,
    mut tint_applied: ResMut<SeasonalTintApplied>,
) {
    for event in season_events.read() {
        let new_season = event.new_season;

        if let Some(ref map_def) = world_map.map_def {
            for (transform, mut sprite) in tile_query.iter_mut() {
                // Convert world position back to grid position
                let g = world_to_grid(transform.translation.x, transform.translation.y);
                let gx = g.x;
                let gy = g.y;

                let tile = map_def.get_tile(gx, gy);

                match tile_atlas_info(tile, new_season, &terrain_atlases, map_def.id) {
                    Some((image, layout, index)) => {
                        // Update the sprite to use the new seasonal atlas image and index.
                        // Reset color to white so apply_seasonal_tint can tint cleanly.
                        *sprite = Sprite::from_atlas_image(
                            image,
                            TextureAtlas { layout, index },
                        );
                    }
                    None => {
                        // Void tile: update color fallback
                        sprite.color = tile_color(tile, new_season);
                    }
                }
            }
        }

        // Force apply_seasonal_tint to re-run on the next frame so the new
        // season's colour tint is applied over the freshly-swapped atlas tiles.
        tint_applied.season = None;
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTABLE HIGHLIGHT — subtle pulse on nearby interactable objects
// ═══════════════════════════════════════════════════════════════════════

/// Gently brighten interactable sprites when the player is within interaction range.
pub fn highlight_nearby_interactables(
    time: Res<Time>,
    player_query: Query<&LogicalPosition, With<Player>>,
    mut interactable_query: Query<(&Transform, &mut Sprite), With<Interactable>>,
) {
    let Ok(player_pos) = player_query.get_single() else {
        return;
    };
    let range = TILE_SIZE * 1.8;
    let pulse = 1.0 + 0.15 * (time.elapsed_secs() * 3.0).sin().abs();

    for (tf, mut sprite) in &mut interactable_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range {
            sprite.color = Color::srgb(pulse, pulse, pulse);
        } else {
            sprite.color = Color::WHITE;
        }
    }
}
