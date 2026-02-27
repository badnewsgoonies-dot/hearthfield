use bevy::prelude::*;
use crate::shared::*;
use super::{ActionSpriteData, PlayerSpriteData, DistanceAnimator};

// Action atlas base indices (2 cols × 12 rows, indexed left-to-right, top-to-bottom)
// Each tool gets 4 frames (2 frames × 2 rows per tool direction).
// Layout: 6 tools × 4 frames = 24 total
const ACTION_HOE_BASE: usize = 0;
const ACTION_WATER_BASE: usize = 4;
const ACTION_AXE_BASE: usize = 8;
const ACTION_PICK_BASE: usize = 12;
const ACTION_FISH_BASE: usize = 16;
const ACTION_SCYTHE_BASE: usize = 20;

/// Map (tool, frame) → atlas index in character_actions.png.
fn action_atlas_index(tool: ToolKind, frame: usize) -> usize {
    let tool_offset = match tool {
        ToolKind::Hoe => ACTION_HOE_BASE,
        ToolKind::WateringCan => ACTION_WATER_BASE,
        ToolKind::Axe => ACTION_AXE_BASE,
        ToolKind::Pickaxe => ACTION_PICK_BASE,
        ToolKind::FishingRod => ACTION_FISH_BASE,
        ToolKind::Scythe => ACTION_SCYTHE_BASE,
    };
    tool_offset + frame
}

/// System: when PlayerAnimState is ToolUse, swap to action atlas and
/// advance frames each tick. When animation completes, swap back to
/// walk atlas and reset state to Idle.
///
/// Each frame of the tool animation advances once per system tick.
/// At 60 FPS this yields ~0.067s per frame, 4 frames ≈ 0.27s total.
pub fn animate_tool_use(
    action_sprites: Option<Res<ActionSpriteData>>,
    walk_sprites: Res<PlayerSpriteData>,
    mut query: Query<(
        Entity,
        &mut PlayerMovement,
        &mut Sprite,
        &mut DistanceAnimator,
        &LogicalPosition,
    ), With<Player>>,
    mut impact_events: EventWriter<ToolImpactEvent>,
) {
    let Some(action_data) = action_sprites else { return };
    if !action_data.loaded { return; }

    for (entity, mut movement, mut sprite, _dist_anim, logical_pos) in query.iter_mut() {
        match movement.anim_state {
            PlayerAnimState::ToolUse { tool, frame, total_frames } => {
                if frame == 0 {
                    // First frame: swap atlas to action sheet
                    sprite.image = action_data.image.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = action_data.layout.clone();
                        atlas.index = action_atlas_index(tool, 0);
                    }
                }

                // Emit impact event on frame 2
                if frame == 2 {
                    let grid_x = (logical_pos.0.x / TILE_SIZE).floor() as i32;
                    let grid_y = (logical_pos.0.y / TILE_SIZE).floor() as i32;
                    impact_events.send(ToolImpactEvent {
                        tool,
                        grid_x,
                        grid_y,
                        player: entity,
                    });
                }

                let new_frame = frame + 1;

                if new_frame >= total_frames {
                    // Animation complete — swap back to walk atlas
                    sprite.image = walk_sprites.image.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = walk_sprites.layout.clone();
                        atlas.index = 0;
                    }
                    movement.anim_state = PlayerAnimState::Idle;
                } else {
                    // Advance frame
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.index = action_atlas_index(tool, new_frame as usize);
                    }
                    movement.anim_state = PlayerAnimState::ToolUse {
                        tool,
                        frame: new_frame,
                        total_frames,
                    };
                }
            }
            _ => {}
        }
    }
}
