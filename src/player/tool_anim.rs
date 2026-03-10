use super::facing_offset;
use crate::shared::*;
use bevy::prelude::*;

/// Per-tool frame duration in seconds. Heavy tools feel weighty,
/// light tools feel snappy. Total animation = duration x 4 frames.
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

/// Map a tool animation frame number onto the walk-frame bob cycle.
fn tool_bob_frame(frame: usize) -> usize {
    (frame + 1) % 4
}

// ═══════════════════════════════════════════════════════════════════════════
// Transform-based tool swing parameters per tool kind
// ═══════════════════════════════════════════════════════════════════════════

/// Returns (rotation_degrees, translation_offset) for each of the 4 frames.
/// Frame 0 = wind-up, 1 = swing, 2 = impact, 3 = recovery.
fn tool_swing_params(tool: ToolKind, frame: usize) -> (f32, Vec2) {
    match tool {
        // Hoe / Pickaxe / Axe: overhead swing arc
        ToolKind::Hoe | ToolKind::Pickaxe | ToolKind::Axe => match frame {
            0 => (-45.0, Vec2::new(0.0, 1.0)),  // wind-up: tilt back
            1 => (-15.0, Vec2::new(0.0, 0.5)),  // mid-swing
            2 => (30.0, Vec2::new(0.0, -1.0)),  // impact: snap forward
            3 => (10.0, Vec2::ZERO),             // recovery
            _ => (0.0, Vec2::ZERO),
        },
        // Watering can: gentle forward tilt (pour)
        ToolKind::WateringCan => match frame {
            0 => (5.0, Vec2::ZERO),              // slight lift
            1 => (15.0, Vec2::new(0.0, -0.5)),  // tilting
            2 => (20.0, Vec2::new(0.0, -1.0)),  // full pour
            3 => (8.0, Vec2::ZERO),              // recovery
            _ => (0.0, Vec2::ZERO),
        },
        // Fishing rod: cast arc (wind back then fling forward)
        ToolKind::FishingRod => match frame {
            0 => (-60.0, Vec2::new(0.0, 1.5)),  // wind-up far back
            1 => (-20.0, Vec2::new(0.0, 0.5)),  // mid-cast
            2 => (45.0, Vec2::new(0.0, -1.5)),  // cast forward
            3 => (15.0, Vec2::ZERO),             // follow-through
            _ => (0.0, Vec2::ZERO),
        },
        // Scythe: horizontal sweep with translation
        ToolKind::Scythe => match frame {
            0 => (-25.0, Vec2::new(-2.0, 0.0)),  // wind-up to the side
            1 => (-10.0, Vec2::new(-1.0, 0.0)),  // mid-sweep
            2 => (20.0, Vec2::new(2.0, 0.0)),    // impact sweep across
            3 => (8.0, Vec2::new(1.0, 0.0)),     // recovery
            _ => (0.0, Vec2::ZERO),
        },
    }
}

