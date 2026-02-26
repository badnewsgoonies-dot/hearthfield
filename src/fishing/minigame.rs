//! Fishing minigame mechanics.
//!
//! The minigame is a vertical skill challenge:
//!
//!  ┌──────────────┐
//!  │  [  fish  ]  │  ← fish zone: moves erratically based on difficulty
//!  │              │
//!  │  [ catch  ]  │  ← catch bar: player holds Space to rise, releases to fall
//!  └──────────────┘
//!  [████░░░░░░░░░░]  ← progress bar: 0% to 100%
//!
//! Progress fills while overlapping, drains while not. Reach 100% = caught!
//! Drain to 0% = escaped.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;
use super::{
    FishingPhase, FishingState, FishingMinigameState,
    MinigameFishZone, MinigameCatchBar, MinigameProgressFill,
};
use super::resolve::{catch_fish, end_fishing_escape};
use super::Bobber;

// ─── Tuning constants ─────────────────────────────────────────────────────────

/// How fast the catch bar rises when Space is held (units/second, 0-100 scale).
const CATCH_RISE_SPEED: f32 = 60.0;
/// How fast the catch bar falls when Space is released.
const CATCH_FALL_SPEED: f32 = 45.0;

/// How fast the progress fills when overlapping (% per second).
const PROGRESS_FILL_RATE: f32 = 20.0;
/// How fast the progress drains when not overlapping (% per second).
const PROGRESS_DRAIN_RATE: f32 = 15.0;

/// Maximum speed of the fish zone (units/second at difficulty 1.0).
const FISH_MAX_SPEED: f32 = 90.0;
/// Minimum speed (at difficulty 0.0).
const FISH_MIN_SPEED: f32 = 20.0;

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
        let target = if rng.gen_bool(0.5) {
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
        minigame_state.direction_change_timer =
            Timer::from_seconds(next_interval, TimerMode::Once);
    }

    // Apply velocity to fish zone center
    let speed = minigame_state.fish_zone_velocity;
    minigame_state.fish_zone_center =
        (minigame_state.fish_zone_center + speed * dt).clamp(
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
    for mut transform in fish_zone_query.iter_mut() {
        let y = zone_to_screen_y(minigame_state.fish_zone_center);
        transform.translation.y = y;
    }
}

/// Update catch bar based on Space key input.
pub fn update_catch_bar(
    mut minigame_state: ResMut<FishingMinigameState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut catch_bar_query: Query<&mut Transform, With<MinigameCatchBar>>,
) {
    let dt = time.delta_secs();
    let space_held = keyboard.pressed(KeyCode::Space);
    minigame_state.space_held = space_held;

    if space_held {
        minigame_state.catch_bar_center =
            (minigame_state.catch_bar_center + CATCH_RISE_SPEED * dt).clamp(
                minigame_state.catch_bar_half,
                100.0 - minigame_state.catch_bar_half,
            );
    } else {
        minigame_state.catch_bar_center =
            (minigame_state.catch_bar_center - CATCH_FALL_SPEED * dt).clamp(
                minigame_state.catch_bar_half,
                100.0 - minigame_state.catch_bar_half,
            );
    }

    // Update sprite
    for mut transform in catch_bar_query.iter_mut() {
        let y = zone_to_screen_y(minigame_state.catch_bar_center);
        transform.translation.y = y;
    }
}

/// Update progress bar and check for completion / failure.
pub fn update_progress(
    mut minigame_state: ResMut<FishingMinigameState>,
    time: Res<Time>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut progress_fill_query: Query<&mut Transform, With<MinigameProgressFill>>,
) {
    let dt = time.delta_secs();

    if minigame_state.is_overlapping() {
        minigame_state.progress =
            (minigame_state.progress + PROGRESS_FILL_RATE * dt).clamp(0.0, 100.0);

        // Tick overlap SFX cooldown
        minigame_state.overlap_sfx_cooldown -= dt;
        if minigame_state.overlap_sfx_cooldown <= 0.0 {
            sfx_events.write(PlaySfxEvent {
                sfx_id: "fishing_overlap_tick".to_string(),
            });
            minigame_state.overlap_sfx_cooldown = 0.3;
        }
    } else {
        minigame_state.progress =
            (minigame_state.progress - PROGRESS_DRAIN_RATE * dt).clamp(0.0, 100.0);
    }

    // Update progress fill bar width
    // The progress fill transform's x-scale represents filled fraction
    for mut transform in progress_fill_query.iter_mut() {
        // Full width = 100 screen units; scale x from 0.0 to 1.0
        let fraction = minigame_state.progress / 100.0;
        transform.scale.x = fraction.max(0.001); // avoid zero scale artifacts
    }
}

/// Check whether the minigame is won, lost, or cancelled.
pub fn check_minigame_result(
    fishing_state: ResMut<FishingState>,
    minigame_state: Res<FishingMinigameState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut item_pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    fish_registry: Res<FishRegistry>,
    bobber_query: Query<Entity, With<Bobber>>,
    mut commands: Commands,
) {
    // Win condition
    if minigame_state.progress >= 100.0 {
        let fish_id = fishing_state.selected_fish_id.clone();
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        catch_fish(
            fishing_state,
            minigame_state,
            &mut next_state,
            &mut stamina_events,
            &mut item_pickup_events,
            &mut sfx_events,
            &fish_registry,
            &mut commands,
            bobber_entities,
        );
        return;
    }

    // Loss condition
    if minigame_state.progress <= 0.0 && minigame_state.elapsed > 0.5 {
        sfx_events.write(PlaySfxEvent {
            sfx_id: "fish_escape".to_string(),
        });
        let mut fishing_state_mut = fishing_state;
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        end_fishing_escape(
            &mut fishing_state_mut,
            &mut next_state,
            &mut stamina_events,
            &mut commands,
            bobber_entities,
            true, // coming from Fishing state
        );
        return;
    }

    // Cancel condition (Escape key during minigame)
    if keyboard.just_pressed(KeyCode::Escape) {
        sfx_events.write(PlaySfxEvent {
            sfx_id: "fish_escape".to_string(),
        });
        let mut fishing_state_mut = fishing_state;
        let bobber_entities: Vec<Entity> = bobber_query.iter().collect();
        end_fishing_escape(
            &mut fishing_state_mut,
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
/// The bar occupies MINIGAME_BAR_HEIGHT screen pixels, centered on screen.
pub(super) fn zone_to_screen_y(zone: f32) -> f32 {
    let bar_bottom = -MINIGAME_BAR_HEIGHT / 2.0;
    bar_bottom + (zone / 100.0) * MINIGAME_BAR_HEIGHT
}

pub(super) const MINIGAME_BAR_HEIGHT: f32 = 200.0;
pub(super) const MINIGAME_BAR_WIDTH: f32 = 40.0;
pub(super) const MINIGAME_BAR_X: f32 = 160.0; // right side of screen (screen coords)
pub(super) const PROGRESS_BAR_Y: f32 = -130.0;
pub(super) const PROGRESS_BAR_WIDTH: f32 = 120.0;
pub(super) const PROGRESS_BAR_HEIGHT: f32 = 12.0;
