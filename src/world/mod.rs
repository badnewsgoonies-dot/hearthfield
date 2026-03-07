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

pub mod chests;
pub mod lighting;
pub mod maps;
pub mod objects;
pub mod seasonal;
pub mod weather_fx;
pub mod ysort;

use lighting::{
    despawn_day_night_overlay, spawn_day_night_overlay, update_day_night_tint,
    update_lightning_flash, LightningFlash,
};
use maps::{generate_map, MapDef};
use objects::{
    handle_forageable_pickup, handle_tool_use_on_objects, handle_weed_scythe,
    regrow_trees_on_season_change, spawn_building_signs, spawn_building_sprites,
    spawn_carpenter_board, spawn_crafting_bench, spawn_daily_weeds, spawn_forageables,
    spawn_interior_decorations, spawn_shipping_bin, spawn_world_objects, update_forage_sparkles,
    update_tree_sprites_on_season_change, WorldObject,
};
use seasonal::{
    apply_seasonal_tint, spawn_falling_leaves, update_falling_leaves, LeafSpawnAccumulator,
    SeasonalTintApplied,
};
use weather_fx::{
    cleanup_all_weather_particles, cleanup_weather_on_change, spawn_weather_particles,
    update_weather_particles, weather_change_notification, PreviousWeather, WeatherParticleCounts,
};

// ═══════════════════════════════════════════════════════════════════════
// PLUGIN
// ═══════════════════════════════════════════════════════════════════════

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldMap>()
            .init_resource::<CurrentMapId>()
            .init_resource::<TerrainAtlases>()
            .init_resource::<objects::ObjectAtlases>()
            .init_resource::<objects::FurnitureAtlases>()
            .init_resource::<chests::ChestInteraction>()
            .init_resource::<chests::ChestSpriteData>()
            .init_resource::<SeasonalTintApplied>()
            .init_resource::<LeafSpawnAccumulator>()
            .init_resource::<WaterAnimationTimer>()
            .init_resource::<WaterEdgePhase>()
            // Day/night + weather resources
            .init_resource::<DayNightTint>()
            .init_resource::<LightningFlash>()
            .init_resource::<PreviousWeather>()
            .init_resource::<WeatherParticleCounts>()
            // Spawn overlay + initial map when entering Playing state
            .add_systems(
                OnEnter(GameState::Playing),
                (
                    spawn_initial_map,
                    spawn_day_night_overlay,
                    chests::load_chest_sprites,
                ),
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
                    // Weed scythe clearing
                    handle_weed_scythe,
                )
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
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
                    weather_change_notification,
                    // Forageable sparkle particles
                    update_forage_sparkles,
                    // Water tile animation
                    animate_water_tiles,
                )
                    .in_set(UpdatePhase::Presentation)
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
                )
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    // Subtle pulse on nearby interactable objects
                    highlight_nearby_interactables,
                )
                    .in_set(UpdatePhase::Presentation)
                    .run_if(in_state(GameState::Playing)),
            )
            // Listen for day-end events (forageable respawn + weed spawning) in any state
            // so we don't miss the event
            .add_systems(
                Update,
                (handle_day_end_forageables, spawn_daily_weeds).in_set(UpdatePhase::Reactions),
            )
            // Listen for season changes for visual updates + tree regrowth.
            // This handles season-switch atlas swaps (index-based).
            // apply_seasonal_tint handles multiplicative colour tinting.
            .add_systems(
                Update,
                (
                    handle_season_change,
                    regrow_trees_on_season_change,
                    update_tree_sprites_on_season_change,
                )
                    .in_set(UpdatePhase::Reactions),
            )
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
fn is_path_neighbor(
    tiles: &[TileKind],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    dx: i32,
    dy: i32,
) -> bool {
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
        return false;
    }
    matches!(
        tiles[ny as usize * width + nx as usize],
        TileKind::Path | TileKind::Bridge
    )
}

fn path_autotile_index(bitmask: u8) -> usize {
    match bitmask {
        0b0000 => 0,
        0b0001 => 1,
        0b0010 => 2,
        0b0011 => 3,
        0b0100 => 4,
        0b0101 => 5,
        0b0110 => 6,
        0b0111 => 7,
        0b1000 => 8,
        0b1001 => 9,
        0b1010 => 10,
        0b1011 => 11,
        0b1100 => 12,
        0b1101 => 13,
        0b1110 => 14,
        0b1111 => 15,
        _ => 5,
    }
}

