//! Calendar domain — the heartbeat of Hearthfield.
//!
//! Responsible for:
//! - Advancing game time (minutes, hours, days, seasons, years)
//! - Rolling daily weather
//! - Detecting festival days
//! - Sending DayEndEvent and SeasonChangeEvent
//! - Pausing / unpausing time based on GameState

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

pub struct CalendarPlugin;

impl Plugin for CalendarPlugin {
    fn build(&self, app: &mut App) {
        app
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
            // Day-end processing runs inside Playing state (but
            // the event can also be sent by the sleep system from other domains)
            .add_systems(
                Update,
                process_day_end
                    .run_if(in_state(GameState::Playing))
                    .after(tick_time),
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
        advance_one_minute(&mut calendar, &mut day_end_writer);
    }
}

/// Advances the calendar by exactly one game-minute.
/// Handles minute → hour → day rollovers.
fn advance_one_minute(
    calendar: &mut Calendar,
    day_end_writer: &mut EventWriter<DayEndEvent>,
) {
    calendar.minute += 1;

    if calendar.minute >= 60 {
        calendar.minute = 0;
        calendar.hour += 1;

        // 2:00 AM = hour 26 → force end of day
        if calendar.hour >= 26 {
            trigger_day_end(calendar, day_end_writer);
        }
    }
}

/// Called when day ends (either 2 AM or explicit sleep).
/// Rolls weather, advances day/season/year, resets clock to 6:00 AM.
fn trigger_day_end(
    calendar: &mut Calendar,
    day_end_writer: &mut EventWriter<DayEndEvent>,
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
            "[Calendar] Season changed: {:?} → {:?} (Year {})",
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

/// Reads DayEndEvent and re-emits SeasonChangeEvent when the season flips.
/// This is separate from trigger_day_end so that other domains (e.g. the sleep
/// system in the player domain) can send DayEndEvent and still get the season
/// change propagated correctly.
fn process_day_end(
    mut day_end_reader: EventReader<DayEndEvent>,
    mut season_writer: EventWriter<SeasonChangeEvent>,
    calendar: Res<Calendar>,
) {
    for event in day_end_reader.read() {
        // Check if a season change has occurred by comparing the event's season
        // to the current calendar season (which was already advanced in trigger_day_end
        // or will need advancing if sent externally).
        // Since trigger_day_end already advances the season in the Calendar resource,
        // we compare the event's season to the now-current calendar season.
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
}
