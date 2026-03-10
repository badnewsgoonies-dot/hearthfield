//! Minigame UI rendering, bobber animation, and fish display animations.

use bevy::prelude::*;

use super::minigame::{
    zone_to_screen_y, MINIGAME_BAR_HEIGHT, MINIGAME_BAR_WIDTH, PROGRESS_BAR_HEIGHT,
    PROGRESS_BAR_WIDTH, PROGRESS_BAR_Y,
};
use super::{
    Bobber, FishingMinigameState, FishingState, MinigameBgBar, MinigameCatchBar, MinigameFishZone,
    MinigameProgressBg, MinigameProgressFill, MinigameRoot,
};
use crate::shared::*;

// ─── Fish Display Animation Components ───────────────────────────────────────

/// Drives a gentle swimming animation on displayed fish sprites (catch popup, encyclopedia).
/// Uses Transform-based sinusoidal oscillation: horizontal sway + slight rotation.
#[derive(Component, Debug, Clone)]
pub struct FishDisplayAnim {
    /// Current phase accumulator (radians).
    pub phase: f32,
    /// Oscillation speed in radians per second. Smaller fish swim faster.
    pub speed: f32,
}

impl FishDisplayAnim {
    /// Create a new animation with speed derived from fish difficulty.
    /// Lower difficulty (smaller/easier fish) → faster oscillation.
    #[allow(dead_code)]
    pub fn from_difficulty(difficulty: f32) -> Self {
        // Map difficulty 0.0→1.0 to speed 4.0→1.5 rad/s (smaller fish are faster)
        let speed = 4.0 - difficulty * 2.5;
        Self { phase: 0.0, speed }
    }
}

/// Marks a glow/aura backdrop entity behind a rare or legendary fish display.
/// The glow pulses in alpha between `alpha_min` and `alpha_max`.
#[derive(Component, Debug, Clone)]
pub struct FishRarityGlow {
    /// Phase accumulator for the pulse.
    pub phase: f32,
    /// Pulse frequency in Hz.
    pub frequency: f32,
    /// Minimum alpha value.
    pub alpha_min: f32,
    /// Maximum alpha value.
    pub alpha_max: f32,
    /// Base color (RGB) of the glow — alpha is overridden by the pulse.
    pub base_color: (f32, f32, f32),
}

// ─── Colors ───────────────────────────────────────────────────────────────────

fn color_bg_bar() -> Color {
    Color::srgba(0.15, 0.15, 0.15, 0.85)
}

fn color_fish_zone() -> Color {
    Color::srgb(0.9, 0.35, 0.1)
}

fn color_catch_bar() -> Color {
    Color::srgb(0.2, 0.85, 0.3)
}

fn color_progress_bg() -> Color {
    Color::srgb(0.25, 0.25, 0.25)
}

fn color_progress_fill() -> Color {
    Color::srgb(0.1, 0.7, 0.95)
}

fn color_progress_fill_near() -> Color {
    Color::srgb(0.95, 0.85, 0.1)
}

// ─── Z-layers ─────────────────────────────────────────────────────────────────

const Z_UI_BG: f32 = 50.0;

// ─── OnEnter(GameState::Fishing) — spawn minigame UI ─────────────────────────

