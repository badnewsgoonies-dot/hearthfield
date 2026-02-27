use bevy::prelude::*;
use crate::shared::*;
use super::{AnimationTimer, CollisionMap, world_to_grid};

/// Core movement system — reads WASD / arrow keys, applies velocity,
/// updates facing direction, snaps grid position, and checks collisions.
///
/// Movement is continuous (smooth pixel motion at `speed` px/s) but the
/// `GridPosition` component is always kept in sync for tile lookups.
pub fn player_movement(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    collision_map: Res<CollisionMap>,
    farm_state: Res<FarmState>,
    player_state: Res<PlayerState>,
    input_blocks: Res<InputBlocks>,
    mut query: Query<(&mut Transform, &mut PlayerMovement, &mut GridPosition), With<Player>>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    let Ok((mut transform, mut movement, mut grid_pos)) = query.get_single_mut() else {
        return;
    };

    // Tick cooldown
    movement.move_cooldown.tick(time.delta());

    // Determine desired direction from input
    let dir = player_input.move_axis;

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

/// Drive the walk-cycle animation on the player sprite.
///
/// - When `is_moving` is true the timer ticks and advances through the four
///   frames of the current facing row in the atlas.
/// - When `is_moving` is false the frame resets to the first frame of the row
///   (the idle/rest pose for that direction).
///
/// Atlas layout (character_spritesheet.png, 4×4 grid of 48×48 frames):
///   Row 0 → Walk Down  (base index  0)
///   Row 1 → Walk Up    (base index  4)
///   Row 2 → Walk Right (base index  8)
///   Row 3 → Walk Left  (base index 12)
pub fn animate_player_sprite(
    time: Res<Time>,
    mut query: Query<(&PlayerMovement, &mut Sprite, &mut AnimationTimer), With<Player>>,
) {
    for (movement, mut sprite, mut anim) in query.iter_mut() {
        // Map the current facing to the first atlas index for that row.
        let base: usize = match movement.facing {
            Facing::Down  =>  0,
            Facing::Up    =>  4,
            Facing::Right =>  8,
            Facing::Left  => 12,
        };

        if movement.is_moving {
            anim.timer.tick(time.delta());
            if anim.timer.just_finished() {
                anim.current_frame = (anim.current_frame + 1) % anim.frame_count;
            }
        } else {
            // Snap back to the idle (first) frame of the current direction.
            anim.current_frame = 0;
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = base + anim.current_frame;
        }
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
