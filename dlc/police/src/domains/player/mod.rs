use crate::shared::{
    Equipment, Facing, FatigueChangeEvent, GameState, GridPosition, InputContext, MapId,
    MapTransitionEvent, Player, PlayerInput, PlayerMovement, PlayerState, StressChangeEvent,
    UpdatePhase, MAX_FATIGUE, MAX_STRESS, PIXEL_SCALE, TILE_SIZE,
};
use bevy::prelude::*;
use std::collections::HashSet;

const WALK_SPEED: f32 = 80.0;
const RUN_SPEED: f32 = WALK_SPEED * RUN_MULTIPLIER;
const RUN_MULTIPLIER: f32 = 1.5;
const CAMERA_LERP_SPEED: f32 = 8.0;
const PLAYER_Z: f32 = 10.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnExit(GameState::Playing), despawn_player)
            .add_systems(
                Update,
                read_keyboard_input
                    .in_set(UpdatePhase::Input)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (move_player, check_map_transition_zone.after(move_player))
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                apply_fatigue_stress
                    .in_set(UpdatePhase::Reactions)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                camera_follow
                    .in_set(UpdatePhase::Presentation)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Optional collision resource for tile-blocking checks.
///
/// Wave 1's world domain is expected to own and populate collision data later.
/// This resource is intentionally not initialized here so movement can skip
/// collision safely until that data exists.
#[derive(Resource, Debug, Clone, Default)]
pub struct CollisionMap {
    pub solid_tiles: HashSet<(i32, i32)>,
}

pub fn read_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    input_context: Res<InputContext>,
    mut player_input: ResMut<PlayerInput>,
) {
    *player_input = PlayerInput::default();

    player_input.menu = keyboard.just_pressed(KeyCode::Escape);
    player_input.cancel = player_input.menu;

    if input_context.in_dialogue || input_context.in_interrogation || input_context.in_menu {
        return;
    }

    player_input.move_dir = Vec2::new(
        axis_input(
            &keyboard,
            &[KeyCode::KeyD, KeyCode::ArrowRight],
            &[KeyCode::KeyA, KeyCode::ArrowLeft],
        ),
        axis_input(
            &keyboard,
            &[KeyCode::KeyW, KeyCode::ArrowUp],
            &[KeyCode::KeyS, KeyCode::ArrowDown],
        ),
    );
    player_input.interact = keyboard.just_pressed(KeyCode::KeyF);
    player_input.run =
        keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
}

pub fn spawn_player(mut commands: Commands, mut player_state: ResMut<PlayerState>) {
    let spawn_grid = GridPosition { x: 16, y: 20 };
    let spawn_position = grid_to_world(spawn_grid);

    if !player_state.equipped.contains(&Equipment::Badge) {
        player_state.equipped.push(Equipment::Badge);
    }

    player_state.position_map = MapId::PrecinctInterior;
    player_state.position_x = spawn_position.x;
    player_state.position_y = spawn_position.y;

    commands.spawn((
        Player,
        PlayerMovement {
            speed: WALK_SPEED,
            facing: Facing::Down,
            is_running: false,
        },
        spawn_grid,
        Sprite::from_color(Color::srgb(0.20, 0.38, 0.95), Vec2::splat(TILE_SIZE)),
        Transform::from_xyz(spawn_position.x, spawn_position.y, PLAYER_Z),
    ));
}

pub fn move_player(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    collision_map: Option<Res<CollisionMap>>,
    mut player_state: ResMut<PlayerState>,
    mut player_query: Query<(&mut Transform, &mut GridPosition, &mut PlayerMovement), With<Player>>,
) {
    let Ok((mut transform, mut grid_position, mut movement)) = player_query.get_single_mut() else {
        return;
    };

    let move_dir = player_input.move_dir.normalize_or_zero();
    movement.is_running = player_input.run && move_dir != Vec2::ZERO;
    movement.speed = if movement.is_running {
        RUN_SPEED
    } else {
        WALK_SPEED
    };

    if move_dir == Vec2::ZERO {
        player_state.position_x = transform.translation.x;
        player_state.position_y = transform.translation.y;
        return;
    }

    movement.facing = facing_from_direction(move_dir, movement.facing);

    let delta = move_dir * movement.speed * time.delta_secs();
    let mut new_position = transform.translation.truncate();
    let collision_map = collision_map.as_deref();

    if delta.x != 0.0 {
        let candidate = Vec2::new(new_position.x + delta.x, new_position.y);
        if !tile_is_blocked(collision_map, world_to_grid(candidate)) {
            new_position.x = candidate.x;
        }
    }

    if delta.y != 0.0 {
        let candidate = Vec2::new(new_position.x, new_position.y + delta.y);
        if !tile_is_blocked(collision_map, world_to_grid(candidate)) {
            new_position.y = candidate.y;
        }
    }

    transform.translation.x = new_position.x;
    transform.translation.y = new_position.y;

    let next_grid = world_to_grid(new_position);
    grid_position.x = next_grid.x;
    grid_position.y = next_grid.y;
    player_state.position_x = new_position.x;
    player_state.position_y = new_position.y;
}