#[allow(clippy::too_many_arguments)]
fn tile_atlas_info(
    kind: TileKind,
    _season: Season,
    atlases: &TerrainAtlases,
    map_id: MapId,
    x: usize,
    y: usize,
    tiles: &[TileKind],
    width: usize,
    height: usize,
) -> Option<(Handle<Image>, Handle<TextureAtlasLayout>, usize)> {
    match kind {
        // Grass: use grass.png atlas. Index 5 is a nice center grass tile.
        // Different rows could represent seasonal variants; for now we pick
        // a reasonable base index per season.
        TileKind::Grass => {
            // Use positional hash for visual variety across the grass tileset.
            // grass.png is 11 cols x 7 rows = 77 frames. Each season gets a row
            // band of ~11 frames. We pick from 4 variants per season using a
            // deterministic hash of (x, y) so it's stable without runtime cost.
            let variant = (x.wrapping_mul(7).wrapping_add(y.wrapping_mul(13))) % 4;
            let season_base = match _season {
                Season::Spring => 4,  // row 0, starting at col 4
                Season::Summer => 15, // row 1, starting at col 4
                Season::Fall => 26,   // row 2, starting at col 4
                Season::Winter => 37, // row 3, starting at col 4
            };
            Some((
                atlases.grass_image.clone(),
                atlases.grass_layout.clone(),
                season_base + variant,
            ))
        }

        // Dirt: use tilled_dirt.png atlas. Index 5 = plain dirt tile.
        TileKind::Dirt => Some((atlases.dirt_image.clone(), atlases.dirt_layout.clone(), 5)),

        // Tilled soil: tilled_dirt.png with a hoed-looking tile (index 12, row 1 col 1).
        TileKind::TilledSoil => Some((atlases.dirt_image.clone(), atlases.dirt_layout.clone(), 12)),

        // Watered soil: tilled_dirt.png with a darker index (index 16, row 1 col 5).
        TileKind::WateredSoil => {
            Some((atlases.dirt_image.clone(), atlases.dirt_layout.clone(), 16))
        }

        // Water: water.png atlas, index 0.
        TileKind::Water => Some((atlases.water_image.clone(), atlases.water_layout.clone(), 0)),

        // Sand: use grass.png atlas with a sandy tile (row 4, col 2 = index 46).
        TileKind::Sand => Some((
            atlases.grass_image.clone(),
            atlases.grass_layout.clone(),
            46,
        )),

        // Stone: use hills.png for a proper rocky/stone texture (index 0, top-left).
        TileKind::Stone => Some((atlases.hills_image.clone(), atlases.hills_layout.clone(), 0)),

        // Wood floor: tilled_dirt.png with a plank-like tile (index 6, row 0 col 6).
        TileKind::WoodFloor => Some((atlases.dirt_image.clone(), atlases.dirt_layout.clone(), 6)),

        TileKind::Path => {
            let mut mask: u8 = 0;
            if is_path_neighbor(tiles, x, y, width, height, 0, -1) {
                mask |= 1;
            } // north
            if is_path_neighbor(tiles, x, y, width, height, 1, 0) {
                mask |= 2;
            } // east
            if is_path_neighbor(tiles, x, y, width, height, 0, 1) {
                mask |= 4;
            } // south
            if is_path_neighbor(tiles, x, y, width, height, -1, 0) {
                mask |= 8;
            } // west
            Some((
                atlases.paths_image.clone(),
                atlases.paths_layout.clone(),
                path_autotile_index(mask),
            ))
        }

        // Bridge: wood_bridge.png atlas, center plank tile (row 1, col 2 = index 7).
        TileKind::Bridge => Some((
            atlases.bridge_image.clone(),
            atlases.bridge_layout.clone(),
            7,
        )),

        // Void: hills for outdoor maps, dark color for indoor maps.
        TileKind::Void => {
            let is_indoor = matches!(
                map_id,
                MapId::PlayerHouse | MapId::GeneralStore | MapId::AnimalShop | MapId::Blacksmith
            );
            if is_indoor {
                // Return None → solid dark color fallback via tile_color()
                None
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
#[derive(Resource, Debug, Clone, Default)]
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

/// Marker component for water tile sprites (for animation cycling).
#[derive(Component, Debug)]
pub struct WaterTile;

/// Bitmask indicating which edges of a water tile border non-water tiles.
/// bit 0 = north, bit 1 = east, bit 2 = south, bit 3 = west.
#[derive(Component, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct WaterEdgeMask(pub u8);

/// Marker component for water edge overlay sprites.
/// Tagged with MapTile so they despawn with the map.
#[derive(Component, Debug)]
pub struct WaterEdgeOverlay;

/// Timer resource for water tile animation (cycles 4 frames).
#[derive(Resource)]
pub struct WaterAnimationTimer(pub Timer);

impl Default for WaterAnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Repeating))
    }
}

