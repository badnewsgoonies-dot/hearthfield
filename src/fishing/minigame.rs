//! Fishing minigame mechanics.
//!
//! The minigame is a vertical skill challenge:
//!
//!  ┌──────────────┐
//!  │  [  fish  ]  │  ← fish zone: moves erratically based on difficulty
//!  │              │
//!  │  [ catch  ]  │  ← catch bar: player holds Space to rise, releases to fall
//!  └──────────────┘
//!  [████░░░░░░░░░░]  ← progress bar: overlap ratio visualization
//!
//! Timer-based mechanic: 10-second minigame. Player must keep the catch bar
//! overlapping the fish zone for at least 80% of the timer to succeed.
//!
//! # Perfect Catch
//! If the catch bar was inside the fish zone for 90%+ of the minigame duration,
//! the player gets a "Perfect catch!" toast and a quality upgrade notification.

use bevy::prelude::*;
use rand::Rng;

use super::resolve::{catch_fish, end_fishing_escape};
use super::Bobber;
use super::{
    FishEncyclopedia, FishingMinigameState, FishingState, MinigameCatchBar, MinigameFishZone,
    MinigameProgressFill,
};
use crate::shared::*;

// ─── Tuning constants ─────────────────────────────────────────────────────────

/// How fast the catch bar rises when Space is held (units/second, 0-100 scale).
const CATCH_RISE_SPEED: f32 = 60.0;
/// How fast the catch bar falls when Space is released.
const CATCH_FALL_SPEED: f32 = 45.0;

/// Maximum speed of the fish zone (units/second at difficulty 1.0).
const FISH_MAX_SPEED: f32 = 90.0;
/// Minimum speed (at difficulty 0.0).
const FISH_MIN_SPEED: f32 = 20.0;

/// Total minigame duration in seconds (spec: 10 seconds).
const MINIGAME_DURATION: f32 = 10.0;
/// Overlap ratio required to catch the fish (spec: 80%).
const CATCH_OVERLAP_THRESHOLD: f32 = 0.80;

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Update the fish zone position using erratic movement.
pub fn update_fish_zone(
    mut minigame_state: ResMut<FishingMinigameState>,
    time: Res<Time>,
    mut fish_zone_query: Query<&mut Transform, With<MinigameFishZone>>,
) {
    let dt = time.delta_secs();
    minigame_state.elapsed += dt;

    // Tick the direction change timer
    minigame_state.direction_change_timer.tick(time.delta());

    if minigame_state.direction_change_timer.just_finished() {
        let mut rng = rand::thread_rng();
        let difficulty = minigame_state.fish_difficulty;

        // Speed scales with difficulty
        let max_speed = FISH_MIN_SPEED + difficulty * (FISH_MAX_SPEED - FISH_MIN_SPEED);

        // Choose a target position biased away from center for harder fish
        let target: f32 = if rng.gen_bool(0.5) {
            rng.gen_range(15.0_f32..45.0_f32)
        } else {
            rng.gen_range(55.0_f32..85.0_f32)
        };

        // Velocity toward target, scaled by difficulty
        let direction = if target > minigame_state.fish_zone_center {
            1.0_f32
        } else {
            -1.0_f32
        };
        minigame_state.fish_zone_velocity = direction * max_speed;

        // Randomize next change interval: harder fish change more frequently
        let min_interval = 0.4 + (1.0 - difficulty) * 0.8;
        let max_interval = min_interval + 1.2 + (1.0 - difficulty) * 0.5;
        let next_interval = rng.gen_range(min_interval..max_interval);
        minigame_state.direction_change_timer = Timer::from_seconds(next_interval, TimerMode::Once);
    }

    // Apply velocity to fish zone center
    let speed = minigame_state.fish_zone_velocity;
    minigame_state.fish_zone_center = (minigame_state.fish_zone_center + speed * dt).clamp(
        minigame_state.fish_zone_half,
        100.0 - minigame_state.fish_zone_half,
    );

    // Bounce: reverse velocity at edges
    let lo = minigame_state.fish_zone_half;
    let hi = 100.0 - minigame_state.fish_zone_half;
    if minigame_state.fish_zone_center <= lo || minigame_state.fish_zone_center >= hi {
        minigame_state.fish_zone_velocity = -minigame_state.fish_zone_velocity * 0.7;
    }

    // Update sprite position
    let fish_center = minigame_state.fish_zone_center;
    for mut transform in fish_zone_query.iter_mut() {
        let y = zone_to_screen_y(fish_center);
        transform.translation.y = y;
    }
}

/// Update catch bar based on Space key input.
pub fn update_catch_bar(
    mut minigame_state: ResMut<FishingMinigameState>,
    player_input: Res<PlayerInput>,
    time: Res<Time>,
    mut catch_bar_query: Query<&mut Transform, With<MinigameCatchBar>>,
) {
    let dt = time.delta_secs();
    let space_held = player_input.fishing_reel;
    minigame_state.space_held = space_held;

    let catch_half = minigame_state.catch_bar_half;

    if space_held {
        minigame_state.catch_bar_center = (minigame_state.catch_bar_center + CATCH_RISE_SPEED * dt)
            .clamp(catch_half, 100.0 - catch_half);
    } else {
        // Lead Bobber reduces fall speed so the bar is easier to hold up.
        let effective_fall = CATCH_FALL_SPEED * minigame_state.catch_fall_multiplier;
        minigame_state.catch_bar_center = (minigame_state.catch_bar_center - effective_fall * dt)
            .clamp(catch_half, 100.0 - catch_half);
    }

    let catch_center = minigame_state.catch_bar_center;

    // Update sprite
    for mut transform in catch_bar_query.iter_mut() {
        let y = zone_to_screen_y(catch_center);
        transform.translation.y = y;
    }
}

