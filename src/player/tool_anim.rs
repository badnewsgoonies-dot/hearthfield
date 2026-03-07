use super::{facing_offset, ActionSpriteData, PlayerSpriteData};
use crate::shared::*;
use bevy::prelude::*;

// Action atlas base indices (2 cols × 12 rows, indexed left-to-right, top-to-bottom)
// Each tool gets 4 frames (2 frames × 2 rows per tool direction).
// Layout: 6 tools × 4 frames = 24 total
const ACTION_HOE_BASE: usize = 0;
const ACTION_WATER_BASE: usize = 4;
const ACTION_AXE_BASE: usize = 8;
const ACTION_PICK_BASE: usize = 12;
const ACTION_FISH_BASE: usize = 16;
const ACTION_SCYTHE_BASE: usize = 20;

/// Plays a sound effect when a tool impact occurs.
pub fn handle_tool_impact_sfx(
    mut impact_events: EventReader<ToolImpactEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for event in impact_events.read() {
        let sfx_id = match event.tool {
            ToolKind::Axe => "axe_chop",
            ToolKind::Pickaxe => "pickaxe_hit",
            ToolKind::Hoe => "hoe_till",
            ToolKind::WateringCan => "water_splash",
            ToolKind::FishingRod => "fishing_cast",
            _ => "tool_generic",
        };
        sfx_writer.send(PlaySfxEvent {
            sfx_id: sfx_id.to_string(),
        });
    }
}

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

/// Per-tool frame duration in seconds. Heavy tools feel weighty,
/// light tools feel snappy. Total animation = duration × 4 frames.
fn tool_frame_duration(tool: ToolKind) -> f32 {
    match tool {
        ToolKind::Axe => 0.15,         // 0.60s total — heavy, impactful chop
        ToolKind::Pickaxe => 0.14,     // 0.56s total — heavy swing
        ToolKind::Hoe => 0.12,         // 0.48s total — deliberate tilling
        ToolKind::FishingRod => 0.11,  // 0.44s total — quick cast flick
        ToolKind::WateringCan => 0.10, // 0.40s total — smooth pour
        ToolKind::Scythe => 0.08,      // 0.32s total — fast sweep
    }
}

/// System: when PlayerAnimState is ToolUse, swap to action atlas and
/// advance frames on a timer. Each tool has a distinct frame duration
/// so heavy tools feel weighty and light tools feel snappy.
pub fn animate_tool_use(
    time: Res<Time>,
    action_sprites: Option<Res<ActionSpriteData>>,
    walk_sprites: Res<PlayerSpriteData>,
    mut query: Query<(Entity, &mut PlayerMovement, &mut Sprite, &LogicalPosition), With<Player>>,
    mut impact_events: EventWriter<ToolImpactEvent>,
    mut frame_timer: Local<f32>,
    mut impact_fired: Local<bool>,
) {
    let Some(action_data) = action_sprites else {
        return;
    };
    if !action_data.loaded {
        return;
    }

    for (entity, mut movement, mut sprite, logical_pos) in query.iter_mut() {
        if let PlayerAnimState::ToolUse {
            tool,
            frame,
            total_frames,
        } = movement.anim_state
        {
            let duration = tool_frame_duration(tool);

            if frame == 0 && *frame_timer == 0.0 {
                // First frame: swap atlas to action sheet, reset impact flag
                sprite.image = action_data.image.clone();
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.layout = action_data.layout.clone();
                    atlas.index = action_atlas_index(tool, 0);
                }
                // Mirror sprite when facing left for directional tool animations
                sprite.flip_x = movement.facing == Facing::Left;
                *impact_fired = false;
            }

            // Accumulate time
            *frame_timer += time.delta_secs();

            // Emit impact event on frame 2 (once)
            if frame >= 2 && !*impact_fired {
                *impact_fired = true;
                let g = world_to_grid(logical_pos.0.x, logical_pos.0.y);
                let (px, py) = (g.x, g.y);
                let (dx, dy) = facing_offset(&movement.facing);
                impact_events.send(ToolImpactEvent {
                    tool,
                    grid_x: px + dx,
                    grid_y: py + dy,
                    player: entity,
                });
            }

            // Check if enough time has passed to advance frame
            if *frame_timer >= duration {
                *frame_timer -= duration;
                let new_frame = frame + 1;

                if new_frame >= total_frames {
                    // Animation complete — swap back to walk atlas
                    sprite.image = walk_sprites.image.clone();
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.layout = walk_sprites.layout.clone();
                        atlas.index = 0;
                    }
                    // Reset flip since walk animation handles its own flipping
                    sprite.flip_x = false;
                    // Avoid 1-frame idle flicker: if player is moving, go straight to Walk
                    movement.anim_state = if movement.is_moving {
                        PlayerAnimState::Walk
                    } else {
                        PlayerAnimState::Idle
                    };
                    *frame_timer = 0.0;
                    *impact_fired = false;
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
        } else {
            // Not in tool animation — reset timer
            *frame_timer = 0.0;
            *impact_fired = false;
        }
    }
}
