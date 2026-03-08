use bevy::prelude::*;
use std::collections::HashSet;

use crate::shared::{
    GameState, GridPosition, MapId, MapTransition, MapTransitionEvent, PlayerState, TileKind,
    UpdatePhase, PIXEL_SCALE, TILE_SIZE,
};

const PRECINCT_WIDTH: usize = 32;
const PRECINCT_HEIGHT: usize = 24;
const TILE_WORLD_SIZE: f32 = TILE_SIZE * PIXEL_SCALE;
const PRECINCT_INTERIOR_SPAWN: GridPosition = GridPosition { x: 16, y: 20 };
const SOUTH_EXIT_POSITION: GridPosition = GridPosition { x: 16, y: 23 };

const PRECINCT_INTERIOR_TRANSITIONS: [MapTransition; 1] = [MapTransition {
    from_map: MapId::PrecinctInterior,
    from_x: SOUTH_EXIT_POSITION.x,
    from_y: SOUTH_EXIT_POSITION.y,
    to_map: MapId::PrecinctExterior,
    to_x: 12,
    to_y: 1,
}];

const CAPTAIN_DOORS: [DoorSpec; 1] = [DoorSpec {
    side: DoorSide::South,
    offset: 4,
}];
const BREAK_ROOM_DOORS: [DoorSpec; 1] = [DoorSpec {
    side: DoorSide::South,
    offset: 4,
}];
const CASE_BOARD_DOORS: [DoorSpec; 1] = [DoorSpec {
    side: DoorSide::North,
    offset: 3,
}];
const LOBBY_DOORS: [DoorSpec; 2] = [
    DoorSpec {
        side: DoorSide::North,
        offset: 5,
    },
    DoorSpec {
        side: DoorSide::South,
        offset: 5,
    },
];
const EVIDENCE_ROOM_DOORS: [DoorSpec; 1] = [DoorSpec {
    side: DoorSide::North,
    offset: 3,
}];
const LOCKER_ROOM_DOORS: [DoorSpec; 1] = [DoorSpec {
    side: DoorSide::West,
    offset: 1,
}];

const PRECINCT_ROOMS: [RoomSpec; 6] = [
    RoomSpec {
        bounds: RoomBounds {
            min_x: 1,
            min_y: 1,
            max_x: 9,
            max_y: 8,
        },
        doors: &CAPTAIN_DOORS,
    },
    RoomSpec {
        bounds: RoomBounds {
            min_x: 22,
            min_y: 1,
            max_x: 30,
            max_y: 8,
        },
        doors: &BREAK_ROOM_DOORS,
    },
    RoomSpec {
        bounds: RoomBounds {
            min_x: 1,
            min_y: 15,
            max_x: 8,
            max_y: 21,
        },
        doors: &CASE_BOARD_DOORS,
    },
    RoomSpec {
        bounds: RoomBounds {
            min_x: 11,
            min_y: 15,
            max_x: 20,
            max_y: 22,
        },
        doors: &LOBBY_DOORS,
    },
    RoomSpec {
        bounds: RoomBounds {
            min_x: 23,
            min_y: 15,
            max_x: 30,
            max_y: 18,
        },
        doors: &EVIDENCE_ROOM_DOORS,
    },
    RoomSpec {
        bounds: RoomBounds {
            min_x: 23,
            min_y: 20,
            max_x: 30,
            max_y: 22,
        },
        doors: &LOCKER_ROOM_DOORS,
    },
];

const PRECINCT_INTERACTABLES: [GridPosition; 6] = [
    GridPosition { x: 7, y: 4 },   // captain's desk
    GridPosition { x: 24, y: 4 },  // break room kitchenette
    GridPosition { x: 3, y: 18 },  // case board
    GridPosition { x: 16, y: 18 }, // lobby desk
    GridPosition { x: 28, y: 16 }, // evidence shelf
    GridPosition { x: 27, y: 21 }, // locker bank
];

#[derive(Resource, Debug, Default)]
pub struct CollisionMap(pub HashSet<(i32, i32)>);

#[derive(Component, Debug)]
pub struct MapTile;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CollisionMap>()
            .add_systems(OnEnter(GameState::Playing), spawn_map)
            .add_systems(
                Update,
                handle_map_transition
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_map);
    }
}

