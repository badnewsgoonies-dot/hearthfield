use bevy::prelude::*;
use rand::Rng;

use crate::shared::{
    DayOfWeek, GameState, Rank, ShiftClock, ShiftEndEvent, ShiftType, UpdatePhase, Weather,
    SHIFT_DURATION_HOURS, TIME_SCALE,
};

pub struct CalendarPlugin;

impl Plugin for CalendarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_clock,
                check_shift_end,
                handle_new_day_weather,
                check_rank_progression,
            )
                .chain()
                .in_set(UpdatePhase::Simulation)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

pub fn tick_clock(time: Res<Time>, mut clock: ResMut<ShiftClock>) {
    clock.time_scale = TIME_SCALE;

    if clock.time_paused {
        return;
    }

    clock.elapsed_real_seconds += time.delta_secs();

    let secs_per_game_minute = 1.0 / TIME_SCALE;
    let whole_minutes = (clock.elapsed_real_seconds / secs_per_game_minute).floor() as u32;

    if whole_minutes == 0 {
        return;
    }

    clock.elapsed_real_seconds -= whole_minutes as f32 * secs_per_game_minute;

    for _ in 0..whole_minutes {
        advance_one_minute(&mut clock);
    }
}

pub fn check_shift_end(mut clock: ResMut<ShiftClock>, mut shift_end: EventWriter<ShiftEndEvent>) {
    if !clock.on_duty || !shift_has_elapsed(&clock) {
        return;
    }

    let completed_shift = clock.shift_number;
    shift_end.send(ShiftEndEvent {
        shift_number: completed_shift,
        cases_progressed: 0,
        evidence_collected: 0,
        xp_earned: 0,
    });

    clock.on_duty = false;
    clock.shift_number = clock.shift_number.saturating_add(1);
}

pub fn handle_new_day_weather(mut clock: ResMut<ShiftClock>, mut last_seen_day: Local<u32>) {
    if *last_seen_day == 0 {
        *last_seen_day = clock.day;
        return;
    }

    if clock.day == *last_seen_day {
        return;
    }

    clock.weather = roll_weather(&mut rand::thread_rng());
    *last_seen_day = clock.day;
}

pub fn check_rank_progression(mut clock: ResMut<ShiftClock>) {
    let next_rank = rank_for_shift(clock.shift_number);
    if clock.rank != next_rank {
        clock.rank = next_rank;
    }
}

fn advance_one_minute(clock: &mut ShiftClock) {
    clock.minute += 1;

    if clock.minute < 60 {
        return;
    }

    clock.minute = 0;
    clock.hour += 1;

    if clock.hour < 24 {
        return;
    }

    clock.hour = 0;
    clock.day = clock.day.saturating_add(1);
    clock.day_of_week = next_day(clock.day_of_week);
}

fn shift_has_elapsed(clock: &ShiftClock) -> bool {
    let current_minutes = i32::from(clock.hour) * 60 + i32::from(clock.minute);
    let shift_start_minutes = i32::from(shift_start_hour(clock.shift_type)) * 60;
    let mut elapsed_minutes = current_minutes - shift_start_minutes;

    if elapsed_minutes < 0 {
        elapsed_minutes += 24 * 60;
    }

    elapsed_minutes >= i32::from(SHIFT_DURATION_HOURS) * 60
}

fn shift_start_hour(shift_type: ShiftType) -> u8 {
    match shift_type {
        ShiftType::Morning => 6,
        ShiftType::Afternoon => 14,
        ShiftType::Night => 22,
    }
}

fn next_day(day: DayOfWeek) -> DayOfWeek {
    match day {
        DayOfWeek::Monday => DayOfWeek::Tuesday,
        DayOfWeek::Tuesday => DayOfWeek::Wednesday,
        DayOfWeek::Wednesday => DayOfWeek::Thursday,
        DayOfWeek::Thursday => DayOfWeek::Friday,
        DayOfWeek::Friday => DayOfWeek::Saturday,
        DayOfWeek::Saturday => DayOfWeek::Sunday,
        DayOfWeek::Sunday => DayOfWeek::Monday,
    }
}

