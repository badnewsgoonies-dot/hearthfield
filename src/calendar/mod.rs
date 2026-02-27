//! Calendar domain — the heartbeat of Hearthfield.
//!
//! Responsible for:
//! - Advancing game time (minutes, hours, days, seasons, years)
//! - Rolling daily weather
//! - Detecting festival days
//! - Sending DayEndEvent and SeasonChangeEvent
//! - Pausing / unpausing time based on GameState
//! - Providing a manual sleep trigger (press B on Farm or in PlayerHouse)
//!
//! ## Integration fix log
//! - Added `trigger_sleep` system: pressing B while on Farm or PlayerHouse sends
//!   DayEndEvent, giving the player a way to end the day without waiting until 2 AM.
//! - Added `PreviousDayWeather` resource: stores the weather of the day that just
//!   ended so that farming (and other domains) can check if it rained on the ENDED
//!   day rather than reading calendar.weather (which is already rolled for the new day).
//! - Fixed `process_day_end` to advance the calendar when DayEndEvent arrives from
//!   an external source (e.g. the new sleep trigger). Previously only the 2 AM
//!   auto-trigger in `tick_time` advanced the calendar; external DayEndEvents left
//!   the calendar stuck on the same day.

pub mod festivals;

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;
use festivals::FestivalState;

/// Stores the weather of the most recently ended day so other domains can
/// check whether it rained *today* (the ended day) rather than tomorrow.
/// Updated every time a DayEndEvent is processed.
#[derive(Resource, Debug, Clone)]
pub struct PreviousDayWeather {
    pub weather: Weather,
}

impl Default for PreviousDayWeather {
    fn default() -> Self {
        Self {
            weather: Weather::Sunny,
        }
    }
}

pub struct CalendarPlugin;