#[derive(Clone, Copy)]
struct TileMapData {
    map_id: MapId,
    width: usize,
    height: usize,
    tiles: [[TileKind; PRECINCT_WIDTH]; PRECINCT_HEIGHT],
    transitions: &'static [MapTransition],
    spawn_point: GridPosition,
}

#[derive(Clone, Copy)]
struct RoomBounds {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
}

#[derive(Clone, Copy)]
struct RoomSpec {
    bounds: RoomBounds,
    doors: &'static [DoorSpec],
}

#[derive(Clone, Copy)]
struct DoorSpec {
    side: DoorSide,
    offset: i32,
}

#[derive(Clone, Copy)]
enum DoorSide {
    North,
    South,
    West,
}

pub fn precinct_interior_data() -> [[TileKind; PRECINCT_WIDTH]; PRECINCT_HEIGHT] {
    let mut tiles = [[TileKind::Floor; PRECINCT_WIDTH]; PRECINCT_HEIGHT];

    for tile in &mut tiles[0] {
        *tile = TileKind::Wall;
    }

    for tile in &mut tiles[PRECINCT_HEIGHT - 1] {
        *tile = TileKind::Wall;
    }

    for row in &mut tiles {
        row[0] = TileKind::Wall;
        row[PRECINCT_WIDTH - 1] = TileKind::Wall;
    }

    for room in PRECINCT_ROOMS {
        draw_room(&mut tiles, room);
    }

    for interactable in PRECINCT_INTERACTABLES {
        tiles[interactable.y as usize][interactable.x as usize] = TileKind::Interactable;
    }

    tiles[SOUTH_EXIT_POSITION.y as usize][SOUTH_EXIT_POSITION.x as usize] = TileKind::Door;
    tiles
}

pub fn spawn_map(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    mut collision_map: ResMut<CollisionMap>,
) {
    let requested_map = player_state.position_map;
    let resolved_map = resolve_supported_map(requested_map);
    let map_data = tile_map_data(resolved_map);

    if requested_map != resolved_map {
        player_state.position_map = resolved_map;
        set_player_grid_position(&mut player_state, map_data.spawn_point);
    } else if player_state.position_x == 0.0 && player_state.position_y == 0.0 {
        set_player_grid_position(&mut player_state, map_data.spawn_point);
    }

    spawn_map_entities(&mut commands, &mut collision_map, map_data);
}

pub fn handle_map_transition(
    mut commands: Commands,
    mut transition_events: EventReader<MapTransitionEvent>,
    mut player_state: ResMut<PlayerState>,
    mut collision_map: ResMut<CollisionMap>,
    map_tiles: Query<Entity, With<MapTile>>,
) {
    let mut latest_transition = None;
    for event in transition_events.read() {
        latest_transition = Some((event.from, event.to));
    }

    let Some((from_map, requested_map)) = latest_transition else {
        return;
    };

    despawn_map_tiles(&mut commands, &map_tiles, &mut collision_map);

    let resolved_map = resolve_supported_map(requested_map);
    let map_data = tile_map_data(resolved_map);
    let target_position = if resolved_map == requested_map {
        transition_target(from_map, requested_map).unwrap_or(map_data.spawn_point)
    } else {
        map_data.spawn_point
    };

    player_state.position_map = resolved_map;
    set_player_grid_position(&mut player_state, target_position);

    spawn_map_entities(&mut commands, &mut collision_map, map_data);
}

pub fn cleanup_map(
    mut commands: Commands,
    map_tiles: Query<Entity, With<MapTile>>,
    mut collision_map: ResMut<CollisionMap>,
) {
    despawn_map_tiles(&mut commands, &map_tiles, &mut collision_map);
}

fn tile_map_data(map_id: MapId) -> TileMapData {
    match map_id {
        MapId::PrecinctInterior => TileMapData {
            map_id,
            width: PRECINCT_WIDTH,
            height: PRECINCT_HEIGHT,
            tiles: precinct_interior_data(),
            transitions: &PRECINCT_INTERIOR_TRANSITIONS,
            spawn_point: PRECINCT_INTERIOR_SPAWN,
        },
        _ => unreachable!("unsupported map should be resolved before loading"),
    }
}

fn resolve_supported_map(requested_map: MapId) -> MapId {
    match requested_map {
        MapId::PrecinctInterior => requested_map,
        _ => {
            bevy::log::warn!(
                "Wave 1 world fallback: {:?} is not implemented yet; loading {:?} instead.",
                requested_map,
                MapId::PrecinctInterior
            );
            MapId::PrecinctInterior
        }
    }
}