pub fn camera_follow(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    let target = player_transform.translation;
    let t = (CAMERA_LERP_SPEED * time.delta_secs()).min(1.0);

    camera_transform.translation.x = lerp(camera_transform.translation.x, target.x, t).round();
    camera_transform.translation.y = lerp(camera_transform.translation.y, target.y, t).round();
}

pub fn check_map_transition_zone(
    player_state: Res<PlayerState>,
    player_query: Query<&GridPosition, With<Player>>,
    mut transition_events: EventWriter<MapTransitionEvent>,
    mut was_in_transition_zone: Local<bool>,
) {
    let Ok(grid_position) = player_query.get_single() else {
        return;
    };

    let in_transition_zone = player_state.position_map == MapId::PrecinctInterior
        && grid_position.x == 16
        && grid_position.y <= 0;

    if in_transition_zone && !*was_in_transition_zone {
        transition_events.send(MapTransitionEvent {
            from: MapId::PrecinctInterior,
            to: MapId::PrecinctExterior,
        });
    }

    *was_in_transition_zone = in_transition_zone;
}

pub fn apply_fatigue_stress(
    mut fatigue_events: EventReader<FatigueChangeEvent>,
    mut stress_events: EventReader<StressChangeEvent>,
    mut player_state: ResMut<PlayerState>,
) {
    for event in fatigue_events.read() {
        player_state.fatigue = (player_state.fatigue + event.delta).clamp(0.0, MAX_FATIGUE);
    }

    for event in stress_events.read() {
        player_state.stress = (player_state.stress + event.delta).clamp(0.0, MAX_STRESS);
    }
}

pub fn despawn_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    for entity in &player_query {
        commands.entity(entity).despawn();
    }
}

fn axis_input(
    keyboard: &ButtonInput<KeyCode>,
    positive_keys: &[KeyCode],
    negative_keys: &[KeyCode],
) -> f32 {
    let positive = positive_keys.iter().any(|key| keyboard.pressed(*key)) as i8;
    let negative = negative_keys.iter().any(|key| keyboard.pressed(*key)) as i8;

    (positive - negative) as f32
}

fn facing_from_direction(direction: Vec2, current: Facing) -> Facing {
    if direction.x.abs() > direction.y.abs() {
        if direction.x > 0.0 {
            Facing::Right
        } else {
            Facing::Left
        }
    } else if direction.y.abs() > 0.0 {
        if direction.y > 0.0 {
            Facing::Up
        } else {
            Facing::Down
        }
    } else {
        current
    }
}

fn grid_to_world(grid_position: GridPosition) -> Vec2 {
    Vec2::new(
        grid_position.x as f32 * world_tile_size(),
        grid_position.y as f32 * world_tile_size(),
    )
}

fn world_to_grid(world_position: Vec2) -> GridPosition {
    GridPosition {
        x: (world_position.x / world_tile_size()).round() as i32,
        y: (world_position.y / world_tile_size()).round() as i32,
    }
}

fn tile_is_blocked(collision_map: Option<&CollisionMap>, grid_position: GridPosition) -> bool {
    collision_map.is_some_and(|map| {
        map.solid_tiles
            .contains(&(grid_position.x, grid_position.y))
    })
}

fn world_tile_size() -> f32 {
    TILE_SIZE * PIXEL_SCALE
}