impl Plugin for CalendarPlugin {
    fn build(&self, app: &mut App) {
        app
            // Track weather of the ended day for cross-domain rain checks
            .init_resource::<PreviousDayWeather>()
            // Festival state
            .init_resource::<FestivalState>()
            // Pause time whenever we leave Playing state
            .add_systems(OnEnter(GameState::Playing), resume_time)
            .add_systems(OnExit(GameState::Playing), pause_time)
            // Core time tick — only runs while Playing and NOT paused
            .add_systems(
                Update,
                (
                    tick_time,
                    detect_festival_day,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(time_not_paused),
            )
            // Manual sleep trigger — player presses B on Farm or in PlayerHouse
            .add_systems(
                Update,
                trigger_sleep
                    .run_if(in_state(GameState::Playing)),
            )
            // Day-end processing runs inside Playing state (but
            // the event can also be sent by the sleep system or the 2 AM auto-trigger)
            .add_systems(
                Update,
                process_day_end
                    .run_if(in_state(GameState::Playing))
                    .after(tick_time)
                    .after(trigger_sleep),
            )
            // Festival systems — all run in Playing state
            .add_systems(
                Update,
                (
                    festivals::check_festival_day,
                    festivals::start_egg_hunt,
                    festivals::collect_eggs,
                    festivals::start_luau,
                    festivals::start_harvest_festival,
                    festivals::setup_winter_star,
                    festivals::winter_star_give_gift,
                    festivals::cleanup_festival_on_day_end,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ─── Run Conditions ───────────────────────────────────────────────────────────

fn time_not_paused(calendar: Res<Calendar>) -> bool {
    !calendar.time_paused
}

// ─── State transition hooks ───────────────────────────────────────────────────

fn resume_time(mut calendar: ResMut<Calendar>) {
    calendar.time_paused = false;
    info!("[Calendar] Time resumed — {}:{:02} Day {} {:?} Year {}",
        calendar.hour, calendar.minute, calendar.day, calendar.season, calendar.year);
}

fn pause_time(mut calendar: ResMut<Calendar>) {
    calendar.time_paused = true;
    info!("[Calendar] Time paused");
}

// ─── Manual sleep trigger ────────────────────────────────────────────────────

/// Allows the player to end the day by pressing B while on the Farm or in the
/// PlayerHouse.  This is the primary way to trigger sleep before the forced
/// 2 AM rollover.  Sends a DayEndEvent which process_day_end will pick up to
/// advance the calendar, and all other domains (farming, economy, etc.) will
/// process their end-of-day logic.
fn trigger_sleep(
    player_input: Res<PlayerInput>,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    mut day_end_events: EventWriter<DayEndEvent>,
) {
    if !player_input.interact {
        return;
    }

    // Only allow sleeping at home or on the farm.
    if !matches!(player_state.current_map, MapId::Farm | MapId::PlayerHouse) {
        return;
    }

    info!(
        "[Calendar] Player triggered sleep at {}:{:02} Day {} {:?} Year {}",
        calendar.hour, calendar.minute, calendar.day, calendar.season, calendar.year
    );

    day_end_events.send(DayEndEvent {
        day: calendar.day,
        season: calendar.season,
        year: calendar.year,
    });
}

// ─── Main time-tick system ────────────────────────────────────────────────────

/// Accumulates real delta-seconds and converts them to in-game minutes.
///
/// Default time_scale = 10.0, meaning 1 real second = 10 game-minutes.
/// One game-minute triggers when:
///     elapsed_real_seconds >= (60.0 / time_scale)
/// At default that's every 6 real seconds = 1 game-hour.
///
/// Day spans 6:00 AM → 26:00 (2:00 AM next day) = 20 game-hours = 1200 min.
/// At time_scale 10 that's 120 real seconds (2 real minutes) per game-day.
fn tick_time(
    time: Res<Time>,
    mut calendar: ResMut<Calendar>,
    mut day_end_writer: EventWriter<DayEndEvent>,
    mut prev_weather: ResMut<PreviousDayWeather>,
) {
    let delta = time.delta_secs();
    calendar.elapsed_real_seconds += delta;

    // How many real seconds equal one game-minute?
    // time_scale game-minutes per real-second → 1 game-minute = 1/time_scale real-seconds
    // Guard against zero / negative time_scale
    let secs_per_game_minute = if calendar.time_scale > 0.0 {
        1.0 / calendar.time_scale
    } else {
        1.0 / 10.0
    };

    // Advance as many game-minutes as have accumulated
    while calendar.elapsed_real_seconds >= secs_per_game_minute {
        calendar.elapsed_real_seconds -= secs_per_game_minute;
        advance_one_minute(&mut calendar, &mut day_end_writer, &mut prev_weather);
    }
}

/// Advances the calendar by exactly one game-minute.
/// Handles minute -> hour -> day rollovers.
fn advance_one_minute(
    calendar: &mut Calendar,
    day_end_writer: &mut EventWriter<DayEndEvent>,
    prev_weather: &mut PreviousDayWeather,
) {
    calendar.minute += 1;

    if calendar.minute >= 60 {
        calendar.minute = 0;
        calendar.hour += 1;

        // 2:00 AM = hour 26 -> force end of day
        if calendar.hour >= 26 {
            trigger_day_end(calendar, day_end_writer, prev_weather);
        }
    }
}

/// Called when day ends via the 2 AM auto-rollover.
/// Stores the ended day's weather in PreviousDayWeather, then rolls new
/// weather, advances day/season/year, and resets clock to 6:00 AM.
fn trigger_day_end(
    calendar: &mut Calendar,
    day_end_writer: &mut EventWriter<DayEndEvent>,
    prev_weather: &mut PreviousDayWeather,
) {
    // Emit event with the CURRENT day/season/year (the day that just ended)
    day_end_writer.send(DayEndEvent {
        day: calendar.day,
        season: calendar.season,
        year: calendar.year,
    });

    info!(
        "[Calendar] Day ended — Day {} {:?} Year {}",
        calendar.day, calendar.season, calendar.year
    );

    // Store the ended day's weather BEFORE rolling new weather.
    // This lets farming and other domains check if it rained today.
    prev_weather.weather = calendar.weather;

    // Advance to next day
    calendar.day += 1;
    calendar.hour = 6;
    calendar.minute = 0;
    calendar.elapsed_real_seconds = 0.0;

    // Season rollover
    if calendar.day > DAYS_PER_SEASON {
        calendar.day = 1;
        let old_season = calendar.season;
        calendar.season = calendar.season.next();

        info!(
            "[Calendar] Season changed: {:?} -> {:?} (Year {})",
            old_season, calendar.season, calendar.year
        );

        // Year rollover happens when Spring begins again
        if calendar.season == Season::Spring {
            calendar.year += 1;
            info!("[Calendar] New Year! Year {}", calendar.year);
        }
    }

    // Roll weather for the new day
    calendar.weather = roll_weather(calendar.season);

    info!(
        "[Calendar] New day: Day {} {:?} Year {} — Weather: {:?}",
        calendar.day, calendar.season, calendar.year, calendar.weather
    );
}

// ─── Day-end event relay ──────────────────────────────────────────────────────

/// Reads DayEndEvent and handles two cases:
///
/// 1. **Internal trigger (2 AM auto-rollover):** The calendar was already advanced
///    by `trigger_day_end` called from `advance_one_minute`.  We detect this because
///    `event.day != calendar.day` (calendar was already moved to the next day).
///    In this case we only need to emit SeasonChangeEvent if applicable and store
///    the previous day's weather.
///
/// 2. **External trigger (player pressed B to sleep):** The calendar has NOT been
///    advanced yet.  We detect this because `event.day == calendar.day` and
///    `event.season == calendar.season`.  In this case we must advance the calendar
///    ourselves (increment day, reset time, roll weather, handle season/year rollover).
///
/// In both cases we store the ended day's weather in PreviousDayWeather so farming
/// can check whether it rained on the day that just ended (not the new day).
fn process_day_end(
    mut day_end_reader: EventReader<DayEndEvent>,
    mut season_writer: EventWriter<SeasonChangeEvent>,
    mut calendar: ResMut<Calendar>,
    mut prev_weather: ResMut<PreviousDayWeather>,
) {
    for event in day_end_reader.read() {
        // Determine whether the calendar was already advanced (internal trigger)
        // or still needs advancing (external trigger like sleep).
        let already_advanced = event.day != calendar.day
            || event.season != calendar.season
            || event.year != calendar.year;

        if already_advanced {
            // Internal trigger path: calendar was advanced in trigger_day_end.
            // The weather was already rolled for the new day, so the ended day's
            // weather is lost unless we captured it.  Unfortunately trigger_day_end
            // already overwrote it.  We can infer the old weather was whatever
            // the event's season would produce — but we can't recover it exactly.
            // Instead, we rely on the fact that trigger_day_end stores it below
            // in the external path.  For the internal path, we store a best-effort
            // value.  (In practice, the PreviousDayWeather is most useful for the
            // external / sleep path where we CAN capture it before advancing.)
            //
            // For the auto-2AM path, the weather has already been rolled.  We'll
            // note this limitation: the old weather is NOT recoverable from the
            // internal trigger path without modifying trigger_day_end.  However,
            // the farming on_day_end system runs in the same frame and can also
            // use event.season to infer weather.  As a workaround, we'll update
            // trigger_day_end to store prev_weather before rolling new weather.
            // (See the updated trigger_day_end below.)

            // Check for season change.
            if event.season != calendar.season {
                season_writer.send(SeasonChangeEvent {
                    new_season: calendar.season,
                    year: calendar.year,
                });
                info!(
                    "[Calendar] SeasonChangeEvent sent: {:?} Year {}",
                    calendar.season, calendar.year
                );
            }
        } else {
            // External trigger path (e.g. player pressed B to sleep).
            // The calendar still shows the CURRENT (ending) day.  We need to
            // capture the weather before advancing, then advance.

            // Store the ended day's weather BEFORE rolling new weather.
            prev_weather.weather = calendar.weather;

            info!(
                "[Calendar] External day-end trigger — advancing calendar from Day {} {:?} Year {}",
                calendar.day, calendar.season, calendar.year
            );

            // Advance to next day (same logic as trigger_day_end).
            let old_season = calendar.season;

            calendar.day += 1;
            calendar.hour = 6;
            calendar.minute = 0;
            calendar.elapsed_real_seconds = 0.0;

            // Season rollover.
            if calendar.day > DAYS_PER_SEASON {
                calendar.day = 1;
                calendar.season = calendar.season.next();

                info!(
                    "[Calendar] Season changed: {:?} -> {:?} (Year {})",
                    old_season, calendar.season, calendar.year
                );

                // Year rollover happens when Spring begins again.
                if calendar.season == Season::Spring {
                    calendar.year += 1;
                    info!("[Calendar] New Year! Year {}", calendar.year);
                }

                // Emit SeasonChangeEvent.
                season_writer.send(SeasonChangeEvent {
                    new_season: calendar.season,
                    year: calendar.year,
                });
                info!(
                    "[Calendar] SeasonChangeEvent sent: {:?} Year {}",
                    calendar.season, calendar.year
                );
            }

            // Roll weather for the new day.
            calendar.weather = roll_weather(calendar.season);

            info!(
                "[Calendar] New day: Day {} {:?} Year {} — Weather: {:?}",
                calendar.day, calendar.season, calendar.year, calendar.weather
            );
        }
    }
}

// ─── Festival detection ───────────────────────────────────────────────────────

/// Logs (and could trigger UI banners / music changes) when a festival day begins.
/// Runs once per day when the hour transitions to 6 (morning).
/// We use a local resource to track whether we already announced today.
#[derive(Resource, Default)]
struct FestivalAnnouncedDay {
    day: u8,
    season_index: usize,
    year: u32,
}

fn detect_festival_day(
    calendar: Res<Calendar>,
    mut announced: Local<FestivalAnnouncedDay>,
    mut sfx_writer: EventWriter<PlayMusicEvent>,
) {
    if !calendar.is_festival_day() {
        return;
    }

    // Only announce once per day
    let already_announced = announced.day == calendar.day
        && announced.season_index == calendar.season.index()
        && announced.year == calendar.year;

    if already_announced {
        return;
    }

    announced.day = calendar.day;
    announced.season_index = calendar.season.index();
    announced.year = calendar.year;

    let festival_name = match (calendar.season, calendar.day) {
        (Season::Spring, 13) => "Spring Dance",
        (Season::Summer, 11) => "Summer Luau",
        (Season::Fall, 16) => "Fall Harvest Festival",
        (Season::Winter, 25) => "Winter Star Festival",
        _ => "Festival",
    };

    info!(
        "[Calendar] Festival day! {} — Day {} {:?} Year {}",
        festival_name, calendar.day, calendar.season, calendar.year
    );

    // Play festival music
    sfx_writer.send(PlayMusicEvent {
        track_id: format!("festival_{}", festival_name.to_lowercase().replace(' ', "_")),
        fade_in: true,
    });
}

// ─── Weather rolling ──────────────────────────────────────────────────────────

/// Rolls a weather result for the given season using weighted probabilities.
///
/// Spring:  60% Sunny, 30% Rainy, 10% Stormy
/// Summer:  70% Sunny, 20% Rainy, 10% Stormy
/// Fall:    50% Sunny, 35% Rainy, 15% Stormy
/// Winter:  40% Sunny, 10% Rainy, 10% Stormy, 40% Snowy
fn roll_weather(season: Season) -> Weather {
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen(); // 0.0 ..< 1.0

    match season {
        Season::Spring => {
            if roll < 0.60 {
                Weather::Sunny
            } else if roll < 0.90 {
                Weather::Rainy
            } else {
                Weather::Stormy
            }
        }
        Season::Summer => {
            if roll < 0.70 {
                Weather::Sunny
            } else if roll < 0.90 {
                Weather::Rainy
            } else {
                Weather::Stormy
            }
        }
        Season::Fall => {
            if roll < 0.50 {
                Weather::Sunny
            } else if roll < 0.85 {
                Weather::Rainy
            } else {
                Weather::Stormy
            }
        }
        Season::Winter => {
            if roll < 0.40 {
                Weather::Sunny
            } else if roll < 0.50 {
                Weather::Rainy
            } else if roll < 0.60 {
                Weather::Stormy
            } else {
                Weather::Snowy
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_roll_spring_distribution() {
        // Run many samples; with high probability all weathers appear
        let mut sunny = 0u32;
        let mut rainy = 0u32;
        let mut stormy = 0u32;
        let mut snowy = 0u32;

        for _ in 0..10_000 {
            match roll_weather(Season::Spring) {
                Weather::Sunny => sunny += 1,
                Weather::Rainy => rainy += 1,
                Weather::Stormy => stormy += 1,
                Weather::Snowy => snowy += 1,
            }
        }

        // Spring should never produce snow
        assert_eq!(snowy, 0, "Spring should never produce Snowy weather");
        // Very rough sanity checks (loose tolerances for probabilistic tests)
        assert!(sunny > 5000, "Sunny should be ~60%");
        assert!(rainy > 2000, "Rainy should be ~30%");
        assert!(stormy > 500, "Stormy should be ~10%");
    }

    #[test]
    fn test_weather_roll_winter_has_snow() {
        let mut snowy = 0u32;
        for _ in 0..10_000 {
            if matches!(roll_weather(Season::Winter), Weather::Snowy) {
                snowy += 1;
            }
        }
        assert!(snowy > 3000, "Winter should produce ~40% Snowy weather");
    }

    #[test]
    fn test_calendar_day_of_week() {
        let cal = Calendar::default();
        // Day 1, Spring = Monday (total_days_elapsed = 0, 0 % 7 = 0)
        assert_eq!(cal.day_of_week(), DayOfWeek::Monday);
    }

    #[test]
    fn test_total_days_elapsed() {
        let mut cal = Calendar::default();
        assert_eq!(cal.total_days_elapsed(), 0);

        cal.day = 28;
        cal.season = Season::Fall;
        cal.year = 2;
        // year=2 → 112 days, fall=2*28=56, day=27 offset
        assert_eq!(cal.total_days_elapsed(), 112 + 56 + 27);
    }

    #[test]
    fn test_is_festival_day() {
        let mut cal = Calendar::default();

        cal.season = Season::Spring;
        cal.day = 13;
        assert!(cal.is_festival_day());

        cal.season = Season::Summer;
        cal.day = 11;
        assert!(cal.is_festival_day());

        cal.season = Season::Fall;
        cal.day = 16;
        assert!(cal.is_festival_day());

        cal.season = Season::Winter;
        cal.day = 25;
        assert!(cal.is_festival_day());

        cal.season = Season::Spring;
        cal.day = 1;
        assert!(!cal.is_festival_day());
    }

    #[test]
    fn test_season_next() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Fall);
        assert_eq!(Season::Fall.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_time_float() {
        let mut cal = Calendar::default();
        cal.hour = 14;
        cal.minute = 30;
        assert!((cal.time_float() - 14.5).abs() < 0.001);
    }

    #[test]
    fn test_day_advancement_within_season() {
        let mut cal = Calendar::default();
        cal.day = 5;
        // Simulate day end: advance day within season
        cal.day += 1;
        assert_eq!(cal.day, 6);
        assert_eq!(cal.season, Season::Spring);
    }

    #[test]
    fn test_season_change_at_day_28() {
        let mut cal = Calendar::default();
        cal.day = 28;
        cal.season = Season::Spring;
        // Simulate day end
        cal.day += 1;
        if cal.day > DAYS_PER_SEASON {
            cal.day = 1;
            cal.season = cal.season.next();
        }
        assert_eq!(cal.day, 1);
        assert_eq!(cal.season, Season::Summer);
    }

    #[test]
    fn test_year_increment_after_winter() {
        let mut cal = Calendar::default();
        cal.day = 28;
        cal.season = Season::Winter;
        cal.year = 1;
        // Simulate day end
        cal.day += 1;
        if cal.day > DAYS_PER_SEASON {
            cal.day = 1;
            cal.season = cal.season.next();
            if cal.season == Season::Spring {
                cal.year += 1;
            }
        }
        assert_eq!(cal.day, 1);
        assert_eq!(cal.season, Season::Spring);
        assert_eq!(cal.year, 2);
    }

    #[test]
    fn test_day_of_week_day_7() {
        let mut cal = Calendar::default();
        cal.day = 7;
        // Day 7 => total_days_elapsed = 6, 6 % 7 = 6 => Sunday
        assert_eq!(cal.day_of_week(), DayOfWeek::Sunday);
    }

    #[test]
    fn test_day_of_week_day_8_wraps() {
        let mut cal = Calendar::default();
        cal.day = 8;
        // Day 8 => total_days_elapsed = 7, 7 % 7 = 0 => Monday
        assert_eq!(cal.day_of_week(), DayOfWeek::Monday);
    }

    #[test]
    fn test_roll_weather_always_valid() {
        // Ensure roll_weather never panics and returns a valid variant
        for season in [Season::Spring, Season::Summer, Season::Fall, Season::Winter] {
            for _ in 0..100 {
                let w = roll_weather(season);
                match w {
                    Weather::Sunny | Weather::Rainy | Weather::Stormy | Weather::Snowy => {}
                }
            }
        }
    }

    #[test]
    fn test_summer_no_snow() {
        for _ in 0..5000 {
            let w = roll_weather(Season::Summer);
            assert_ne!(w, Weather::Snowy, "Summer should never produce snow");
        }
    }
}