fn spawn_map_entities(
    commands: &mut Commands,
    collision_map: &mut CollisionMap,
    map_data: TileMapData,
) {
    collision_map.0.clear();

    for y in 0..map_data.height {
        for x in 0..map_data.width {
            let tile_kind = map_data.tiles[y][x];
            let grid_position = GridPosition {
                x: x as i32,
                y: y as i32,
            };

            if tile_kind == TileKind::Wall {
                collision_map.0.insert((grid_position.x, grid_position.y));
            }

            commands.spawn((
                MapTile,
                grid_position,
                Sprite::from_color(tile_color(tile_kind), Vec2::splat(TILE_WORLD_SIZE)),
                Transform::from_xyz(
                    grid_position.x as f32 * TILE_WORLD_SIZE,
                    grid_position.y as f32 * TILE_WORLD_SIZE,
                    tile_z(tile_kind),
                ),
            ));
        }
    }

    debug_assert_eq!(
        map_data.transitions.len(),
        1,
        "Wave 1 should expose exactly one transition zone"
    );
    debug_assert_eq!(map_data.map_id, MapId::PrecinctInterior);
}

fn despawn_map_tiles(
    commands: &mut Commands,
    map_tiles: &Query<Entity, With<MapTile>>,
    collision_map: &mut CollisionMap,
) {
    for entity in map_tiles.iter() {
        commands.entity(entity).despawn();
    }
    collision_map.0.clear();
}

fn transition_target(from_map: MapId, to_map: MapId) -> Option<GridPosition> {
    for transition in PRECINCT_INTERIOR_TRANSITIONS {
        if transition.from_map == from_map && transition.to_map == to_map {
            return Some(GridPosition {
                x: transition.to_x,
                y: transition.to_y,
            });
        }

        if transition.from_map == to_map && transition.to_map == from_map {
            return Some(GridPosition {
                x: transition.from_x,
                y: transition.from_y,
            });
        }
    }

    None
}

fn draw_room(tiles: &mut [[TileKind; PRECINCT_WIDTH]; PRECINCT_HEIGHT], room: RoomSpec) {
    let bounds = room.bounds;

    for x in bounds.min_x..=bounds.max_x {
        tiles[bounds.min_y as usize][x as usize] = TileKind::Wall;
        tiles[bounds.max_y as usize][x as usize] = TileKind::Wall;
    }

    for y in bounds.min_y..=bounds.max_y {
        tiles[y as usize][bounds.min_x as usize] = TileKind::Wall;
        tiles[y as usize][bounds.max_x as usize] = TileKind::Wall;
    }

    for door in room.doors {
        let position = room_door_position(bounds, *door);
        tiles[position.y as usize][position.x as usize] = TileKind::Door;
    }
}

fn room_door_position(bounds: RoomBounds, door: DoorSpec) -> GridPosition {
    match door.side {
        DoorSide::North => GridPosition {
            x: bounds.min_x + door.offset,
            y: bounds.min_y,
        },
        DoorSide::South => GridPosition {
            x: bounds.min_x + door.offset,
            y: bounds.max_y,
        },
        DoorSide::West => GridPosition {
            x: bounds.min_x,
            y: bounds.min_y + door.offset,
        },
    }
}

fn set_player_grid_position(player_state: &mut PlayerState, position: GridPosition) {
    player_state.position_x = position.x as f32 * TILE_WORLD_SIZE;
    player_state.position_y = position.y as f32 * TILE_WORLD_SIZE;
}

fn tile_color(tile_kind: TileKind) -> Color {
    match tile_kind {
        TileKind::Floor => Color::srgb_u8(0x3a, 0x3a, 0x4a),
        TileKind::Wall => Color::srgb_u8(0x2a, 0x1a, 0x0a),
        TileKind::Door => Color::srgb_u8(0x5a, 0x3a, 0x1a),
        TileKind::Sidewalk => Color::srgb_u8(0x6a, 0x6a, 0x7a),
        TileKind::Road => Color::srgb_u8(0x4a, 0x4a, 0x4a),
        TileKind::Grass => Color::srgb_u8(0x2a, 0x4a, 0x2a),
        TileKind::Interactable => Color::srgb_u8(0x4a, 0x4a, 0x6a),
        TileKind::Water => Color::srgb_u8(0x2a, 0x4a, 0x6a),
        TileKind::CrimeTape => Color::srgb_u8(0xc9, 0xa2, 0x00),
    }
}

