//! NPC walk-cycle animation: cycles atlas frames based on movement direction.
//!
//! Uses the same 4×4 spritesheet layout as the player character:
//!   Row 0 (indices 0-3):  Walk down
//!   Row 1 (indices 4-7):  Walk left
//!   Row 2 (indices 8-11): Walk right
//!   Row 3 (indices 12-15): Walk up

use super::spawning::NpcMovement;
use crate::shared::*;
use bevy::prelude::*;

/// Timer component driving NPC walk-cycle frame advancement.
#[derive(Component, Debug, Clone)]
pub struct NpcAnimationTimer {
    pub timer: Timer,
    pub frame_count: usize,
    pub current_frame: usize,
    /// Last atlas row base (0/4/8/12) so idle NPCs keep their facing.
    pub last_base: usize,
}

/// System: animate NPC sprites based on movement direction and speed.
///
/// When `is_moving` is true, determines facing from the vector between
/// current position and target, then cycles through the 4 frames of the
/// matching atlas row. When idle, snaps to frame 0 of the current
/// direction.
pub fn animate_npc_sprites(
    time: Res<Time>,
    mut query: Query<
        (
            &NpcMovement,
            &LogicalPosition,
            &mut Sprite,
            &mut NpcAnimationTimer,
        ),
        With<Npc>,
    >,
) {
    for (movement, logical_pos, mut sprite, mut anim) in query.iter_mut() {
        // Determine facing from movement vector (current pos → target).
        // Only update facing when actually moving; idle NPCs keep last direction.
        let dx = movement.target_x - logical_pos.0.x;
        let dy = movement.target_y - logical_pos.0.y;

        let base: usize = if dx.abs() > 0.5 || dy.abs() > 0.5 {
            // Meaningful movement delta — update facing
            // Row 0=Down, Row 1=Left, Row 2=Right, Row 3=Up
            let new_base = if dx.abs() > dy.abs() {
                if dx > 0.0 { 8 } else { 4 }  // Right (row 2) : Left (row 1)
            } else if dy > 0.0 {
                12  // Up (row 3)
            } else {
                0   // Down (row 0)
            };
            anim.last_base = new_base;
            new_base
        } else {
            // At target or nearly there — keep last facing
            anim.last_base
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
