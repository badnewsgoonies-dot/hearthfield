//! Day/night cycle ambient tint overlay system.
//!
//! Spawns a full-screen UI overlay that tints the scene based on the current
//! time of day read from the Calendar resource. Indoor maps are always fully
//! lit (no tint). Smoothly interpolates between keyframed tint values.

use bevy::prelude::*;

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker component for the full-screen day/night overlay entity.
#[derive(Component, Debug)]
pub struct DayNightOverlay;

// ═══════════════════════════════════════════════════════════════════════
// LIGHTNING FLASH RESOURCE (for storms)
// ═══════════════════════════════════════════════════════════════════════

/// Tracks lightning flash state during storms.
#[derive(Resource, Debug)]
pub struct LightningFlash {
    /// Timer until the next lightning flash.
    pub next_flash_timer: Timer,
    /// Whether the screen is currently flashing white.
    pub flashing: bool,
    /// Remaining fade time for the current flash (starts at 0.3s).
    pub flash_fade_remaining: f32,
    /// Delay before thunder SFX plays after the flash.
    pub thunder_delay: Option<f32>,
}

impl Default for LightningFlash {
    fn default() -> Self {
        Self {
            next_flash_timer: Timer::from_seconds(10.0, TimerMode::Once),
            flashing: false,
            flash_fade_remaining: 0.0,
            thunder_delay: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// KEYFRAME DATA
// ═══════════════════════════════════════════════════════════════════════

/// A single keyframe for the day/night tint cycle.
struct TintKeyframe {
    hour: f32,
    tint: (f32, f32, f32),
    intensity: f32,
}

/// Returns the ordered list of tint keyframes for a 24-hour cycle.
fn tint_keyframes() -> &'static [TintKeyframe] {
    // Using a static array allocated once.
    // Keyframes define tint color and overlay intensity at specific hours.
    // intensity = alpha of the overlay (0.0 = invisible, 0.5 = heavy tint)
    static KEYFRAMES: &[TintKeyframe] = &[
        TintKeyframe { hour: 0.0,  tint: (0.3, 0.3, 0.5), intensity: 0.5 },  // midnight
        TintKeyframe { hour: 5.0,  tint: (0.3, 0.3, 0.5), intensity: 0.5 },  // late night
        TintKeyframe { hour: 6.0,  tint: (1.0, 0.9, 0.7), intensity: 0.15 }, // sunrise
        TintKeyframe { hour: 8.0,  tint: (1.0, 1.0, 0.95), intensity: 0.05 }, // morning
        TintKeyframe { hour: 10.0, tint: (1.0, 1.0, 1.0), intensity: 0.0 },  // full daylight
        TintKeyframe { hour: 16.0, tint: (1.0, 1.0, 1.0), intensity: 0.0 },  // full daylight
        TintKeyframe { hour: 18.0, tint: (1.0, 0.85, 0.6), intensity: 0.15 }, // sunset
        TintKeyframe { hour: 20.0, tint: (0.6, 0.6, 0.9), intensity: 0.3 },  // twilight
        TintKeyframe { hour: 22.0, tint: (0.3, 0.3, 0.5), intensity: 0.5 },  // night
        TintKeyframe { hour: 24.0, tint: (0.3, 0.3, 0.5), intensity: 0.5 },  // midnight (wrap)
    ];
    KEYFRAMES
}

/// Linearly interpolate between two floats.
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Sample the tint keyframes at a given time (0.0 - 24.0+).
fn sample_tint(time: f32) -> ((f32, f32, f32), f32) {
    // Clamp time to 0..24 range. Calendar can go up to 25 (1 AM next day).
    let t = time.clamp(0.0, 24.0);
    let keyframes = tint_keyframes();

    // Find the two keyframes that bracket `t`.
    for i in 0..keyframes.len() - 1 {
        let kf_a = &keyframes[i];
        let kf_b = &keyframes[i + 1];
        if t >= kf_a.hour && t <= kf_b.hour {
            let range = kf_b.hour - kf_a.hour;
            if range < 0.001 {
                return (kf_a.tint, kf_a.intensity);
            }
            let frac = (t - kf_a.hour) / range;
            let tint = (
                lerp_f32(kf_a.tint.0, kf_b.tint.0, frac),
                lerp_f32(kf_a.tint.1, kf_b.tint.1, frac),
                lerp_f32(kf_a.tint.2, kf_b.tint.2, frac),
            );
            let intensity = lerp_f32(kf_a.intensity, kf_b.intensity, frac);
            return (tint, intensity);
        }
    }

    // Fallback: full daylight
    ((1.0, 1.0, 1.0), 0.0)
}

/// Returns true if the given map is indoors (should have no day/night tint).
fn is_indoor_map(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PlayerHouse | MapId::GeneralStore | MapId::AnimalShop | MapId::Blacksmith
    )
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn the full-screen day/night overlay when entering Playing state.
pub fn spawn_day_night_overlay(mut commands: Commands) {
    commands.spawn((
        DayNightOverlay,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        // Start fully transparent
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        // Very high z-index so it draws on top of everything else
        ZIndex(900),
        // CRITICAL: Don't block mouse/touch input
        PickingBehavior::IGNORE,
    ));
}

/// Despawn the overlay when leaving Playing state.
pub fn despawn_day_night_overlay(
    mut commands: Commands,
    query: Query<Entity, With<DayNightOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

/// Every frame, update the overlay color based on the current time of day.
///
/// For indoor maps the overlay is fully transparent. For outdoor maps, the
/// overlay fades between keyframed tint values to simulate sunrise, daylight,
/// sunset, twilight, and night.
///
/// The overlay color is computed as a semi-transparent tinted layer:
/// - At full daylight (intensity 0.0): alpha = 0, overlay invisible.
/// - At night (intensity 0.5): dark blue overlay with alpha ~0.5.
/// - The tint RGB channels darken towards the tint color as intensity grows.
pub fn update_day_night_tint(
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    mut day_night_tint: ResMut<DayNightTint>,
    mut overlay_query: Query<&mut BackgroundColor, With<DayNightOverlay>>,
    lightning: Option<Res<LightningFlash>>,
) {
    // If we're on an indoor map, force no tint
    if is_indoor_map(player_state.current_map) {
        day_night_tint.intensity = 0.0;
        day_night_tint.tint = (1.0, 1.0, 1.0);
        for mut bg in &mut overlay_query {
            *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0));
        }
        return;
    }

    // Compute time as float
    let time = calendar.time_float();

    // Sample the keyframes
    let (tint, intensity) = sample_tint(time);

    // Update the shared resource
    day_night_tint.intensity = intensity;
    day_night_tint.tint = tint;

    // Check if a lightning flash is overriding the overlay
    if let Some(ref flash) = lightning {
        if flash.flashing && flash.flash_fade_remaining > 0.0 {
            // Flash: bright white overlay. Alpha proportional to remaining fade time.
            let flash_alpha = (flash.flash_fade_remaining / 0.3).clamp(0.0, 1.0) * 0.7;
            for mut bg in &mut overlay_query {
                *bg = BackgroundColor(Color::srgba(1.0, 1.0, 1.0, flash_alpha));
            }
            return;
        }
    }

    // Convert the tint + intensity into an overlay color.
    // The tint color represents what the scene should look like.
    // We want the overlay to push the scene towards that tint.
    //
    // Strategy: the overlay is a color whose RGB is the inverse-brightness
    // component of the tint, and alpha is the intensity.
    //
    // For night (tint 0.3, 0.3, 0.5, intensity 0.5):
    //   overlay = dark blue-ish at alpha 0.5 → darkens and tints scene blue
    //
    // For daylight (tint 1.0, 1.0, 1.0, intensity 0.0):
    //   overlay = anything at alpha 0.0 → invisible
    //
    // We invert the tint so that bright tint → dark overlay color (less visible
    // at low alpha anyway) and dark tint → dark overlay color (very visible at
    // high alpha). Specifically we use overlay_rgb = 1.0 - tint, then we add
    // a subtle bias towards the tint's hue.
    //
    // Actually a simpler approach: use the tint color directly at the given alpha
    // but make it a darkening overlay. The overlay darkens the scene, so we want:
    //   overlay_r = 0 when tint_r = 1.0 (no darkening)
    //   overlay_r = 1.0 - tint_r when tint_r < 1.0 (darkening amount)
    //
    // Wait, that also isn't right for a simple BackgroundColor overlay in Bevy UI
    // which composites on top using standard alpha blending (src_over).
    //
    // With src_over: result = overlay * alpha + scene * (1 - alpha)
    // We want: at night, the scene looks like it's multiplied by tint color.
    // So we want: result ≈ scene * tint
    // With overlay approach: result = overlay_rgb * alpha + scene * (1 - alpha)
    //
    // Best approximation: use a dark overlay (RGB close to 0) with the tint
    // influencing the exact hue. At alpha = intensity, this darkens the scene.
    // For tint hue, we shift the overlay color slightly towards the complement.
    //
    // For simplicity and good visual results:
    //   overlay_rgb = tint * 0.15  (very dark version of the tint color)
    //   overlay_alpha = intensity
    //
    // Night example: tint=(0.3,0.3,0.5) intensity=0.5
    //   overlay = rgba(0.045, 0.045, 0.075, 0.5) — dark blue-ish at 50% opacity
    //
    // This effectively darkens the scene and pushes it towards dark blue.
    let overlay_r = tint.0 * 0.15;
    let overlay_g = tint.1 * 0.15;
    let overlay_b = tint.2 * 0.15;
    let overlay_a = intensity;

    for mut bg in &mut overlay_query {
        *bg = BackgroundColor(Color::srgba(overlay_r, overlay_g, overlay_b, overlay_a));
    }
}

/// Update lightning flash state during stormy weather.
pub fn update_lightning_flash(
    time: Res<Time>,
    calendar: Res<Calendar>,
    mut lightning: ResMut<LightningFlash>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // Only flash during storms
    if calendar.weather != Weather::Stormy {
        lightning.flashing = false;
        lightning.flash_fade_remaining = 0.0;
        lightning.thunder_delay = None;
        // Reset timer to a reasonable interval for when storm starts
        lightning.next_flash_timer = Timer::from_seconds(10.0, TimerMode::Once);
        return;
    }

    let dt = time.delta_secs();

    // Handle active flash fade-out
    if lightning.flashing {
        lightning.flash_fade_remaining -= dt;
        if lightning.flash_fade_remaining <= 0.0 {
            lightning.flashing = false;
            lightning.flash_fade_remaining = 0.0;
        }
    }

    // Handle pending thunder sound
    if let Some(ref mut delay) = lightning.thunder_delay {
        *delay -= dt;
        if *delay <= 0.0 {
            sfx_events.send(PlaySfxEvent {
                sfx_id: "thunder".to_string(),
            });
            lightning.thunder_delay = None;
        }
    }

    // Tick the next-flash timer
    lightning.next_flash_timer.tick(time.delta());
    if lightning.next_flash_timer.finished() {
        // Trigger a new flash
        lightning.flashing = true;
        lightning.flash_fade_remaining = 0.3;

        // Schedule thunder 0.5-2.0 seconds after the flash
        use rand::Rng;
        let mut rng = rand::thread_rng();
        lightning.thunder_delay = Some(rng.gen_range(0.5..2.0));

        // Schedule next flash in 8-15 seconds
        let next_interval = rng.gen_range(8.0..15.0);
        lightning.next_flash_timer = Timer::from_seconds(next_interval, TimerMode::Once);
    }
}
