use super::facing_offset;
use crate::shared::*;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::Rng;

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
            0 => (-45.0, Vec2::new(0.0, 1.0)), // wind-up: tilt back
            1 => (-15.0, Vec2::new(0.0, 0.5)), // mid-swing
            2 => (30.0, Vec2::new(0.0, -1.0)), // impact: snap forward
            3 => (10.0, Vec2::ZERO),           // recovery
            _ => (0.0, Vec2::ZERO),
        },
        // Watering can: gentle forward tilt (pour)
        ToolKind::WateringCan => match frame {
            0 => (5.0, Vec2::ZERO),            // slight lift
            1 => (15.0, Vec2::new(0.0, -0.5)), // tilting
            2 => (20.0, Vec2::new(0.0, -1.0)), // full pour
            3 => (8.0, Vec2::ZERO),            // recovery
            _ => (0.0, Vec2::ZERO),
        },
        // Fishing rod: cast arc (wind back then fling forward)
        ToolKind::FishingRod => match frame {
            0 => (-60.0, Vec2::new(0.0, 1.5)), // wind-up far back
            1 => (-20.0, Vec2::new(0.0, 0.5)), // mid-cast
            2 => (45.0, Vec2::new(0.0, -1.5)), // cast forward
            3 => (15.0, Vec2::ZERO),           // follow-through
            _ => (0.0, Vec2::ZERO),
        },
        // Scythe: horizontal sweep with translation
        ToolKind::Scythe => match frame {
            0 => (-25.0, Vec2::new(-2.0, 0.0)), // wind-up to the side
            1 => (-10.0, Vec2::new(-1.0, 0.0)), // mid-sweep
            2 => (20.0, Vec2::new(2.0, 0.0)),   // impact sweep across
            3 => (8.0, Vec2::new(1.0, 0.0)),    // recovery
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

// ═══════════════════════════════════════════════════════════════════════════
// Procedural tool sprites
// ═══════════════════════════════════════════════════════════════════════════

/// Cached procedural tool sprite handles, generated once on first use.
#[derive(Resource, Default)]
pub struct ProceduralToolSprites {
    pub hoe: Option<Handle<Image>>,
    pub axe: Option<Handle<Image>>,
    pub pickaxe: Option<Handle<Image>>,
    pub watering_can: Option<Handle<Image>>,
    pub fishing_rod: Option<Handle<Image>>,
    pub scythe: Option<Handle<Image>>,
    pub loaded: bool,
}

/// Generate and cache all procedural tool sprites.
/// Runs once on `OnEnter(GameState::Playing)`.
pub fn load_procedural_tool_sprites(
    mut images: ResMut<Assets<Image>>,
    mut sprites: ResMut<ProceduralToolSprites>,
) {
    if sprites.loaded {
        return;
    }
    sprites.hoe = Some(images.add(make_tool_image_hoe()));
    sprites.axe = Some(images.add(make_tool_image_axe()));
    sprites.pickaxe = Some(images.add(make_tool_image_pickaxe()));
    sprites.watering_can = Some(images.add(make_tool_image_watering_can()));
    sprites.fishing_rod = Some(images.add(make_tool_image_fishing_rod()));
    sprites.scythe = Some(images.add(make_tool_image_scythe()));
    sprites.loaded = true;
}

/// Returns the cached handle for the given tool kind (if any).
pub fn tool_sprite_handle(
    sprites: &ProceduralToolSprites,
    tool: ToolKind,
) -> Option<Handle<Image>> {
    match tool {
        ToolKind::Hoe => sprites.hoe.clone(),
        ToolKind::Axe => sprites.axe.clone(),
        ToolKind::Pickaxe => sprites.pickaxe.clone(),
        ToolKind::WateringCan => sprites.watering_can.clone(),
        ToolKind::FishingRod => sprites.fishing_rod.clone(),
        ToolKind::Scythe => sprites.scythe.clone(),
    }
}

// ─── image builders ────────────────────────────────────────────────────────

fn new_tool_image(w: usize, h: usize, data: Vec<u8>) -> Image {
    let mut img = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img
}

/// Set a pixel at (px, py) in a flat RGBA buffer.
#[allow(clippy::too_many_arguments)]
fn set_px(data: &mut [u8], w: usize, px: usize, py: usize, r: u8, g: u8, b: u8, a: u8) {
    let i = (py * w + px) * 4;
    data[i] = r;
    data[i + 1] = g;
    data[i + 2] = b;
    data[i + 3] = a;
}

/// Brown handle: pixels (col, rows) within a handle column range.
fn draw_handle(data: &mut [u8], w: usize, col: usize, row_start: usize, row_end: usize) {
    for row in row_start..=row_end {
        set_px(data, w, col, row, 101, 67, 33, 255);
        if col + 1 < w {
            set_px(data, w, col + 1, row, 101, 67, 33, 255);
        }
    }
}

/// Hoe: brown handle (cols 6-7, rows 0-11) + gray blade (cols 2-7, rows 12-14)
fn make_tool_image_hoe() -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];
    // Handle
    draw_handle(&mut data, w, 6, 0, 11);
    // Blade
    for col in 2..=8 {
        for row in 12..=14 {
            set_px(&mut data, w, col, row, 120, 120, 120, 255);
        }
    }
    new_tool_image(w, h, data)
}

