//! Minigame UI rendering, bobber animation, and fish display animations.

use bevy::prelude::*;
use rand::Rng;

use super::minigame::{
    zone_to_screen_y, MINIGAME_BAR_HEIGHT, MINIGAME_BAR_WIDTH, PROGRESS_BAR_HEIGHT,
    PROGRESS_BAR_WIDTH, PROGRESS_BAR_Y,
};
use super::{
    Bobber, BobberRippleTimer, BobberSplashSpawned, FishingMinigameState, FishingPhase,
    FishingState, MinigameBgBar, MinigameCatchBar, MinigameFishZone, MinigameProgressBg,
    MinigameProgressFill, MinigameRoot, WaterDroplet, WaterRipple,
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
    Color::srgb(0.72, 0.58, 0.34)
}

fn color_catch_bar() -> Color {
    Color::srgb(0.46, 0.72, 0.56)
}

fn color_progress_bg() -> Color {
    Color::srgb(0.25, 0.25, 0.25)
}

fn color_progress_fill() -> Color {
    Color::srgb(0.39, 0.64, 0.74)
}

fn color_progress_fill_near() -> Color {
    Color::srgb(0.73, 0.70, 0.42)
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
    let bar_world_x = (SCREEN_WIDTH / 2.0 - 140.0) * screen_to_world;
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
    for (mut transform, mut bobber) in bobber_query.iter_mut() {
        let is_bite = fishing_state.phase == FishingPhase::BitePending;

        // Faster, deeper bob when a fish has bitten
        let bob_speed = if is_bite { 3.0 } else { 1.2 };
        let bob_amplitude = if is_bite { 4.0 } else { 1.5 };

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

        sprite.color = Color::srgba(
            glow.base_color.0,
            glow.base_color.1,
            glow.base_color.2,
            alpha,
        );
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
                Transform::from_translation(Vec3::new(position.x, position.y, position.z - 0.1)),
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

// ─── Water Splash / Ripple helpers ───────────────────────────────────────────

/// Parameters for a single water splash burst.
struct SplashParams {
    pos: Vec3,
    count: usize,
    /// Square droplet side length (world units).
    droplet_size: f32,
    speed_min: f32,
    speed_max: f32,
    ripple_start: f32,
    ripple_end: f32,
    ripple_alpha: f32,
}

/// Spawn a burst of water droplet particles plus an initial ripple at `params.pos`.
fn spawn_water_splash(commands: &mut Commands, rng: &mut impl rand::Rng, p: &SplashParams) {
    // Droplets
    for _ in 0..p.count {
        // Upward burst: angle biased upward (between 30° and 150° from horizontal)
        let angle = rng.gen_range(
            std::f32::consts::FRAC_PI_6..std::f32::consts::PI - std::f32::consts::FRAC_PI_6,
        );
        let speed = rng.gen_range(p.speed_min..p.speed_max);
        let vx = angle.cos() * speed * rng.gen_range(-1.0f32..1.0).signum();
        let vy = angle.sin() * speed;
        let lifetime_secs = rng.gen_range(0.3f32..0.4);

        commands.spawn((
            WaterDroplet {
                lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                velocity: Vec2::new(vx, vy),
                gravity: 80.0,
                initial_alpha: 0.6,
            },
            Sprite {
                color: Color::srgba(0.5, 0.7, 1.0, 0.6),
                custom_size: Some(Vec2::splat(p.droplet_size)),
                ..default()
            },
            Transform::from_translation(p.pos),
        ));
    }

    // Ripple circle
    commands.spawn((
        WaterRipple {
            lifetime: Timer::from_seconds(0.3, TimerMode::Once),
            start_size: p.ripple_start,
            end_size: p.ripple_end,
            start_alpha: p.ripple_alpha,
        },
        Sprite {
            color: Color::srgba(0.4, 0.6, 1.0, p.ripple_alpha),
            custom_size: Some(Vec2::splat(p.ripple_start)),
            ..default()
        },
        Transform::from_translation(p.pos),
    ));
}

// ─── System: spawn landing splash when bobber first appears ──────────────────

/// Query filter for bobbers that haven't yet received their landing splash.
type NewBobberQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static Transform), (With<Bobber>, Without<BobberSplashSpawned>)>;

/// Detects newly spawned bobber entities (those without `BobberSplashSpawned`)
/// and fires the landing water splash effect.
pub fn spawn_bobber_landing_splash(mut commands: Commands, new_bobbers: NewBobberQuery) {
    let mut rng = rand::thread_rng();

    for (entity, transform) in new_bobbers.iter() {
        let pos = transform.translation;
        // Landing splash: 3-4 droplets, 2×2 px, 20-40 px/s, ripple 6→12, alpha 0.2
        let droplet_count = rng.gen_range(3..=4);
        spawn_water_splash(
            &mut commands,
            &mut rng,
            &SplashParams {
                pos: Vec3::new(pos.x, pos.y, Z_EFFECTS),
                count: droplet_count,
                droplet_size: 2.0,
                speed_min: 20.0,
                speed_max: 40.0,
                ripple_start: 6.0,
                ripple_end: 12.0,
                ripple_alpha: 0.2,
            },
        );

        // Mark spawned and add the ambient ripple timer (fires every 1.0–1.5 s)
        let ripple_interval = rng.gen_range(1.0f32..1.5);
        commands.entity(entity).insert((
            BobberSplashSpawned,
            BobberRippleTimer {
                timer: Timer::from_seconds(ripple_interval, TimerMode::Repeating),
            },
        ));
    }
}

// ─── System: ambient ripple while bobber waits for a bite ────────────────────

/// Spawns a small ripple every 1.0–1.5 seconds at the bobber's current position.
pub fn tick_bobber_ambient_ripple(
    mut commands: Commands,
    mut bobber_query: Query<(&Transform, &mut BobberRippleTimer), With<Bobber>>,
    fishing_state: Res<FishingState>,
    time: Res<Time>,
) {
    // Only during WaitingForBite phase
    if fishing_state.phase != FishingPhase::WaitingForBite {
        return;
    }

    for (transform, mut ripple_timer) in bobber_query.iter_mut() {
        ripple_timer.timer.tick(time.delta());

        if ripple_timer.timer.just_finished() {
            let pos = transform.translation;
            commands.spawn((
                WaterRipple {
                    lifetime: Timer::from_seconds(0.3, TimerMode::Once),
                    start_size: 4.0,
                    end_size: 8.0,
                    start_alpha: 0.1,
                },
                Sprite {
                    color: Color::srgba(0.4, 0.6, 1.0, 0.1),
                    custom_size: Some(Vec2::splat(4.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, Z_EFFECTS),
            ));

            // Randomise the next interval
            let mut rng = rand::thread_rng();
            let next = rng.gen_range(1.0f32..1.5);
            ripple_timer.timer = Timer::from_seconds(next, TimerMode::Repeating);
        }
    }
}

// ─── System: bite splash when FishingPhase transitions to BitePending ────────

/// Detects the first frame the phase becomes `BitePending` and fires a larger
/// splash to signal "FISH ON!" visually.
pub fn spawn_bite_splash(
    mut commands: Commands,
    fishing_state: Res<FishingState>,
    bobber_query: Query<&Transform, With<Bobber>>,
    mut was_bite_pending: Local<bool>,
) {
    let now_bite = fishing_state.phase == FishingPhase::BitePending;

    if now_bite && !*was_bite_pending {
        // First frame of BitePending — fire the bite splash
        if let Ok(transform) = bobber_query.get_single() {
            let pos = transform.translation;
            let mut rng = rand::thread_rng();
            // Bite splash: 5-6 droplets, 3×3 px, 40-80 px/s, ripple 8→16, alpha 0.25
            let droplet_count = rng.gen_range(5..=6);
            spawn_water_splash(
                &mut commands,
                &mut rng,
                &SplashParams {
                    pos: Vec3::new(pos.x, pos.y, Z_EFFECTS),
                    count: droplet_count,
                    droplet_size: 3.0,
                    speed_min: 40.0,
                    speed_max: 80.0,
                    ripple_start: 8.0,
                    ripple_end: 16.0,
                    ripple_alpha: 0.25,
                },
            );
        }
    }

    *was_bite_pending = now_bite;
}

// ─── System: update water droplets ───────────────────────────────────────────

/// Move droplets, apply gravity, fade alpha, despawn when lifetime expires.
pub fn update_water_droplets(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut WaterDroplet)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut droplet) in query.iter_mut() {
        droplet.lifetime.tick(time.delta());

        if droplet.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Move
        tf.translation.x += droplet.velocity.x * dt;
        tf.translation.y += droplet.velocity.y * dt;

        // Apply gravity
        droplet.velocity.y -= droplet.gravity * dt;

        // Fade alpha
        let fraction = 1.0 - droplet.lifetime.fraction();
        let alpha = droplet.initial_alpha * fraction;
        sprite.color = Color::srgba(0.5, 0.7, 1.0, alpha);
    }
}

// ─── System: update water ripples ────────────────────────────────────────────

/// Expand ripple circles and fade them to transparent, then despawn.
pub fn update_water_ripples(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut WaterRipple)>,
) {
    for (entity, mut tf, mut sprite, mut ripple) in query.iter_mut() {
        ripple.lifetime.tick(time.delta());

        if ripple.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let t = ripple.lifetime.fraction(); // 0.0 → 1.0

        // Interpolate size
        let size = ripple.start_size + (ripple.end_size - ripple.start_size) * t;
        tf.scale = Vec3::splat(size / ripple.start_size);

        // Fade alpha from start_alpha → 0
        let alpha = ripple.start_alpha * (1.0 - t);
        sprite.color = Color::srgba(0.4, 0.6, 1.0, alpha);
    }
}
