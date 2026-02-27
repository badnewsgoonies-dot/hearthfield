//! NPC walk-cycle animation: cycles atlas frames based on movement direction.
//!
//! Uses the same 4×4 spritesheet layout as the player character:
//!   Row 0 (indices 0-3):  Walk down
//!   Row 1 (indices 4-7):  Walk up
//!   Row 2 (indices 8-11): Walk right
//!   Row 3 (indices 12-15): Walk left

use bevy::prelude::*;
use crate::shared::*;
use super::spawning::NpcMovement;

/// Timer component driving NPC walk-cycle frame advancement.
#[derive(Component, Debug, Clone)]
pub struct NpcAnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
}

/// System: animate NPC sprites based on movement direction and speed.
///
/// When `is_moving` is true, determines facing from the vector between
/// current position and target, then cycles through the 4 frames of the
/// matching atlas row. When idle, snaps to frame 0 of the current
/// direction.
pub fn animate_npc_sprites(
    time: Res<Time>,
    mut query: Query<(&NpcMovement, &LogicalPosition, &mut Sprite, &mut NpcAnimationTimer), With<Npc>>,
) {
    for (movement, logical_pos, mut sprite, mut anim) in query.iter_mut() {
        // Determine facing from movement vector (current pos → target)
        let dx = movement.target_x - logical_pos.0.x;
        let dy = movement.target_y - logical_pos.0.y;

        let base: usize = if dx.abs() > dy.abs() {
            if dx > 0.0 { 8 } else { 12 } // Right : Left
        } else {
            if dy > 0.0 { 4 } else { 0 } // Up : Down
        };

        if movement.is_moving {
            anim.timer.tick(time.delta());
            if anim.timer.just_finished() {
                anim.current_frame = (anim.current_frame + 1) % anim.frame_count;
            }
        } else {
            anim.current_frame = 0;
        }

        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = base + anim.current_frame;
        }
    }
}