fn lerp(current: f32, target: f32, factor: f32) -> f32 {
    current + (target - current) * factor
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0,
        )));
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
        app.init_resource::<PlayerState>()
            .init_resource::<PlayerInput>()
            .init_resource::<InputContext>()
            .insert_resource(ButtonInput::<KeyCode>::default())
            .add_event::<FatigueChangeEvent>()
            .add_event::<StressChangeEvent>()
            .add_event::<MapTransitionEvent>()
            .add_plugins(PlayerPlugin);

        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
    }

    #[test]
    fn player_spawns_at_precinct_entry_point() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let mut query = app
            .world_mut()
            .query::<(&GridPosition, &Transform, &Sprite, &PlayerMovement)>();
        let (grid_position, transform, sprite, movement) = query.single(app.world());

        assert_eq!(grid_position.x, 16);
        assert_eq!(grid_position.y, 20);
        assert_eq!(transform.translation.x, 16.0 * 16.0 * 3.0);
        assert_eq!(transform.translation.y, 20.0 * 16.0 * 3.0);
        assert_eq!(sprite.custom_size, Some(Vec2::splat(TILE_SIZE)));
        assert_eq!(movement.speed, WALK_SPEED);
        assert_eq!(movement.facing, Facing::Down);
        assert_eq!(
            app.world().resource::<PlayerState>().position_map,
            MapId::PrecinctInterior
        );
    }

    #[test]
    fn keyboard_input_sets_move_interact_menu_and_run() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.press(KeyCode::KeyW);
            keyboard.press(KeyCode::ArrowRight);
            keyboard.press(KeyCode::ShiftLeft);
            keyboard.press(KeyCode::KeyF);
            keyboard.press(KeyCode::Escape);
        }

        app.update();

        let input = app.world().resource::<PlayerInput>();
        assert_eq!(input.move_dir, Vec2::new(1.0, 1.0));
        assert!(input.interact);
        assert!(input.menu);
        assert!(input.cancel);
        assert!(input.run);
    }

    #[test]
    fn movement_updates_transform_and_facing() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        app.update();

        let delta_secs = app.world().resource::<Time>().delta_secs();
        let expected_x = (16.0 * 16.0 * 3.0) + (WALK_SPEED * delta_secs);
        let expected_grid = world_to_grid(Vec2::new(expected_x, 20.0 * 16.0 * 3.0));

        let mut query = app
            .world_mut()
            .query::<(&GridPosition, &Transform, &PlayerMovement)>();
        let (grid_position, transform, movement) = query.single(app.world());

        assert_eq!(transform.translation.x, expected_x);
        assert_eq!(transform.translation.y, 20.0 * 16.0 * 3.0);
        assert_eq!(grid_position.x, expected_grid.x);
        assert_eq!(grid_position.y, expected_grid.y);
        assert_eq!(movement.facing, Facing::Right);
        assert!(!movement.is_running);
    }

    #[test]
    fn movement_does_not_enter_solid_tiles() {
        let mut app = build_test_app();
        app.insert_resource(CollisionMap {
            solid_tiles: HashSet::from([(16, 20), (17, 20), (18, 20)]),
        });
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&GridPosition, &Transform, &PlayerMovement)>();
        let (grid_position, transform, movement) = query.single(app.world());

        assert_eq!(grid_position.x, 16);
        assert_eq!(grid_position.y, 20);
        assert_eq!(transform.translation.x, 16.0 * 16.0 * 3.0);
        assert_eq!(movement.facing, Facing::Right);
    }

    #[test]
    fn fatigue_changes_apply_and_clamp() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        app.world_mut().resource_mut::<PlayerState>().fatigue = 90.0;

        app.world_mut()
            .send_event(FatigueChangeEvent { delta: 20.0 });
        app.update();
        assert_eq!(app.world().resource::<PlayerState>().fatigue, MAX_FATIGUE);

        app.world_mut()
            .send_event(FatigueChangeEvent { delta: -250.0 });
        app.update();
        assert_eq!(app.world().resource::<PlayerState>().fatigue, 0.0);
    }

    #[test]
    fn stress_changes_apply_and_clamp() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        app.world_mut().resource_mut::<PlayerState>().stress = 95.0;

        app.world_mut()
            .send_event(StressChangeEvent { delta: 20.0 });
        app.update();
        assert_eq!(app.world().resource::<PlayerState>().stress, MAX_STRESS);

        app.world_mut()
            .send_event(StressChangeEvent { delta: -250.0 });
        app.update();
        assert_eq!(app.world().resource::<PlayerState>().stress, 0.0);
    }

    #[test]
    fn player_despawns_on_exit_playing() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Paused);
        app.update();

        let mut query = app.world_mut().query_filtered::<Entity, With<Player>>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    #[test]
    fn running_uses_faster_speed() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut keyboard = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keyboard.press(KeyCode::KeyD);
            keyboard.press(KeyCode::ShiftLeft);
        }

        app.update();

        let delta_secs = app.world().resource::<Time>().delta_secs();
        let expected_x = (16.0 * 16.0 * 3.0) + (RUN_SPEED * delta_secs);

        let mut query = app.world_mut().query::<(&Transform, &PlayerMovement)>();
        let (transform, movement) = query.single(app.world());

        assert_eq!(transform.translation.x, expected_x);
        assert_eq!(movement.speed, RUN_SPEED);
        assert!(movement.is_running);
    }
}