/// Mirror rotation direction based on facing. Left-facing tools swing
/// the opposite way to maintain visual consistency.
fn facing_rotation_sign(facing: &Facing) -> f32 {
    match facing {
        Facing::Left => -1.0,
        _ => 1.0,
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

/// System: when PlayerAnimState is ToolUse, apply transform-based rotation
/// and walk-frame bob to sell the swing animation.
/// Each tool has distinct rotation arcs and per-frame timing.
#[allow(clippy::too_many_arguments)]
pub fn animate_tool_use(
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &mut PlayerMovement,
            &mut Sprite,
            &mut Transform,
            &LogicalPosition,
        ),
        With<Player>,
    >,
    mut impact_events: EventWriter<ToolImpactEvent>,
    mut frame_timer: Local<f32>,
    mut impact_fired: Local<bool>,
) {
    for (entity, mut movement, mut sprite, mut transform, logical_pos) in query.iter_mut() {
        if let PlayerAnimState::ToolUse {
            tool,
            frame,
            total_frames,
        } = movement.anim_state
        {
            let duration = tool_frame_duration(tool);
            let rot_sign = facing_rotation_sign(&movement.facing);

            // Walk atlas layout: Row 0=Down, Row 1=Left, Row 2=Right, Row 3=Up.
            let facing_base: usize = match movement.facing {
                Facing::Down => 0,
                Facing::Left => 4,
                Facing::Right => 8,
                Facing::Up => 12,
            };

            let (rot_deg, translate_offset) = tool_swing_params(tool, frame as usize);

            if frame == 0 && *frame_timer == 0.0 {
                // First frame: start the bob on the next walk frame.
                sprite.flip_x = false;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = facing_base + 1;
                }
                *impact_fired = false;
                // Wind-up tint: slightly dim the sprite
                sprite.color = Color::srgb(0.85, 0.85, 0.9);
            }

            // Apply transform-based rotation for the current frame
            let angle_rad = (rot_deg * rot_sign).to_radians();
            transform.rotation = Quat::from_rotation_z(angle_rad);

            // Apply slight translation offset for tool swing feel
            let offset = Vec2::new(translate_offset.x * rot_sign, translate_offset.y);
            transform.translation.x = logical_pos.0.x + offset.x;
            transform.translation.y = logical_pos.0.y + offset.y;

            // Accumulate time
            *frame_timer += time.delta_secs();

            // Emit impact event on frame 2 (once)
            if frame >= 2 && !*impact_fired {
                *impact_fired = true;
                // Impact flash: bright white burst
                sprite.color = Color::srgb(1.5, 1.5, 1.5);
                // Impact squash: brief scale distortion
                transform.scale = Vec3::new(1.1, 0.9, 1.0);
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
                    // Animation complete — return to idle pose, normal color, reset transform
                    sprite.color = Color::WHITE;
                    sprite.flip_x = false;
                    transform.rotation = Quat::IDENTITY;
                    transform.scale = Vec3::ONE;
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
                    // Bob through walk frames to sell the swing
                    sprite.flip_x = false;
                    if let Some(atlas) = &mut sprite.texture_atlas {
                        let walk_frame = tool_bob_frame(new_frame as usize);
                        atlas.index = facing_base + walk_frame;
                    }
                    // Recovery frame: scale back to normal, fade tint
                    if new_frame == 3 {
                        sprite.color = Color::srgb(1.15, 1.15, 1.05);
                        transform.scale = Vec3::ONE;
                    }
                    movement.anim_state = PlayerAnimState::ToolUse {
                        tool,
                        frame: new_frame,
                        total_frames,
                    };
                }
            }
        } else {
            // Not in tool animation — reset timer and ensure transform is clean
            *frame_timer = 0.0;
            *impact_fired = false;
            if transform.rotation != Quat::IDENTITY {
                transform.rotation = Quat::IDENTITY;
            }
            if transform.scale != Vec3::ONE {
                transform.scale = Vec3::ONE;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Player shadow
// ═══════════════════════════════════════════════════════════════════════════

/// Marker component for the player shadow entity.
#[derive(Component)]
pub struct PlayerShadow;

/// Spawn a small elliptical shadow beneath the player.
/// Runs on `OnEnter(GameState::Playing)` after spawn_player.
pub fn spawn_player_shadow(mut commands: Commands) {
    commands.spawn((
        PlayerShadow,
        Sprite {
            color: Color::srgba(0.0, 0.0, 0.0, 0.2),
            custom_size: Some(Vec2::new(10.0, 5.0)),
            anchor: bevy::sprite::Anchor::Center,
            ..default()
        },
        // Start at origin; update_player_shadow will position it each frame.
        Transform::from_xyz(0.0, 0.0, Z_ENTITY_BASE - 0.5),
        Visibility::default(),
    ));
}

/// Each frame, position the shadow at the player's feet.
pub fn update_player_shadow(
    player_query: Query<&LogicalPosition, With<Player>>,
    mut shadow_query: Query<&mut Transform, With<PlayerShadow>>,
) {
    let Ok(pos) = player_query.get_single() else {
        return;
    };
    let Ok(mut shadow_tf) = shadow_query.get_single_mut() else {
        return;
    };

    // Position shadow at player's feet (bottom-center of sprite).
    shadow_tf.translation.x = pos.0.x;
    shadow_tf.translation.y = pos.0.y - 1.0;
    shadow_tf.translation.z = Z_ENTITY_BASE - 0.5;
}

// ═══════════════════════════════════════════════════════════════════════════
// Player breathing idle animation
// ═══════════════════════════════════════════════════════════════════════════

/// Adds a subtle breathing scale oscillation when the player is idle.
/// Scale Y oscillates between 0.98 and 1.02 at ~0.3 Hz.
pub fn animate_player_breathing(
    time: Res<Time>,
    mut query: Query<(&PlayerMovement, &mut Transform), With<Player>>,
) {
    for (movement, mut transform) in query.iter_mut() {
        if movement.anim_state == PlayerAnimState::Idle {
            // 0.3 Hz = period of ~3.33 seconds
            let t = time.elapsed_secs() * 0.3 * std::f32::consts::TAU;
            let breath_scale = 1.0 + 0.02 * t.sin(); // oscillates 0.98..1.02
            transform.scale.y = breath_scale;
            // Keep X at 1.0 during idle (no horizontal distortion)
            transform.scale.x = 1.0;
        }
        // During ToolUse, animate_tool_use manages scale.
        // During Walk, scale should be 1.0 (reset by tool_use cleanup or default).
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tool_bob_uses_walk_frame_cycle() {
        assert_eq!(super::tool_bob_frame(1), 2);
        assert_eq!(super::tool_bob_frame(2), 3);
        assert_eq!(super::tool_bob_frame(3), 0);
    }

    #[test]
    fn tool_swing_params_return_four_frames() {
        use crate::shared::ToolKind;
        for tool in [
            ToolKind::Axe,
            ToolKind::Pickaxe,
            ToolKind::Hoe,
            ToolKind::WateringCan,
            ToolKind::FishingRod,
            ToolKind::Scythe,
        ] {
            for frame in 0..4 {
                let (rot, _offset) = super::tool_swing_params(tool, frame);
                assert!(
                    rot.is_finite(),
                    "tool {:?} frame {} has non-finite rotation",
                    tool,
                    frame
                );
            }
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