fn rank_for_shift(shift_number: u32) -> Rank {
    match shift_number {
        0..=28 => Rank::PatrolOfficer,
        29..=56 => Rank::Detective,
        57..=84 => Rank::Sergeant,
        _ => Rank::Lieutenant,
    }
}

fn roll_weather(rng: &mut impl Rng) -> Weather {
    match rng.gen_range(0..100) {
        0..=59 => Weather::Clear,
        60..=79 => Weather::Rainy,
        80..=89 => Weather::Foggy,
        _ => Weather::Snowy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;
    use bevy::time::{TimeUpdateStrategy, Virtual};
    use bevy::utils::Duration;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        );
        app.init_resource::<ShiftClock>();
        // Allow manual test durations to advance the virtual clock without Bevy's default 250ms cap.
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .set_max_delta(Duration::MAX);
        app.add_event::<ShiftEndEvent>();
        app.add_plugins(CalendarPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    fn set_delta(app: &mut App, duration: Duration) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(duration));
    }

    #[test]
    fn clock_ticks_forward_when_not_paused() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        set_delta(&mut app, Duration::from_secs(1));

        app.update();

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.hour, 6);
        assert_eq!(clock.minute, 2);
        assert!((clock.time_scale - TIME_SCALE).abs() < f32::EPSILON);
    }

    #[test]
    fn clock_does_not_tick_when_paused() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.hour = 10;
            clock.minute = 30;
            clock.time_paused = true;
        }

        set_delta(&mut app, Duration::from_secs(1));
        app.update();

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.hour, 10);
        assert_eq!(clock.minute, 30);
    }

    #[test]
    fn day_advances_when_hour_reaches_twenty_four() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.day = 1;
            clock.day_of_week = DayOfWeek::Monday;
            clock.hour = 23;
            clock.minute = 59;
        }

        set_delta(&mut app, Duration::from_millis(500));
        app.update();

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.day, 2);
        assert_eq!(clock.day_of_week, DayOfWeek::Tuesday);
        assert_eq!(clock.hour, 0);
        assert_eq!(clock.minute, 0);
    }

    #[test]
    fn shift_end_event_emits_after_eight_hours_on_duty() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.shift_number = 1;
            clock.shift_type = ShiftType::Morning;
            clock.on_duty = true;
            clock.hour = 13;
            clock.minute = 59;
        }

        set_delta(&mut app, Duration::from_millis(500));
        app.update();

        let events = app
            .world_mut()
            .resource_mut::<Events<ShiftEndEvent>>()
            .drain()
            .collect::<Vec<_>>();

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].shift_number, 1);
        assert!(!clock.on_duty);
        assert_eq!(clock.shift_number, 2);
    }

    #[test]
    fn weather_roll_matches_weight_bands() {
        let mut rng = StdRng::seed_from_u64(7);
        let mut clear: i32 = 0;
        let mut rainy: i32 = 0;
        let mut foggy: i32 = 0;
        let mut snowy: i32 = 0;

        for _ in 0..10_000 {
            match roll_weather(&mut rng) {
                Weather::Clear => clear += 1,
                Weather::Rainy => rainy += 1,
                Weather::Foggy => foggy += 1,
                Weather::Snowy => snowy += 1,
            }
        }

        assert!((clear - 6_000).abs() < 500);
        assert!((rainy - 2_000).abs() < 350);
        assert!((foggy - 1_000).abs() < 250);
        assert!((snowy - 1_000).abs() < 250);
    }

    #[test]
    fn rank_progression_updates_at_shift_thresholds() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.shift_number = 57;
            clock.rank = Rank::PatrolOfficer;
        }

        set_delta(&mut app, Duration::ZERO);
        app.update();

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.rank, Rank::Sergeant);
    }
}
