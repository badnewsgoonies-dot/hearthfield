use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Accumulates real delta-seconds and converts them to in-game minutes.
///
/// Default time_scale = 1/6, meaning 1 real minute = 10 game-minutes.
/// One game-minute triggers when:
///     elapsed_real_seconds >= (1.0 / time_scale)
/// At default that's every 6 real seconds = 1 game-minute.
///
/// Day spans 6:00 AM → 26:00 (2:00 AM next day) = 20 game-hours = 1200 min.
/// At time_scale 1/6 that's 7200 real seconds (120 real minutes) per game-day.
pub fn record_tick_system(
    time: Res<Time>,
    mut calendar: ResMut<Calendar>,
    mut day_end_writer: EventWriter<DayEndEvent>,
    mut prev_weather: ResMut<PreviousDayWeather>,
    mut cutscene_queue: ResMut<CutsceneQueue>,
) {
    let delta = time.delta_secs();
    calendar.elapsed_real_seconds += delta;

    // How many real seconds equal one game-minute?
    let secs_per_game_minute = if calendar.time_scale > 0.0 {
        1.0 / calendar.time_scale
    } else {
        1.0 / (1.0 / 6.0)
    };

    // Record state before advancing so we can detect auto-2AM rollover.
    let day_before = calendar.day;
    let season_before = calendar.season;

    // Advance as many game-minutes as have accumulated
    while calendar.elapsed_real_seconds >= secs_per_game_minute {
        calendar.elapsed_real_seconds -= secs_per_game_minute;
        advance_one_minute(&mut calendar, &mut day_end_writer, &mut prev_weather);
    }

    // If the day changed during this tick (auto 2AM rollover), build a
    // cutscene transition so the player sees a day card instead of the
    // calendar silently advancing.
    if calendar.day != day_before && !cutscene_queue.active {
        let day_label = format!(
            "Day {} - {:?}, Year {}",
            calendar.day, calendar.season, calendar.year
        );
        let mut steps = std::collections::VecDeque::new();
        steps.push_back(CutsceneStep::FadeOut(1.4));
        steps.push_back(CutsceneStep::Wait(0.9));

        if calendar.season != season_before {
            steps.push_back(CutsceneStep::PlayBgm(
                match calendar.season {
                    Season::Spring => "spring",
                    Season::Summer => "summer",
                    Season::Fall => "fall",
                    Season::Winter => "winter",
                }
                .to_string(),
            ));
            steps.push_back(CutsceneStep::ShowText(
                format!("The first morning of {:?}.", calendar.season),
                4.0,
            ));
        }

        steps.push_back(CutsceneStep::ShowText(day_label, 3.0));
        steps.push_back(CutsceneStep::FadeIn(2.0));

        cutscene_queue.steps = steps;
        // Don't activate or change state here — same rationale as
        // trigger_sleep. activate_pending_cutscene handles this in
        // PostUpdate after all DayEndEvent readers have run.
    }
}