/// Axe: brown handle (cols 6-7, rows 0-9) + gray triangular head (rows 10-15)
fn make_tool_image_axe() -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];
    draw_handle(&mut data, w, 6, 0, 9);
    // Triangular head: wider at bottom, narrower at top
    let head_rows = [
        (10usize, 5usize, 9usize),
        (11, 4, 10),
        (12, 3, 11),
        (13, 3, 11),
        (14, 4, 10),
        (15, 5, 9),
    ];
    for (row, col_start, col_end) in head_rows {
        for col in col_start..=col_end {
            set_px(&mut data, w, col, row, 120, 120, 120, 255);
        }
    }
    new_tool_image(w, h, data)
}

/// Pickaxe: brown handle (cols 6-7, rows 0-9) + two-prong gray head
fn make_tool_image_pickaxe() -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];
    draw_handle(&mut data, w, 6, 0, 9);
    // Horizontal bar
    for col in 2..=12 {
        set_px(&mut data, w, col, 10, 120, 120, 120, 255);
        set_px(&mut data, w, col, 11, 120, 120, 120, 255);
    }
    // Left prong
    for row in 12..=15 {
        set_px(&mut data, w, 2, row, 120, 120, 120, 255);
        set_px(&mut data, w, 3, row, 120, 120, 120, 255);
    }
    // Right prong
    for row in 12..=15 {
        set_px(&mut data, w, 11, row, 120, 120, 120, 255);
        set_px(&mut data, w, 12, row, 120, 120, 120, 255);
    }
    new_tool_image(w, h, data)
}

/// WateringCan: blue-gray body (cols 3-12, rows 6-13) + small spout (cols 10-14, row 4-7)
fn make_tool_image_watering_can() -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];
    // Main body
    for col in 3..=12 {
        for row in 6..=13 {
            set_px(&mut data, w, col, row, 90, 120, 160, 255);
        }
    }
    // Spout (angled up-right)
    for col in 10..=14 {
        set_px(&mut data, w, col, 4, 90, 120, 160, 255);
        set_px(&mut data, w, col, 5, 90, 120, 160, 255);
    }
    // Handle (top arc)
    for col in 5..=9 {
        set_px(&mut data, w, col, 3, 101, 67, 33, 255);
        set_px(&mut data, w, col, 4, 101, 67, 33, 255);
    }
    new_tool_image(w, h, data)
}

/// FishingRod: thin brown line (col 7, rows 0-13) + white tip pixel
fn make_tool_image_fishing_rod() -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];
    // Single-pixel wide rod
    for row in 0..14 {
        set_px(&mut data, w, 7, row, 101, 67, 33, 255);
    }
    // White string at tip
    set_px(&mut data, w, 7, 14, 240, 240, 240, 255);
    set_px(&mut data, w, 8, 15, 240, 240, 240, 255);
    new_tool_image(w, h, data)
}

/// Scythe: brown handle (cols 6-7, rows 0-8) + curved gray blade
fn make_tool_image_scythe() -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];
    draw_handle(&mut data, w, 6, 0, 8);
    // Blade arc — approximate with a few rows of decreasing width
    let blade: [(usize, usize, usize); 6] = [
        (9, 2, 12),
        (10, 1, 11),
        (11, 1, 8),
        (12, 2, 6),
        (13, 4, 6),
        (14, 5, 6),
    ];
    for (row, col_start, col_end) in blade {
        for col in col_start..=col_end {
            set_px(&mut data, w, col, row, 120, 120, 120, 255);
        }
    }
    new_tool_image(w, h, data)
}