/// Tracks the current animation phase (0-3) for water edge overlay alpha pulsing.
#[derive(Resource, Default)]
pub struct WaterEdgePhase(pub u8);

// ═══════════════════════════════════════════════════════════════════════
// TILE COLORS (fallback for Void tiles and season change)
// ═══════════════════════════════════════════════════════════════════════

/// Get the color for a tile kind, optionally adjusted for season.
/// Kept as a fallback for Void tiles and the season change system.
fn tile_color(kind: TileKind, season: Season) -> Color {
    match kind {
        TileKind::Grass => match season {
            Season::Spring => Color::srgb(0.28, 0.71, 0.25),
            Season::Summer => Color::srgb(0.22, 0.65, 0.20),
            Season::Fall => Color::srgb(0.63, 0.55, 0.24),
            Season::Winter => Color::srgb(0.75, 0.82, 0.78),
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
            Season::Spring => Color::srgb(0.14, 0.35, 0.57),
            Season::Summer => Color::srgb(0.16, 0.38, 0.55),
            Season::Fall => Color::srgb(0.12, 0.30, 0.50),
            Season::Winter => Color::srgb(0.35, 0.50, 0.65),
        },
        TileKind::Sand => Color::srgb(0.9, 0.85, 0.6),
        TileKind::Stone => match season {
            Season::Winter => Color::srgb(0.6, 0.62, 0.68),
            _ => Color::srgb(0.5, 0.5, 0.55),
        },
        TileKind::WoodFloor => Color::srgb(0.65, 0.5, 0.3),
        TileKind::Path => match season {
            Season::Winter => Color::srgb(0.72, 0.68, 0.62),
            _ => Color::srgb(0.69, 0.55, 0.37),
        },
        TileKind::Bridge => Color::srgb(0.55, 0.42, 0.25),
        TileKind::Void => Color::srgb(0.08, 0.08, 0.1),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MAP LOADING AND RENDERING
// ═══════════════════════════════════════════════════════════════════════

/// Load a map by ID: populate WorldMap resource and spawn tile entities.
#[allow(clippy::too_many_arguments)]
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
    // Stone is solid on outdoor maps (building walls) but walkable in the Mine
    // and interior maps (where it serves as floor/counters).
    let stone_is_solid = matches!(
        map_id,
        MapId::Farm | MapId::Town | MapId::Beach | MapId::Forest | MapId::MineEntrance
    );
    for y in 0..map_def.height {
        for x in 0..map_def.width {
            let tile = map_def.tiles[y * map_def.width + x];
            if matches!(tile, TileKind::Water | TileKind::Void) {
                world_map.solid_tiles.insert((x as i32, y as i32));
            }
            if stone_is_solid && tile == TileKind::Stone {
                world_map.solid_tiles.insert((x as i32, y as i32));
            }
        }
    }

    // Exempt building door tiles so players can still trigger transitions.
    // These coordinates match the door-entry zones in edge_transition()
    // (src/player/interaction.rs).
    if map_id == MapId::Farm {
        // Player House door at (15-16, 2) and exit landing tiles (15-16, 1)
        world_map.solid_tiles.remove(&(15, 2));
        world_map.solid_tiles.remove(&(16, 2));
        world_map.solid_tiles.remove(&(15, 1));
        world_map.solid_tiles.remove(&(16, 1));
        // Also clear a path south from the house (15-16, 0) so player can walk away
        world_map.solid_tiles.remove(&(15, 0));
        world_map.solid_tiles.remove(&(16, 0));
    }
    if map_id == MapId::Town {
        // General Store door at (5-6, 2)
        world_map.solid_tiles.remove(&(5, 2));
        world_map.solid_tiles.remove(&(6, 2));
        // Animal Shop door at (22-23, 2)
        world_map.solid_tiles.remove(&(22, 2));
        world_map.solid_tiles.remove(&(23, 2));
        // Blacksmith door at (22-23, 13)
        world_map.solid_tiles.remove(&(22, 13));
        world_map.solid_tiles.remove(&(23, 13));
    }

    // Spawn tile sprites using texture atlases
    spawn_tile_sprites(commands, &map_def, season, atlases);

    // Spawn world objects with atlas sprites
    let object_placements = map_def.objects.clone();
    spawn_world_objects(
        commands,
        &object_placements,
        world_map,
        object_atlases,
        season,
    );

    // Spawn forageables for today
    let forage_points = map_def.forage_points.clone();
    spawn_forageables(
        commands,
        &forage_points,
        season,
        day,
        world_map,
        object_atlases,
    );

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

            match tile_atlas_info(
                tile,
                season,
                atlases,
                map_def.id,
                x,
                y,
                &map_def.tiles,
                map_def.width,
                map_def.height,
            ) {
                Some((image, layout, index)) => {
                    // Use texture atlas sprite
                    let mut entity_cmd = commands.spawn((
                        {
                            let mut sprite =
                                Sprite::from_atlas_image(image, TextureAtlas { layout, index });
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
                    // Tag water tiles for animation cycling and spawn edge overlays
                    if tile == TileKind::Water {
                        let mask = water_edge_mask(x, y, &map_def.tiles, map_def.width, map_def.height);
                        entity_cmd.insert((WaterTile, WaterEdgeMask(mask)));
                        if mask != 0 {
                            spawn_water_edge_overlays(commands, x, y, mask, season);
                        }
                    }
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


/// Compute the water edge bitmask for the tile at (x, y).
/// bit 0 = north (y+1), bit 1 = east (x+1), bit 2 = south (y-1), bit 3 = west (x-1).
/// A bit is set when the neighbor in that direction is non-water (or out of bounds).
fn water_edge_mask(x: usize, y: usize, tiles: &[TileKind], width: usize, height: usize) -> u8 {
    let mut mask: u8 = 0;
    let is_non_water = |nx: i32, ny: i32| -> bool {
        if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
            return true;
        }
        tiles[ny as usize * width + nx as usize] != TileKind::Water
    };
    if is_non_water(x as i32, y as i32 + 1) { mask |= 0b0001; } // north
    if is_non_water(x as i32 + 1, y as i32) { mask |= 0b0010; } // east
    if is_non_water(x as i32, y as i32 - 1) { mask |= 0b0100; } // south
    if is_non_water(x as i32 - 1, y as i32) { mask |= 0b1000; } // west
    mask
}

/// Spawn semi-transparent overlay sprites on the edges of a water tile that
/// border non-water tiles. Each overlay is a thin stripe of water colour
/// with alpha so it blends softly against the adjacent land tile.
fn spawn_water_edge_overlays(
    commands: &mut Commands,
    x: usize,
    y: usize,
    mask: u8,
    season: Season,
) {
    let cx = x as f32 * TILE_SIZE;
    let cy = y as f32 * TILE_SIZE;
    let z = Z_GROUND + 0.1;

    let water_color = {
        let base = match season {
            Season::Spring => Color::srgb(0.14, 0.35, 0.57),
            Season::Summer => Color::srgb(0.16, 0.38, 0.55),
            Season::Fall => Color::srgb(0.12, 0.30, 0.50),
            Season::Winter => Color::srgb(0.35, 0.50, 0.65),
        };
        let [r, g, b, _] = base.to_srgba().to_f32_array();
        Color::srgba(r, g, b, 0.40)
    };

    let half = TILE_SIZE / 2.0;
    let edge_thick = 4.0;
    let edge_offset = half - edge_thick / 2.0;

    if mask & 0b0001 != 0 {
        commands.spawn((
            Sprite { color: water_color, custom_size: Some(Vec2::new(TILE_SIZE, edge_thick)), ..default() },
            Transform::from_translation(Vec3::new(cx, cy + edge_offset, z)),
            MapTile, WaterEdgeOverlay,
        ));
    }
    if mask & 0b0010 != 0 {
        commands.spawn((
            Sprite { color: water_color, custom_size: Some(Vec2::new(edge_thick, TILE_SIZE)), ..default() },
            Transform::from_translation(Vec3::new(cx + edge_offset, cy, z)),
            MapTile, WaterEdgeOverlay,
        ));
    }
    if mask & 0b0100 != 0 {
        commands.spawn((
            Sprite { color: water_color, custom_size: Some(Vec2::new(TILE_SIZE, edge_thick)), ..default() },
            Transform::from_translation(Vec3::new(cx, cy - edge_offset, z)),
            MapTile, WaterEdgeOverlay,
        ));
    }
    if mask & 0b1000 != 0 {
        commands.spawn((
            Sprite { color: water_color, custom_size: Some(Vec2::new(edge_thick, TILE_SIZE)), ..default() },
            Transform::from_translation(Vec3::new(cx - edge_offset, cy, z)),
            MapTile, WaterEdgeOverlay,
        ));
    }
}

/// Despawn all map tiles and world objects.
fn despawn_map(
    commands: &mut Commands,
    tile_query: &Query<Entity, With<MapTile>>,
    object_query: &Query<Entity, With<WorldObject>>,
) {
    for entity in tile_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in object_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Animate water tiles by cycling through 4 atlas frames, and pulse edge overlay alpha.
fn animate_water_tiles(
    time: Res<Time>,
    mut timer: ResMut<WaterAnimationTimer>,
    mut phase: ResMut<WaterEdgePhase>,
    mut water_query: Query<&mut Sprite, With<WaterTile>>,
    mut overlay_query: Query<&mut Sprite, (With<WaterEdgeOverlay>, Without<WaterTile>)>,
) {
    // Alpha values for the 4 phases: ramp up then back down for a gentle pulse.
    const EDGE_ALPHAS: [f32; 4] = [0.30, 0.40, 0.50, 0.40];

    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        for mut sprite in water_query.iter_mut() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = (atlas.index + 1) % 4;
            }
        }
        phase.0 = (phase.0 + 1) % 4;
        let alpha = EDGE_ALPHAS[phase.0 as usize];
        for mut sprite in overlay_query.iter_mut() {
            let [r, g, b, _] = sprite.color.to_srgba().to_f32_array();
            sprite.color = Color::srgba(r, g, b, alpha);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn the initial farm map when the game enters Playing state.
#[allow(clippy::too_many_arguments)]
fn spawn_initial_map(
    mut commands: Commands,
    mut world_map: ResMut<WorldMap>,
    mut current_map_id: ResMut<CurrentMapId>,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
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
    objects::ensure_furniture_atlases_loaded(
        &asset_server,
        &mut atlas_layouts,
        &mut furniture_atlases,
    );

    load_map(
        &mut commands,
        player_state.current_map,
        &mut world_map,
        &mut current_map_id,
        calendar.season,
        calendar.day,
        &terrain_atlases,
        &object_atlases,
    );
}

/// Handle MapTransitionEvent: despawn current map, load new one.
#[allow(clippy::too_many_arguments)]
fn handle_map_transition(
    mut commands: Commands,
    mut events: EventReader<MapTransitionEvent>,
    tile_query: Query<Entity, With<MapTile>>,
    object_query: Query<Entity, With<WorldObject>>,
    mut world_map: ResMut<WorldMap>,
    mut current_map_id: ResMut<CurrentMapId>,
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
    if world_map.width > 0 && world_map.height > 0 {
        collision_map.bounds = (
            0,
            world_map.width as i32 - 1,
            0,
            world_map.height as i32 - 1,
        );
        collision_map.initialised = true;
    }
}

type SeasonTileQuery<'w, 's> = Query<'w, 's, (&'static Transform, &'static mut Sprite), (With<MapTile>, Without<WaterEdgeOverlay>)>;

/// Handle SeasonChangeEvent: update tile sprites for the new season.
/// For atlas-based tiles, we swap the atlas index to the seasonal variant.
/// For Void tiles (plain colored), we leave them as-is.
///
/// Also resets `SeasonalTintApplied` so that `apply_seasonal_tint` will
/// re-apply the new season's colour tint on the next frame (after the atlas
/// indices have been updated).
fn handle_season_change(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut tile_query: SeasonTileQuery,
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

                match tile_atlas_info(
                    tile,
                    new_season,
                    &terrain_atlases,
                    map_def.id,
                    gx as usize,
                    gy as usize,
                    &map_def.tiles,
                    map_def.width,
                    map_def.height,
                ) {
                    Some((image, layout, index)) => {
                        // Update the sprite to use the new seasonal atlas image and index.
                        // Reset color to white so apply_seasonal_tint can tint cleanly.
                        *sprite = Sprite::from_atlas_image(image, TextureAtlas { layout, index });
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