/// Spawn the fishing minigame UI when entering the Fishing game state.
///
/// The minigame bar is positioned in the right portion of the visible screen.
/// Since the camera uses a scale of 1/PIXEL_SCALE, we need to account for that
/// when converting screen pixel coordinates to world coordinates. With the camera
/// at 1/PIXEL_SCALE scale, 1 screen pixel = PIXEL_SCALE world units. However,
/// the camera projection already handles this — sprites placed at world coords
/// appear at their world position divided by the camera scale factor on screen.
/// In practice, we position UI relative to screen center (world origin).
pub fn spawn_minigame_ui(mut commands: Commands, minigame_state: Res<FishingMinigameState>) {
    // The camera scale is 1/PIXEL_SCALE. With a 960x540 screen:
    // The camera shows a region of 960*PIXEL_SCALE × 540*PIXEL_SCALE world units.
    // Screen right edge ≈ SCREEN_WIDTH/2 * PIXEL_SCALE world units from center.
    // We place the minigame bar near the right edge.
    let screen_to_world = PIXEL_SCALE;

    // Bar position: right side, vertically centered
    let bar_world_x = (SCREEN_WIDTH / 2.0 - 90.0) * screen_to_world;
    let bar_world_y = 0.0_f32;

    // Convert bar dimensions from screen-pixels to world units
    let bar_h_world = MINIGAME_BAR_HEIGHT * screen_to_world;
    let bar_w_world = MINIGAME_BAR_WIDTH * screen_to_world;

    let fish_zone_h_world = minigame_state.fish_zone_half * 2.0 * (bar_h_world / 100.0);
    let catch_bar_h_world = minigame_state.catch_bar_half * 2.0 * (bar_h_world / 100.0);

    let progress_bar_y_world = PROGRESS_BAR_Y * screen_to_world;
    let progress_w_world = PROGRESS_BAR_WIDTH * screen_to_world;
    let progress_h_world = PROGRESS_BAR_HEIGHT * screen_to_world;

    // Scale factor: zone 0-100 maps to bar height
    let y_scale = bar_h_world / MINIGAME_BAR_HEIGHT;

    let fish_y = zone_to_screen_y(minigame_state.fish_zone_center) * y_scale;
    let catch_y = zone_to_screen_y(minigame_state.catch_bar_center) * y_scale;

    // Spawn root entity (transparent container)
    commands
        .spawn((
            Sprite {
                color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(1.0, 1.0)),
                ..default()
            },
            Transform::from_translation(Vec3::new(bar_world_x, bar_world_y, Z_UI_BG)),
            MinigameRoot,
        ))
        .with_children(|parent| {
            // Background bar (dark, semi-transparent)
            parent.spawn((
                Sprite {
                    color: color_bg_bar(),
                    custom_size: Some(Vec2::new(bar_w_world, bar_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                MinigameBgBar,
            ));

            // Fish zone (red/orange — target to overlap)
            parent.spawn((
                Sprite {
                    color: color_fish_zone(),
                    custom_size: Some(Vec2::new(bar_w_world * 0.85, fish_zone_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, fish_y, 1.0)),
                MinigameFishZone,
            ));

            // Catch bar (green — controlled by player)
            parent.spawn((
                Sprite {
                    color: color_catch_bar(),
                    custom_size: Some(Vec2::new(bar_w_world, catch_bar_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, catch_y, 1.5)),
                MinigameCatchBar,
            ));

            // Progress bar background
            parent.spawn((
                Sprite {
                    color: color_progress_bg(),
                    custom_size: Some(Vec2::new(progress_w_world, progress_h_world)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, progress_bar_y_world, 1.0)),
                MinigameProgressBg,
            ));

            // Progress bar fill
            // Anchored to the left edge; x-scale = 0.001 to 1.0 representing 0-100%
            parent.spawn((
                Sprite {
                    color: color_progress_fill(),
                    custom_size: Some(Vec2::new(progress_w_world, progress_h_world)),
                    anchor: bevy::sprite::Anchor::CenterLeft,
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    -progress_w_world / 2.0,
                    progress_bar_y_world,
                    2.0,
                ))
                .with_scale(Vec3::new(0.001, 1.0, 1.0)),
                MinigameProgressFill,
            ));
        });
}

/// Clean up all minigame UI entities when leaving GameState::Fishing.
pub fn despawn_minigame_ui(mut commands: Commands, root_query: Query<Entity, With<MinigameRoot>>) {
    for entity in root_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Bobber animation ─────────────────────────────────────────────────────────

/// Animate the bobber with gentle sinusoidal bobbing.
/// When a bite is pending, the bobber dips more aggressively to signal the player.
pub fn animate_bobber(
    mut bobber_query: Query<(&mut Transform, &mut Bobber)>,
    fishing_state: Res<FishingState>,
    time: Res<Time>,
) {
    use super::FishingPhase;

    for (mut transform, mut bobber) in bobber_query.iter_mut() {
        let is_bite = fishing_state.phase == FishingPhase::BitePending;

        // Faster, deeper bob when a fish has bitten
        let bob_speed = if is_bite { 4.0 } else { 1.5 };
        let bob_amplitude = if is_bite { 6.0 } else { 2.0 };

        bobber.bob_timer.tick(time.delta());

        let elapsed = time.elapsed_secs();
        let bob_y = (elapsed * bob_speed).sin() * bob_amplitude;

        transform.translation.y = bobber.original_y + bob_y;
    }
}

// ─── Progress fill color update ───────────────────────────────────────────────

/// Update the progress fill color to give visual feedback as it nears 100%.
/// Called every frame during the Fishing state.
pub fn update_progress_fill_color(
    minigame_state: Res<FishingMinigameState>,
    mut fill_query: Query<&mut Sprite, With<MinigameProgressFill>>,
) {
    for mut sprite in fill_query.iter_mut() {
        if minigame_state.progress > 75.0 {
            // Lerp from blue toward yellow as progress approaches 100%
            let t = (minigame_state.progress - 75.0) / 25.0;
            let base = color_progress_fill().to_srgba();
            let near = color_progress_fill_near().to_srgba();
            let r = base.red * (1.0 - t) + near.red * t;
            let g = base.green * (1.0 - t) + near.green * t;
            let b = base.blue * (1.0 - t) + near.blue * t;
            sprite.color = Color::srgb(r, g, b);
        } else {
            sprite.color = color_progress_fill();
        }
    }
}

// ─── Fish Display Swimming Animation ─────────────────────────────────────────

/// Animate fish display sprites with a gentle swimming motion.
///
/// Applies sinusoidal horizontal oscillation (±2.5 pixels) and slight rotation
/// (±5 degrees) to any entity bearing `FishDisplayAnim`. The animation phase
/// accumulates over time and the speed varies by fish size (smaller = faster).
///
/// Note: this system writes the x-offset and rotation directly from the phase,
/// so the entity's base x position should remain at its spawn location. The
/// oscillation is purely cosmetic.
pub fn animate_fish_display(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut FishDisplayAnim)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut anim) in query.iter_mut() {
        anim.phase += anim.speed * dt;

        // Horizontal sway: ±2.5 pixels via sin wave
        // We apply the delta (difference between current and previous sin value)
        // so the transform tracks the oscillation without drift.
        let prev_phase = anim.phase - anim.speed * dt;
        let dx = anim.phase.sin() * 2.5 - prev_phase.sin() * 2.5;
        transform.translation.x += dx;

        // Slight rotation: ±5 degrees (±0.087 radians)
        let rotation_angle = anim.phase.sin() * 0.087;
        transform.rotation = Quat::from_rotation_z(rotation_angle);
    }
}

// ─── Fish Rarity Glow Animation ──────────────────────────────────────────────

/// Pulse the alpha of rarity glow backdrop entities.
///
/// Legendary fish get a gold aura (1.0, 0.85, 0.2) and Rare fish get a silver
/// aura (0.8, 0.85, 0.9). The glow pulses at ~0.7 Hz between alpha 0.2 and 0.5.
pub fn animate_fish_rarity_glow(
    time: Res<Time>,
    mut query: Query<(&mut Sprite, &mut FishRarityGlow)>,
) {
    let dt = time.delta_secs();
    for (mut sprite, mut glow) in query.iter_mut() {
        glow.phase += std::f32::consts::TAU * glow.frequency * dt;

        // Oscillate alpha between alpha_min and alpha_max using sin wave
        let t = (glow.phase.sin() + 1.0) * 0.5; // map [-1,1] → [0,1]
        let alpha = glow.alpha_min + t * (glow.alpha_max - glow.alpha_min);

        sprite.color =
            Color::srgba(glow.base_color.0, glow.base_color.1, glow.base_color.2, alpha);
    }
}

// ─── Helper: Spawn a fish display with optional rarity glow ──────────────────

/// Spawn a fish display entity with swimming animation and optional rarity glow.
///
/// This is a helper used by catch popups and the encyclopedia screen.
/// Call from any system that has access to `Commands` and the fishing atlas.
///
/// - `position`: world-space position for the fish sprite
/// - `sprite_index`: atlas index of the fish sprite
/// - `difficulty`: the fish's difficulty rating (affects swim speed)
/// - `rarity`: determines whether a glow backdrop is spawned
/// - `atlas_image` / `atlas_layout`: the fishing sprite atlas handles
///
/// Returns the spawned entity.
#[allow(dead_code)]
pub fn spawn_fish_display(
    commands: &mut Commands,
    position: Vec3,
    sprite_index: u32,
    difficulty: f32,
    rarity: Rarity,
    atlas_image: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
) -> Entity {
    let fish_entity = commands
        .spawn((
            Sprite::from_atlas_image(
                atlas_image.clone(),
                TextureAtlas {
                    layout: atlas_layout.clone(),
                    index: sprite_index as usize,
                },
            ),
            Transform::from_translation(position),
            FishDisplayAnim::from_difficulty(difficulty),
        ))
        .id();

    // Spawn glow backdrop for Rare and Legendary fish
    match rarity {
        Rarity::Legendary => {
            commands.spawn((
                Sprite {
                    color: Color::srgba(1.0, 0.85, 0.2, 0.3),
                    custom_size: Some(Vec2::new(24.0, 24.0)), // slightly larger than 16x16 fish
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    position.x,
                    position.y,
                    position.z - 0.1, // behind the fish
                )),
                FishRarityGlow {
                    phase: 0.0,
                    frequency: 0.7,
                    alpha_min: 0.2,
                    alpha_max: 0.5,
                    base_color: (1.0, 0.85, 0.2), // gold
                },
            ));
        }
        Rarity::Rare => {
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.8, 0.85, 0.9, 0.3),
                    custom_size: Some(Vec2::new(24.0, 24.0)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    position.x,
                    position.y,
                    position.z - 0.1,
                )),
                FishRarityGlow {
                    phase: 0.0,
                    frequency: 0.7,
                    alpha_min: 0.2,
                    alpha_max: 0.5,
                    base_color: (0.8, 0.85, 0.9), // silver
                },
            ));
        }
        _ => {}
    }

    fish_entity
}
