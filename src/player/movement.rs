use bevy::prelude::*;
use crate::shared::*;
use super::{CollisionMap, world_to_grid};

/// Core movement system — reads WASD / arrow keys, applies velocity,
/// updates facing direction, snaps grid position, and checks collisions.
///
/// Movement is continuous (smooth pixel motion at `speed` px/s) but the
/// `GridPosition` component is always kept in sync for tile lookups.
pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    collision_map: Res<CollisionMap>,
    farm_state: Res<FarmState>,
    player_state: Res<PlayerState>,
    mut query: Query<(&mut Transform, &mut PlayerMovement, &mut GridPosition), With<Player>>,
) {
    let Ok((mut transform, mut movement, mut grid_pos)) = query.get_single_mut() else {
        return;
    };

    // Tick cooldown
    movement.move_cooldown.tick(time.delta());

    // Determine desired direction from input
    let mut dir = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }

    // Update facing — prioritise vertical if both pressed, since that feels
    // more natural for a top-down farming game (approaching plots).
    if dir != Vec2::ZERO {
        movement.is_moving = true;

        // Determine primary facing direction. When moving diagonally, pick the
        // axis with the most recently changed input. For simplicity we pick the
        // axis with larger absolute magnitude (they are equal for diagonals, so
        // we bias towards vertical in that case).
        if dir.y.abs() >= dir.x.abs() {
            movement.facing = if dir.y > 0.0 { Facing::Up } else { Facing::Down };
        } else {
            movement.facing = if dir.x > 0.0 { Facing::Right } else { Facing::Left };
        }

        // Normalise so diagonal speed equals cardinal speed.
        let normalized = dir.normalize();
        let delta = normalized * movement.speed * time.delta_secs();

        // Candidate new position
        let candidate_x = transform.translation.x + delta.x;
        let candidate_y = transform.translation.y + delta.y;

        // Collision check — test the grid tile the player would enter.
        // We do axis-separated collision so the player can slide along walls.
        let can_move_x = !is_blocked(candidate_x, transform.translation.y, &collision_map, &farm_state, &player_state);
        let can_move_y = !is_blocked(transform.translation.x, candidate_y, &collision_map, &farm_state, &player_state);

        if can_move_x {
            transform.translation.x = candidate_x;
        }
        if can_move_y {
            transform.translation.y = candidate_y;
        }

        // Update grid position from the (possibly clamped) world position.
        let (gx, gy) = world_to_grid(transform.translation.x, transform.translation.y);
        grid_pos.x = gx;
        grid_pos.y = gy;
    } else {
        movement.is_moving = false;
    }
}

/// Check whether a world position is blocked by a solid tile, farm object,
/// or out-of-bounds in the collision map.
fn is_blocked(
    wx: f32,
    wy: f32,
    collision_map: &CollisionMap,
    farm_state: &FarmState,
    player_state: &PlayerState,
) -> bool {
    let (gx, gy) = world_to_grid(wx, wy);

    // 1. Explicit collision map tiles (populated by the world domain).
    if collision_map.initialised && collision_map.solid_tiles.contains(&(gx, gy)) {
        return true;
    }

    // 2. Map boundary check — prevent walking off the map.
    if collision_map.initialised {
        let (min_x, max_x, min_y, max_y) = collision_map.bounds;
        if gx < min_x || gx > max_x || gy < min_y || gy > max_y {
            return true;
        }
    }

    // 3. Farm objects — trees, rocks, stumps are solid.
    if player_state.current_map == MapId::Farm {
        if let Some(obj) = farm_state.objects.get(&(gx, gy)) {
            match obj {
                FarmObject::Tree { .. }
                | FarmObject::Rock { .. }
                | FarmObject::Stump { .. }
                | FarmObject::Fence => return true,
                _ => {}
            }
        }
    }

    false
}
