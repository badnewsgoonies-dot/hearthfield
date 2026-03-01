use bevy::prelude::*;
use crate::shared::*;
use super::{CollisionMap, DistanceAnimator};

/// Core movement system — reads input, applies velocity to LogicalPosition,
/// updates facing direction, snaps grid position, and checks collisions.
pub fn player_movement(
    time: Res<Time>,
    player_input: Res<PlayerInput>,
    collision_map: Res<CollisionMap>,
    farm_state: Res<FarmState>,
    player_state: Res<PlayerState>,
    input_blocks: Res<InputBlocks>,
    mut query: Query<(&mut LogicalPosition, &mut PlayerMovement, &mut GridPosition), With<Player>>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    let Ok((mut logical_pos, mut movement, mut grid_pos)) = query.get_single_mut() else {
        return;
    };

    // Tick cooldown
    movement.move_cooldown.tick(time.delta());

    // Determine desired direction from input
    let dir = player_input.move_axis;

    if dir != Vec2::ZERO {
        movement.is_moving = true;

        if dir.y.abs() >= dir.x.abs() {
            movement.facing = if dir.y > 0.0 { Facing::Up } else { Facing::Down };
        } else {
            movement.facing = if dir.x > 0.0 { Facing::Right } else { Facing::Left };
        }

        let normalized = dir.normalize();
        let delta = normalized * movement.speed * time.delta_secs();

        let candidate_x = logical_pos.0.x + delta.x;
        let candidate_y = logical_pos.0.y + delta.y;

        let can_move_x = !is_blocked(candidate_x, logical_pos.0.y, &collision_map, &farm_state, &player_state);
        let can_move_y = !is_blocked(logical_pos.0.x, candidate_y, &collision_map, &farm_state, &player_state);

        if can_move_x {
            logical_pos.0.x = candidate_x;
        }
        if can_move_y {
            logical_pos.0.y = candidate_y;
        }

        let g = world_to_grid(logical_pos.0.x, logical_pos.0.y);
        grid_pos.x = g.x;
        grid_pos.y = g.y;
    } else {
        movement.is_moving = false;
    }

    // Update animation state — preserve ToolUse regardless of movement
    movement.anim_state = match movement.anim_state {
        PlayerAnimState::ToolUse { .. } => movement.anim_state,
        _ if movement.is_moving => PlayerAnimState::Walk,
        _ => PlayerAnimState::Idle,
    };
}

/// Drive the walk-cycle animation using distance-based frame advance.
pub fn animate_player_sprite(
    mut query: Query<(
        &LogicalPosition,
        &PlayerMovement,
        &mut Sprite,
        &mut DistanceAnimator,
    ), With<Player>>,
) {
    for (pos, movement, mut sprite, mut anim) in query.iter_mut() {
        let base: usize = match movement.facing {
            Facing::Down  =>  0,
            Facing::Up    =>  4,
            Facing::Left  =>  8,
            Facing::Right => 12,
        };

        match movement.anim_state {
            PlayerAnimState::Walk => {
                let delta = pos.0 - anim.last_pos;
                let dist = delta.length();

                if dist > 0.0 {
                    anim.distance_budget += dist;
                    anim.last_pos = pos.0;

                    while anim.distance_budget >= anim.pixels_per_frame {
                        anim.distance_budget -= anim.pixels_per_frame;
                        anim.current_frame = (anim.current_frame + 1) % anim.frames_per_row;
                    }
                }

                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = base + anim.current_frame;
                }
            }

            PlayerAnimState::Idle => {
                anim.current_frame = 0;
                anim.distance_budget = 0.0;
                anim.last_pos = pos.0;

                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = base;
                }
            }

            PlayerAnimState::ToolUse { .. } => {
                // Part E handles tool animation frames — skip walk logic
            }
        }
    }
}

/// Check whether a world position is blocked.
fn is_blocked(
    wx: f32,
    wy: f32,
    collision_map: &CollisionMap,
    farm_state: &FarmState,
    player_state: &PlayerState,
) -> bool {
    let g = world_to_grid(wx, wy);
    let (gx, gy) = (g.x, g.y);

    if collision_map.initialised && collision_map.solid_tiles.contains(&(gx, gy)) {
        return true;
    }

    if collision_map.initialised {
        let (min_x, max_x, min_y, max_y) = collision_map.bounds;
        if gx < min_x || gx > max_x || gy < min_y || gy > max_y {
            return true;
        }
    }

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