/// Update overlap tracking and progress bar fill.
///
/// Uses a 10-second timer. The progress bar shows current overlap ratio
/// relative to elapsed time. Catch requires 80% overlap when the timer expires.
pub fn update_progress(
    mut minigame_state: ResMut<FishingMinigameState>,
    time: Res<Time>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut progress_fill_query: Query<&mut Transform, With<MinigameProgressFill>>,
) {
    let dt = time.delta_secs();

    // Only accumulate timing after the first 0.5s grace period,
    // so the initial bar-placement isn't counted against the player.
    if minigame_state.elapsed > 0.5 {
        minigame_state.minigame_total_time += dt;
    }

    if minigame_state.is_overlapping() {
        // Track how long the bar was overlapping (for catch calculation).
        if minigame_state.elapsed > 0.5 {
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

/// Check whether the minigame timer has expired, determine catch/fail, or cancel.
#[allow(clippy::too_many_arguments)]
pub fn check_minigame_result(
    mut fishing_state: ResMut<FishingState>,
    minigame_state: Res<FishingMinigameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut item_pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    player_input: Res<PlayerInput>,
    fish_registry: Res<FishRegistry>,
    calendar: Res<Calendar>,
    mut encyclopedia: ResMut<FishEncyclopedia>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut commands: Commands,
) {
    let timer_expired = minigame_state.minigame_total_time >= MINIGAME_DURATION;

    // Check overlap ratio
    let overlap_ratio = if minigame_state.minigame_total_time > 0.1 {
        minigame_state.overlap_time_total / minigame_state.minigame_total_time
    } else {
        0.0
    };

    if timer_expired {
        if overlap_ratio >= CATCH_OVERLAP_THRESHOLD {
            // Win: overlap >= 80% — caught the fish!
            let is_perfect = minigame_state.is_perfect_catch();
            let bait_id = fishing_state.bait_id.clone();
            let selected_fish = fishing_state.selected_fish_id.clone();

            let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
            catch_fish(
                &mut fishing_state,
                &mut next_state,
                &mut stamina_events,
                &mut item_pickup_events,
                &mut sfx_events,
                &fish_registry,
                &mut commands,
                bobber_entities,
                &mut encyclopedia,
                &calendar,
                &mut toast_events,
                &mut gold_events,
                bait_id.as_deref(),
            );

            // Perfect catch notification (after the normal catch is processed)
            if is_perfect {
                toast_events.send(ToastEvent {
                    message: "Perfect catch! Quality upgraded!".to_string(),
                    duration_secs: 3.5,
                });
                sfx_events.send(PlaySfxEvent {
                    sfx_id: "perfect_catch".to_string(),
                });
            }

            // Wild bait double-catch: 15% chance for a bonus fish (wild_bait only)
            if bait_id.as_deref() == Some("wild_bait")
                && super::cast::wild_bait_double_catch_roll()
            {
                if let Some(ref fid) = selected_fish {
                    item_pickup_events.send(ItemPickupEvent {
                        item_id: fid.clone(),
                        quantity: 1,
                    });
                    toast_events.send(ToastEvent {
                        message: "Wild Bait bonus: extra fish!".to_string(),
                        duration_secs: 2.5,
                    });
                }
            }
        } else {
            // Fail: timer expired but overlap < 80%
            sfx_events.send(PlaySfxEvent {
                sfx_id: "fish_escape".to_string(),
            });
            toast_events.send(ToastEvent {
                message: format!(
                    "The fish got away! ({:.0}% overlap, need {:.0}%)",
                    overlap_ratio * 100.0,
                    CATCH_OVERLAP_THRESHOLD * 100.0
                ),
                duration_secs: 2.5,
            });
            let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
            end_fishing_escape(
                &mut fishing_state,
                &mut next_state,
                &mut stamina_events,
                &mut commands,
                bobber_entities,
                true,
            );
        }
        return;
    }

    // Cancel with Escape key
    if player_input.ui_cancel {
        sfx_events.send(PlaySfxEvent {
            sfx_id: "fish_escape".to_string(),
        });
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        end_fishing_escape(
            &mut fishing_state,
            &mut next_state,
            &mut stamina_events,
            &mut commands,
            bobber_entities,
            true,
        );
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Convert a 0-100 zone position to a screen Y coordinate within the minigame bar.
/// The bar occupies MINIGAME_BAR_HEIGHT screen pixels, centered on the bar origin.
pub(super) fn zone_to_screen_y(zone: f32) -> f32 {
    let bar_bottom = -MINIGAME_BAR_HEIGHT / 2.0;
    bar_bottom + (zone / 100.0) * MINIGAME_BAR_HEIGHT
}

pub(super) const MINIGAME_BAR_HEIGHT: f32 = 200.0;
pub(super) const MINIGAME_BAR_WIDTH: f32 = 40.0;
pub(super) const PROGRESS_BAR_Y: f32 = -130.0;
pub(super) const PROGRESS_BAR_WIDTH: f32 = 120.0;
pub(super) const PROGRESS_BAR_HEIGHT: f32 = 12.0;