fn tile_z(tile_kind: TileKind) -> f32 {
    match tile_kind {
        TileKind::Floor
        | TileKind::Sidewalk
        | TileKind::Road
        | TileKind::Grass
        | TileKind::Water => 0.0,
        TileKind::Wall | TileKind::Door | TileKind::CrimeTape | TileKind::Interactable => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;
    use std::collections::HashSet;

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        );
        app.init_resource::<PlayerState>();
        app.add_event::<MapTransitionEvent>();
        app.add_plugins(WorldPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
    }

    fn map_tile_entities(app: &mut App) -> HashSet<Entity> {
        let mut query = app.world_mut().query_filtered::<Entity, With<MapTile>>();
        query.iter(app.world()).collect()
    }

    #[test]
    fn map_spawns_all_precinct_tiles() {
        let mut app = build_test_app();

        enter_playing(&mut app);

        let mut query = app.world_mut().query_filtered::<Entity, With<MapTile>>();
        assert_eq!(
            query.iter(app.world()).count(),
            PRECINCT_WIDTH * PRECINCT_HEIGHT
        );
    }

    #[test]
    fn collision_map_contains_every_wall_tile() {
        let mut app = build_test_app();

        enter_playing(&mut app);

        let map = precinct_interior_data();
        let expected_walls: HashSet<(i32, i32)> = map
            .iter()
            .enumerate()
            .flat_map(|(y, row)| {
                row.iter().enumerate().filter_map(move |(x, tile)| {
                    (tile == &TileKind::Wall).then_some((x as i32, y as i32))
                })
            })
            .collect();

        let collision_map = &app.world().resource::<CollisionMap>().0;
        assert_eq!(collision_map, &expected_walls);
    }

    #[test]
    fn collision_map_excludes_walkable_tiles() {
        let mut app = build_test_app();

        enter_playing(&mut app);

        let collision_map = &app.world().resource::<CollisionMap>().0;
        assert!(!collision_map.contains(&(PRECINCT_INTERIOR_SPAWN.x, PRECINCT_INTERIOR_SPAWN.y)));
        assert!(!collision_map.contains(&(SOUTH_EXIT_POSITION.x, SOUTH_EXIT_POSITION.y)));

        let map = precinct_interior_data();
        assert_eq!(
            map[PRECINCT_INTERIOR_SPAWN.y as usize][PRECINCT_INTERIOR_SPAWN.x as usize],
            TileKind::Floor
        );
        assert_eq!(
            map[SOUTH_EXIT_POSITION.y as usize][SOUTH_EXIT_POSITION.x as usize],
            TileKind::Door
        );
    }

    #[test]
    fn map_transition_replaces_existing_tile_entities() {
        let mut app = build_test_app();

        enter_playing(&mut app);
        let first_tile_set = map_tile_entities(&mut app);

        app.world_mut()
            .resource_mut::<Events<MapTransitionEvent>>()
            .send(MapTransitionEvent {
                from: MapId::PrecinctExterior,
                to: MapId::PrecinctInterior,
            });

        app.update();

        let second_tile_set = map_tile_entities(&mut app);
        assert_eq!(second_tile_set.len(), PRECINCT_WIDTH * PRECINCT_HEIGHT);
        assert!(first_tile_set.is_disjoint(&second_tile_set));

        let player_state = app.world().resource::<PlayerState>();
        assert_eq!(player_state.position_map, MapId::PrecinctInterior);
        assert_eq!(
            (player_state.position_x, player_state.position_y),
            (
                SOUTH_EXIT_POSITION.x as f32 * TILE_WORLD_SIZE,
                SOUTH_EXIT_POSITION.y as f32 * TILE_WORLD_SIZE,
            )
        );
    }

    #[test]
    fn precinct_spawn_point_is_walkable() {
        let map = precinct_interior_data();
        let spawn_tile =
            map[PRECINCT_INTERIOR_SPAWN.y as usize][PRECINCT_INTERIOR_SPAWN.x as usize];

        assert_ne!(spawn_tile, TileKind::Wall);
        assert_eq!(spawn_tile, TileKind::Floor);
    }
}
