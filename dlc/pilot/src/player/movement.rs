//! Player movement — 8-dir with normalization, sprint/stamina, collision, animation states.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AnimState {
    #[default]
    Idle,
    WalkUp,
    WalkDown,
    WalkLeft,
    WalkRight,
    SprintUp,
    SprintDown,
    SprintLeft,
    SprintRight,
}

#[derive(Resource, Default)]
pub struct PlayerAnimState {
    pub state: AnimState,
    pub footstep_timer: f32,
}

const FOOTSTEP_INTERVAL: f32 = 0.35;
const SPRINT_FOOTSTEP_INTERVAL: f32 = 0.22;
const STAMINA_DRAIN_PER_SEC: f32 = 8.0;
const STAMINA_REGEN_PER_SEC: f32 = 3.0;
const TIRED_SPEED_MULT: f32 = 0.6;
const INDOOR_SPEED_MULT: f32 = 0.85;

#[allow(clippy::too_many_arguments)]
pub fn player_movement(
    time: Res<Time>,
    input: Res<PlayerInput>,
    collision_map: Res<CollisionMap>,
    mut player_q: Query<&mut Transform, With<Player>>,
    mut movement: ResMut<PlayerMovement>,
    mut grid_pos: ResMut<GridPosition>,
    mut pilot_state: ResMut<PilotState>,
    mut anim: ResMut<PlayerAnimState>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    player_location: Res<PlayerLocation>,
    world_map: Res<WorldMap>,
) {
    let Ok(mut transform) = player_q.get_single_mut() else { return; };
    let dt = time.delta_secs();

    let dir = input.movement.normalize_or_zero();
    movement.is_moving = dir.length_squared() > 0.0;

    if movement.is_moving {
        if dir.x.abs() > dir.y.abs() {
            movement.facing = if dir.x > 0.0 { Facing::Right } else { Facing::Left };
        } else {
            movement.facing = if dir.y > 0.0 { Facing::Up } else { Facing::Down };
        }
    }

    let wants_sprint = input.sprint && movement.is_moving;
    let can_sprint = pilot_state.stamina > 5.0;
    let sprinting = wants_sprint && can_sprint;

    if sprinting {
        pilot_state.stamina = (pilot_state.stamina - STAMINA_DRAIN_PER_SEC * dt).max(0.0);
    } else if !movement.is_moving {
        pilot_state.stamina = (pilot_state.stamina + STAMINA_REGEN_PER_SEC * dt).min(pilot_state.max_stamina);
    }

    let mut speed = if sprinting { PLAYER_SPEED * 1.6 } else { PLAYER_SPEED };
    if pilot_state.stamina < 15.0 { speed *= TIRED_SPEED_MULT; }
    if player_location.zone.is_indoor() { speed *= INDOOR_SPEED_MULT; }

    if movement.is_moving {
        anim.state = match (movement.facing, sprinting) {
            (Facing::Up, false) => AnimState::WalkUp,
            (Facing::Down, false) => AnimState::WalkDown,
            (Facing::Left, false) => AnimState::WalkLeft,
            (Facing::Right, false) => AnimState::WalkRight,
            (Facing::Up, true) => AnimState::SprintUp,
            (Facing::Down, true) => AnimState::SprintDown,
            (Facing::Left, true) => AnimState::SprintLeft,
            (Facing::Right, true) => AnimState::SprintRight,
        };
    } else {
        anim.state = AnimState::Idle;
    }

    let proposed = transform.translation.truncate() + dir * speed * dt;

    // Per-axis collision (slide along walls)
    let try_x = Vec2::new(proposed.x, transform.translation.y);
    let (gx_x, gy_x) = world_to_grid(try_x);
    let x_ok = !collision_map.is_blocked(gx_x, gy_x) && in_map_bounds(gx_x, gy_x, &world_map);

    let try_y = Vec2::new(transform.translation.x, proposed.y);
    let (gx_y, gy_y) = world_to_grid(try_y);
    let y_ok = !collision_map.is_blocked(gx_y, gy_y) && in_map_bounds(gx_y, gy_y, &world_map);

    if x_ok { transform.translation.x = proposed.x; }
    if y_ok { transform.translation.y = proposed.y; }

    let final_pos = transform.translation.truncate();
    let (gx, gy) = world_to_grid(final_pos);
    grid_pos.x = gx;
    grid_pos.y = gy;

    // Footstep SFX
    if movement.is_moving {
        let interval = if sprinting { SPRINT_FOOTSTEP_INTERVAL } else { FOOTSTEP_INTERVAL };
        anim.footstep_timer += dt;
        if anim.footstep_timer >= interval {
            anim.footstep_timer = 0.0;
            sfx_events.send(PlaySfxEvent { sfx_id: "footstep".to_string() });
        }
    } else {
        anim.footstep_timer = 0.0;
    }
}

fn in_map_bounds(gx: i32, gy: i32, world_map: &WorldMap) -> bool {
    if world_map.width == 0 { return true; }
    gx >= 0 && gy >= 0 && gx < world_map.width as i32 && gy < world_map.height as i32
}
