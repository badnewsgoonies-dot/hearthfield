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

use maps::{generate_map, MapDef};
use objects::{
    handle_forageable_pickup, handle_tool_use_on_objects, spawn_forageables, spawn_world_objects,
    WorldObject,
};

// ═══════════════════════════════════════════════════════════════════════
// PLUGIN
// ═══════════════════════════════════════════════════════════════════════

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldMap>()
            .init_resource::<CurrentMapId>()
            // Spawn the initial farm map when entering Playing state
            .add_systems(OnEnter(GameState::Playing), spawn_initial_map)
            // Gameplay systems: tool interactions, transitions, forageables
            .add_systems(
                Update,
                (
                    handle_map_transition,
                    handle_tool_use_on_objects,
                    handle_forageable_pickup,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Listen for day-end events (forageable respawn) in any state
            // so we don't miss the event
            .add_systems(Update, handle_day_end_forageables)
            // Listen for season changes for visual updates
            .add_systems(Update, handle_season_change);
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
    pub fn is_water(&self, x: i32, y: i32) -> bool {
        if let Some(ref map_def) = self.map_def {
            matches!(map_def.get_tile(x, y), TileKind::Water)
        } else {
            false
        }
    }

    /// Get the tile kind at a position.
    pub fn get_tile(&self, x: i32, y: i32) -> TileKind {
        if let Some(ref map_def) = self.map_def {
            map_def.get_tile(x, y)
        } else {
            TileKind::Void
        }
    }

    /// Get the list of map transitions for the current map.
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
// TILE COLORS
// ═══════════════════════════════════════════════════════════════════════

/// Get the color for a tile kind, optionally adjusted for season.
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

    // Spawn tile sprites
    spawn_tile_sprites(commands, &map_def, season);

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

    // Spawn world objects
    let object_placements = map_def.objects.clone();
    spawn_world_objects(commands, &object_placements, world_map);

    // Spawn forageables for today
    let forage_points = map_def.forage_points.clone();
    spawn_forageables(commands, &forage_points, season, day, world_map);

    // Store the map definition
    world_map.map_def = Some(map_def);
}

/// Spawn individual tile sprites for the map.
fn spawn_tile_sprites(commands: &mut Commands, map_def: &MapDef, season: Season) {
    for y in 0..map_def.height {
        for x in 0..map_def.width {
            let tile = map_def.tiles[y * map_def.width + x];
            let color = tile_color(tile, season);

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    0.0,
                )),
                MapTile,
            ));
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
) {
    load_map(
        &mut commands,
        MapId::Farm,
        &mut world_map,
        &mut current_map_id,
        calendar.season,
        calendar.day,
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

        // Load the new map
        load_map(
            &mut commands,
            event.to_map,
            &mut world_map,
            &mut current_map_id,
            calendar.season,
            calendar.day,
        );
    }
}

/// Handle DayEndEvent: despawn old forageables and spawn new ones.
fn handle_day_end_forageables(
    mut commands: Commands,
    mut day_events: EventReader<DayEndEvent>,
    forageable_query: Query<Entity, With<objects::Forageable>>,
    world_map: Res<WorldMap>,
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
            );
        }
    }
}

/// Handle SeasonChangeEvent: update tile colors for the new season.
fn handle_season_change(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut tile_query: Query<(&Transform, &mut Sprite), With<MapTile>>,
    world_map: Res<WorldMap>,
) {
    for event in season_events.read() {
        let new_season = event.new_season;

        if let Some(ref map_def) = world_map.map_def {
            for (transform, mut sprite) in tile_query.iter_mut() {
                // Convert world position back to grid position
                let gx = (transform.translation.x / TILE_SIZE).round() as i32;
                let gy = (transform.translation.y / TILE_SIZE).round() as i32;

                let tile = map_def.get_tile(gx, gy);
                sprite.color = tile_color(tile, new_season);
            }
        }
    }
}