// ═══════════════════════════════════════════════════════════════════════════
// Held tool sprite (child of player during tool animation)
// ═══════════════════════════════════════════════════════════════════════════

/// Marker component for the held tool sprite entity.
#[derive(Component)]
pub struct HeldToolSprite;

/// Pixel offset from player center for the held tool, based on facing direction.
fn tool_hand_offset(facing: &Facing) -> Vec2 {
    match facing {
        Facing::Down => Vec2::new(6.0, -4.0),
        Facing::Up => Vec2::new(6.0, 4.0),
        Facing::Left => Vec2::new(-7.0, 0.0),
        Facing::Right => Vec2::new(7.0, 0.0),
    }
}

/// Spawn the held-tool sprite at animation start (frame 0 of ToolUse),
/// despawn it when animation ends.
///
/// This system runs after `animate_tool_use` so the anim state is up-to-date.
#[allow(clippy::too_many_arguments)]
pub fn update_held_tool_sprite(
    mut commands: Commands,
    player_query: Query<(Entity, &PlayerMovement, &LogicalPosition, &Transform), With<Player>>,
    held_query: Query<Entity, With<HeldToolSprite>>,
    tool_sprites: Res<ProceduralToolSprites>,
    mut spawned_for: Local<Option<ToolKind>>,
) {
    let Ok((player_entity, movement, logical_pos, player_tf)) = player_query.get_single() else {
        // Player entity gone (map transition / despawn) — reset tracking state
        if spawned_for.is_some() {
            *spawned_for = None;
        }
        return;
    };

    match movement.anim_state {
        PlayerAnimState::ToolUse { tool, frame, .. } => {
            // Spawn on first frame if not already spawned for this tool
            if spawned_for.is_none() {
                let Some(img) = tool_sprite_handle(&tool_sprites, tool) else {
                    return;
                };

                let hand_offset = tool_hand_offset(&movement.facing);
                let rot_sign = facing_rotation_sign(&movement.facing);
                let (rot_deg, _) = tool_swing_params(tool, frame as usize);
                let angle_rad = (rot_deg * rot_sign).to_radians();

                let tool_entity = commands
                    .spawn((
                        HeldToolSprite,
                        Sprite {
                            image: img,
                            custom_size: Some(Vec2::new(12.0, 6.0)),
                            anchor: bevy::sprite::Anchor::Center,
                            ..default()
                        },
                        Transform::from_xyz(
                            logical_pos.0.x + hand_offset.x,
                            logical_pos.0.y + hand_offset.y,
                            player_tf.translation.z + 0.2,
                        )
                        .with_rotation(Quat::from_rotation_z(angle_rad)),
                        Visibility::default(),
                    ))
                    .id();

                commands.entity(player_entity).add_child(tool_entity);
                *spawned_for = Some(tool);
            } else {
                // Update rotation + position each frame to match the swing arc
                let Ok(tool_entity) = held_query.get_single() else {
                    return;
                };
                let rot_sign = facing_rotation_sign(&movement.facing);
                let (rot_deg, swing_offset) = tool_swing_params(tool, frame as usize);
                let angle_rad = (rot_deg * rot_sign).to_radians();
                let hand_offset = tool_hand_offset(&movement.facing);
                let offset = Vec2::new(swing_offset.x * rot_sign, swing_offset.y);

                commands.entity(tool_entity).insert((Transform::from_xyz(
                    hand_offset.x + offset.x,
                    hand_offset.y + offset.y,
                    0.2, // local Z relative to parent
                )
                .with_rotation(Quat::from_rotation_z(angle_rad)),));
            }
        }
        _ => {
            // Animation ended — despawn the held sprite
            if spawned_for.is_some() {
                for e in held_query.iter() {
                    commands.entity(e).despawn_recursive();
                }
                *spawned_for = None;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Impact particles
// ═══════════════════════════════════════════════════════════════════════════

/// A particle emitted when a tool hits the ground.
#[derive(Component)]
pub struct ImpactParticle {
    /// Remaining lifetime.
    pub lifetime: Timer,
    /// Current velocity (px/s).
    pub velocity: Vec2,
    /// Gravity deceleration applied to Y each second.
    pub gravity: f32,
    /// Initial alpha for fade calculation.
    pub initial_alpha: f32,
}

/// Spawn impact particles when the tool reaches frame 2 (the impact frame).
/// This system reads `ToolImpactEvent` and spawns colour-coded debris.
pub fn spawn_impact_particles(
    mut commands: Commands,
    mut impact_events: EventReader<ToolImpactEvent>,
) {
    let mut rng = rand::thread_rng();

    for event in impact_events.read() {
        // Target tile centre in world space
        let wx = event.grid_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let wy = event.grid_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        let (color, count, gravity) = match event.tool {
            ToolKind::Hoe => (
                Color::srgba(0.55, 0.42, 0.28, 0.8),
                rng.gen_range(4..=6usize),
                100.0f32,
            ),
            ToolKind::Pickaxe => (
                Color::srgba(0.6, 0.6, 0.6, 0.8),
                rng.gen_range(4..=6usize),
                100.0f32,
            ),
            ToolKind::Axe => (
                Color::srgba(0.6, 0.45, 0.25, 0.8),
                rng.gen_range(4..=6usize),
                100.0f32,
            ),
            ToolKind::Scythe => (
                Color::srgba(0.4, 0.6, 0.25, 0.8),
                rng.gen_range(4..=6usize),
                100.0f32,
            ),
            ToolKind::WateringCan => (
                Color::srgba(0.4, 0.6, 1.0, 0.6),
                rng.gen_range(3..=4usize),
                60.0f32,
            ),
            // Fishing rod: no ground impact particles
            ToolKind::FishingRod => continue,
        };

        for _ in 0..count {
            // Random burst direction
            let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
            let speed = rng.gen_range(30.0f32..80.0);
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;
            let lifetime_secs = rng.gen_range(0.3f32..0.5);

            commands.spawn((
                ImpactParticle {
                    lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                    velocity: Vec2::new(vx, vy),
                    gravity,
                    initial_alpha: color.to_srgba().alpha,
                },
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(2.0, 2.0)),
                    ..default()
                },
                Transform::from_xyz(wx, wy, Z_EFFECTS),
                Visibility::default(),
            ));
        }
    }
}

/// Each frame: move particles, apply gravity, fade alpha, despawn when expired.
pub fn update_impact_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut ImpactParticle)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Move
        tf.translation.x += particle.velocity.x * dt;
        tf.translation.y += particle.velocity.y * dt;

        // Gravity: reduce Y velocity
        particle.velocity.y -= particle.gravity * dt;

        // Fade alpha proportional to remaining lifetime fraction
        let fraction = 1.0 - particle.lifetime.fraction();
        let alpha = particle.initial_alpha * fraction;
        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(base.red, base.green, base.blue, alpha);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Soil tilling dust poof
// ═══════════════════════════════════════════════════════════════════════════

/// A brief expanding dust-cloud overlay spawned when the hoe tills soil.
#[derive(Component)]
pub struct TillPoof {
    pub timer: Timer,
}

/// Listen for hoe impact events and spawn a dust poof at the tilled tile.
pub fn spawn_till_poof(mut commands: Commands, mut impact_events: EventReader<ToolImpactEvent>) {
    for event in impact_events.read() {
        if event.tool != ToolKind::Hoe {
            continue;
        }
        let wx = event.grid_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let wy = event.grid_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        commands.spawn((
            TillPoof {
                timer: Timer::from_seconds(0.2, TimerMode::Once),
            },
            Sprite {
                color: Color::srgba(0.55, 0.42, 0.28, 0.3),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(wx, wy, Z_FARM_OVERLAY + 2.0).with_scale(Vec3::splat(0.5)),
            Visibility::default(),
        ));
    }
}

/// Each frame: scale the poof from 0.5 → 1.5, fade alpha to 0, despawn when done.
pub fn update_till_poof(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut TillPoof)>,
) {
    for (entity, mut tf, mut sprite, mut poof) in query.iter_mut() {
        poof.timer.tick(time.delta());

        if poof.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let t = poof.timer.fraction(); // 0.0 → 1.0
        let scale = 0.5 + t * 1.0; // 0.5 → 1.5
        tf.scale = Vec3::splat(scale);

        let alpha = 0.3 * (1.0 - t);
        sprite.color = Color::srgba(0.55, 0.42, 0.28, alpha);
    }
}
