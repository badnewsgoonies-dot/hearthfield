use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Update overlap tracking and progress bar fill.
///
/// Uses a 12-second timer. The progress bar shows current overlap ratio
/// relative to elapsed time. Catch requires 68% overlap when the timer expires.
pub fn update_hud_system(
    mut minigame_state: ResMut<FishingMinigameState>,
    time: Res<Time>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut progress_fill_query: Query<&mut Transform, With<MinigameProgressFill>>,
) {
    let dt = time.delta_secs();

    // Only accumulate timing after the first 0.75s grace period,
    // so the initial bar-placement isn't counted against the player.
    if minigame_state.elapsed > 0.75 {
        minigame_state.minigame_total_time += dt;
    }

    if minigame_state.is_overlapping() {
        // Track how long the bar was overlapping (for catch calculation).
        if minigame_state.elapsed > 0.75 {
            minigame_state.overlap_time_total += dt;
        }

        // Overlap SFX — pulsed to avoid spam
        minigame_state.overlap_sfx_cooldown -= dt;
        if minigame_state.overlap_sfx_cooldown <= 0.0 {
            sfx_events.send(PlaySfxEvent {
                sfx_id: "fishing_overlap_tick".to_string(),
            });
            minigame_state.overlap_sfx_cooldown = 0.3;
        }
    }

    // Progress bar shows current overlap ratio (0-100%)
    let effective_time = minigame_state.minigame_total_time;
    let ratio = if effective_time > 0.1 {
        (minigame_state.overlap_time_total / effective_time).clamp(0.0, 1.0)
    } else {
        0.5 // Show 50% during grace period
    };
    minigame_state.progress = ratio * 100.0;

    // Update progress fill bar x-scale
    let fraction = ratio;
    for mut transform in progress_fill_query.iter_mut() {
        transform.scale.x = fraction.max(0.001);
    }
}


