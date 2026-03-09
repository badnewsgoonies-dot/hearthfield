use super::facing_offset;
use crate::shared::*;
use bevy::prelude::*;

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

/// System: when PlayerAnimState is ToolUse, cycle through the walk frames
/// in the current facing direction to create a "bob" effect.
/// Each tool has a distinct frame duration so heavy tools feel weighty.
pub fn animate_tool_use(
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerMovement, &mut Sprite, &LogicalPosition), With<Player>>,
    mut impact_events: EventWriter<ToolImpactEvent>,
    mut frame_timer: Local<f32>,
    mut impact_fired: Local<bool>,
) {
    for (entity, mut movement, mut sprite, logical_pos) in query.iter_mut() {
        if let PlayerAnimState::ToolUse {
            tool,
            frame,
            total_frames,
        } = movement.anim_state
        {
            let duration = tool_frame_duration(tool);

            // Facing-direction base index in the walk atlas
            let facing_base: usize = match movement.facing {
                Facing::Down => 0,
                Facing::Up => 4,
                Facing::Left => 8,
                Facing::Right => 12,
            };

            if frame == 0 && *frame_timer == 0.0 {
                // First frame: set the bob start index only.
                // Do NOT overwrite sprite.image — spawn.rs already set the correct
                // spritesheet handle. Overwriting with walk_sprites.image risks using a
                // stale or default handle, causing the sprite to vanish.
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = facing_base + 1;
                }
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
                    // Animation complete — return to idle pose
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        atlas.index = facing_base;
                    }
                    movement.anim_state = if movement.is_moving {
                        PlayerAnimState::Walk
                    } else {
                        PlayerAnimState::Idle
                    };
                    *frame_timer = 0.0;
                    *impact_fired = false;
                } else {
                    // Bob through walk frames: cycle 1 → 2 → 3 → 0
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        let walk_frame = (new_frame as usize + 1) % 4;
                        atlas.index = facing_base + walk_frame;
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

// ═══════════════════════════════════════════════════════════════════════════
// Tile cursor: highlights the tile the equipped tool will act on
// ═══════════════════════════════════════════════════════════════════════════

/// Marker component for the tile cursor entity.
#[derive(Component)]
pub struct ToolTileCursor;

/// Spawn the tile cursor entity once on entering Playing state.
pub fn spawn_tool_cursor(mut commands: Commands) {
    commands.spawn((
        ToolTileCursor,
        Sprite {
            color: Color::srgba(1.0, 1.0, 0.6, 0.35),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            anchor: bevy::sprite::Anchor::BottomLeft,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, Z_FARM_OVERLAY + 1.0),
        Visibility::default(),
    ));
}

/// Each frame, move the cursor to the tile the player is facing.
pub fn update_tool_cursor(
    player_query: Query<(&LogicalPosition, &PlayerMovement), With<Player>>,
    mut cursor_query: Query<(&mut Transform, &mut Visibility), With<ToolTileCursor>>,
    input_blocks: Res<InputBlocks>,
) {
    let Ok((pos, movement)) = player_query.get_single() else {
        return;
    };
    let Ok((mut cursor_tf, mut cursor_vis)) = cursor_query.get_single_mut() else {
        return;
    };

    // Hide cursor when input is blocked (menus, dialogue, etc.)
    if input_blocks.is_blocked() {
        *cursor_vis = Visibility::Hidden;
        return;
    }

    *cursor_vis = Visibility::Inherited;

    let g = world_to_grid(pos.0.x, pos.0.y);
    let (dx, dy) = facing_offset(&movement.facing);
    let target_x = g.x + dx;
    let target_y = g.y + dy;

    // Position cursor at the target tile's bottom-left corner
    cursor_tf.translation.x = target_x as f32 * TILE_SIZE;
    cursor_tf.translation.y = target_y as f32 * TILE_SIZE;
}
