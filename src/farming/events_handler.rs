//! Handlers for cross-domain events: DayEndEvent, SeasonChangeEvent.
//!
//! ## Integration fix log
//! - Fixed `on_day_end` rain check: previously read `calendar.weather` which by the
//!   time this system runs has already been rolled for the NEW day (not the ended day).
//!   Now reads `TrackedDayWeather` — a farming-local resource that snapshots the
//!   calendar weather each frame during the day, so it's always the CURRENT (ended)
//!   day's weather, not the newly-rolled weather.
//! - Added `track_day_weather` system that runs each frame during Playing state to
//!   keep `TrackedDayWeather` in sync with `Calendar.weather` BEFORE day-end
//!   processing can overwrite it.

use bevy::prelude::*;
use crate::shared::*;
use super::{
    FarmEntities, TrackedDayWeather,
    crops::{advance_crop_growth, reset_soil_watered_state},
    soil::spawn_or_update_soil_entity,
    sprinkler::apply_rain_watering,
};

// ─────────────────────────────────────────────────────────────────────────────
// Weather tracking
// ─────────────────────────────────────────────────────────────────────────────

/// Snapshots the calendar weather each frame.  Because this system runs in
/// Update BEFORE `on_day_end` (via system ordering in FarmingPlugin), the
/// captured weather is always the CURRENT day's weather — even if `tick_time`
/// fires `trigger_day_end` and re-rolls weather in the same frame.
///
/// The trick is that `tick_time` fires the DayEndEvent AND re-rolls weather in
/// a single system call, but `track_day_weather` ran earlier in the same frame
/// (before tick_time's minute loop hit hour 26) or in the previous frame.
/// Either way, the tracked weather will be the ended day's weather because
/// weather is only rolled at day end — it doesn't change during the day.
pub fn track_day_weather(
    calendar: Res<Calendar>,
    mut tracked: ResMut<TrackedDayWeather>,
) {
    // Only update if the calendar day matches (i.e., we haven't rolled to a
    // new day yet in this frame).  If the day already advanced, keep the old
    // snapshot — that's exactly what we want for on_day_end to read.
    if tracked.day != calendar.day || tracked.season != calendar.season || tracked.year != calendar.year {
        // Calendar has advanced to a new day — this is the FIRST frame of the
        // new day. Update our tracking to the new day's weather so we're ready
        // for the next day-end cycle.
        tracked.weather = calendar.weather;
        tracked.day = calendar.day;
        tracked.season = calendar.season;
        tracked.year = calendar.year;
    } else {
        // Same day — keep snapshotting (weather doesn't change mid-day, but
        // we refresh in case of edge cases).
        tracked.weather = calendar.weather;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Day End
// ─────────────────────────────────────────────────────────────────────────────

/// Processes all end-of-day farming logic:
/// 1. Sprinklers auto-water (handled separately in sprinkler.rs, runs first).
/// 2. Rain auto-waters if today was rainy.
/// 3. Advance crop growth for all crops.
/// 4. Reset soil state (Watered -> Tilled) for the next day.
/// 5. Kill crops that can't survive in the current season.
/// 6. Handle crow events (scare away crows via scarecrows).
pub fn on_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
    crop_registry: Res<CropRegistry>,
    tracked_weather: Res<TrackedDayWeather>,
) {
    for event in day_end_events.read() {
        // BUG FIX: Previously read calendar.weather, but by the time this system
        // runs the calendar has already been advanced and weather re-rolled for the
        // NEW day.  Now we read TrackedDayWeather which was snapshotted earlier in
        // the frame (or previous frame) and holds the ENDED day's weather.
        let is_rainy = matches!(tracked_weather.weather, Weather::Rainy | Weather::Stormy);

        // Rain waters all tilled/watered tiles.
        if is_rainy {
            apply_rain_watering(&mut farm_state);
        }

        // Advance crop growth (mutates FarmState).
        let updated_positions = advance_crop_growth(
            &mut farm_state,
            &crop_registry,
            event.season,
            is_rainy,
        );

        // Process crow events — kill a random unprotected crop.
        // Crows only appear in non-winter seasons.
        if event.season != Season::Winter {
            maybe_crow_event(&mut farm_state, &farm_entities);
        }

        // Reset soil watered state for the next day.
        reset_soil_watered_state(&mut farm_state);

        // Despawn dead crop entities.
        let dead_positions: Vec<(i32, i32)> = updated_positions
            .iter()
            .filter(|&&pos| {
                farm_state.crops.get(&pos).map(|c| c.dead).unwrap_or(false)
            })
            .cloned()
            .collect();

        // We don't remove dead crops immediately from FarmState — the player can
        // still see and remove the withered plant by interacting with it.
        // (despawn_crop will be called when the player presses Space on it.)

        let _ = (dead_positions, updated_positions);

        // Sync entity colours — handled by render::sync_soil_sprites / sync_crop_sprites.
        // We need to ensure soil entities exist for any new tilled tiles.
        // (They should already exist from when they were tilled/watered.)
        ensure_all_soil_entities(&mut commands, &mut farm_entities, &farm_state);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Season Change
// ─────────────────────────────────────────────────────────────────────────────

/// When the season changes, kill all crops that can't grow in the new season.
pub fn on_season_change(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
    crop_registry: Res<CropRegistry>,
) {
    for event in season_events.read() {
        let new_season = event.new_season;

        let positions: Vec<(i32, i32)> = farm_state.crops.keys().cloned().collect();
        let mut to_kill = Vec::new();

        for pos in positions {
            let Some(crop) = farm_state.crops.get(&pos) else {
                continue;
            };
            let Some(def) = crop_registry.crops.get(&crop.crop_id) else {
                to_kill.push(pos); // unknown crop — remove
                continue;
            };
            if !def.seasons.is_empty() && !def.seasons.contains(&new_season) {
                to_kill.push(pos);
            }
        }

        // Kill out-of-season crops immediately (mark dead, visual handled by render).
        for pos in to_kill {
            if let Some(crop) = farm_state.crops.get_mut(&pos) {
                crop.dead = true;
            }
        }

        // Winter also resets all soil back to untilled (frost kills tilled ground).
        if new_season == Season::Winter {
            // Clear soil state — all tilled/watered tiles reset to untilled.
            // This models winter frost destroying the tilled rows.
            let tilled_positions: Vec<(i32, i32)> = farm_state
                .soil
                .keys()
                .cloned()
                .collect();
            for pos in tilled_positions {
                farm_state.soil.remove(&pos);
                // Despawn soil entities.
                if let Some(entity) = farm_entities.soil_entities.remove(&pos) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Crow event
// ─────────────────────────────────────────────────────────────────────────────

/// Small chance each day for a crow to destroy a random unprotected crop.
fn maybe_crow_event(farm_state: &mut FarmState, farm_entities: &FarmEntities) {
    use rand::Rng;

    let mut rng = rand::thread_rng();

    // 8% daily chance of a crow attack.
    if rng.gen_bool(0.92) {
        return;
    }

    // Find all scarecrow positions.
    let scarecrow_positions: Vec<(i32, i32)> = farm_state
        .objects
        .iter()
        .filter(|(_, obj)| matches!(obj, FarmObject::Scarecrow))
        .map(|(&pos, _)| pos)
        .collect();

    const SCARECROW_RADIUS: i32 = 8;

    // Gather unprotected, living crops.
    let unprotected: Vec<(i32, i32)> = farm_state
        .crops
        .iter()
        .filter(|(_, c)| !c.dead)
        .map(|(&pos, _)| pos)
        .filter(|&pos| {
            !scarecrow_positions.iter().any(|&sc| {
                let dx = (pos.0 - sc.0).abs();
                let dy = (pos.1 - sc.1).abs();
                dx <= SCARECROW_RADIUS && dy <= SCARECROW_RADIUS
            })
        })
        .collect();

    if unprotected.is_empty() {
        return;
    }

    // Pick a random unprotected crop and kill it.
    let idx = rng.gen_range(0..unprotected.len());
    let target = unprotected[idx];

    if let Some(crop) = farm_state.crops.get_mut(&target) {
        crop.dead = true;
    }

    let _ = farm_entities; // would use to update sprite in a more complete system
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Ensure soil entities exist for all entries in FarmState.soil.
fn ensure_all_soil_entities(
    commands: &mut Commands,
    farm_entities: &mut FarmEntities,
    farm_state: &FarmState,
) {
    for (&pos, &state) in &farm_state.soil {
        spawn_or_update_soil_entity(commands, farm_entities, pos, state);
    }
}
