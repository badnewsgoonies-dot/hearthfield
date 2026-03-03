//! Day/night cycle and time management — advances calendar time each frame,
//! handles late-night warnings, forces sleep, and applies lighting tints.

use bevy::prelude::*;
use crate::shared::*;

// ─── Time Period ─────────────────────────────────────────────────────────

/// Coarse time-of-day buckets used for lighting and gameplay checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimePeriod {
    Dawn,      // 5–7
    Morning,   // 7–12
    Afternoon, // 12–17
    Evening,   // 17–20
    Night,     // 20–5
}

impl TimePeriod {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            5..=6 => TimePeriod::Dawn,
            7..=11 => TimePeriod::Morning,
            12..=16 => TimePeriod::Afternoon,
            17..=19 => TimePeriod::Evening,
            _ => TimePeriod::Night,
        }
    }

    /// Ambient colour tint for the camera / global overlay.
    pub fn lighting_tint(&self) -> Color {
        match self {
            TimePeriod::Dawn => Color::srgb(0.95, 0.8, 0.7),
            TimePeriod::Morning => Color::srgb(1.0, 0.97, 0.9),
            TimePeriod::Afternoon => Color::srgb(1.0, 1.0, 1.0),
            TimePeriod::Evening => Color::srgb(0.95, 0.7, 0.45),
            TimePeriod::Night => Color::srgb(0.3, 0.3, 0.55),
        }
    }
}

// ─── Day-cycle configuration ─────────────────────────────────────────────

/// How many game-minutes pass per real second. Adjustable for debug/testing.
#[derive(Resource)]
pub struct DayCycleConfig {
    pub game_minutes_per_real_second: f32,
    pub warned_late: bool,
}

impl Default for DayCycleConfig {
    fn default() -> Self {
        // DAY_LENGTH_SECS = 720 → 24*60 game minutes in 720 real secs = 2 min/sec
        Self {
            game_minutes_per_real_second: (HOURS_IN_DAY as f32 * 60.0) / DAY_LENGTH_SECS,
            warned_late: false,
        }
    }
}

// ─── Component for lighting overlay ──────────────────────────────────────

#[derive(Component)]
pub struct DayLightOverlay;

// ─── Systems ─────────────────────────────────────────────────────────────

/// Advance the calendar clock each frame by converting real delta-time into
/// game minutes and rolling over hours/days as needed.
pub fn advance_time(
    time: Res<Time>,
    mut calendar: ResMut<Calendar>,
    config: Res<DayCycleConfig>,
) {
    if calendar.time_paused {
        return;
    }

    let delta_minutes = time.delta_secs() * config.game_minutes_per_real_second;
    calendar.time_of_day_secs += time.delta_secs();

    let total_minutes = calendar.minute as f32 + delta_minutes;
    let extra_hours = (total_minutes / 60.0).floor() as u32;
    calendar.minute = (total_minutes % 60.0) as u32;
    calendar.hour += extra_hours;

    // Don't auto-roll past 24; `day_end_check` handles that.
}

/// Warn the player when it gets late; force sleep at midnight.
pub fn day_end_check(
    calendar: Res<Calendar>,
    mut config: ResMut<DayCycleConfig>,
    mut toast: EventWriter<ToastEvent>,
    mut day_end: EventWriter<DayEndEvent>,
) {
    if calendar.hour >= 22 && !config.warned_late {
        config.warned_late = true;
        toast.send(ToastEvent {
            message: "Getting late... you should head home soon.".into(),
            duration_secs: 4.0,
        });
    }

    if calendar.hour >= 24 {
        day_end.send(DayEndEvent);
    }
}

/// Execute end-of-day: reset calendar to next morning, restore stamina,
/// fire a save request, advance the calendar day/season/year.
pub fn trigger_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut calendar: ResMut<Calendar>,
    mut pilot: ResMut<PilotState>,
    mut config: ResMut<DayCycleConfig>,
    mut season_events: EventWriter<SeasonChangeEvent>,
    mut save_events: EventWriter<SaveRequestEvent>,
) {
    for _evt in day_end_events.read() {
        // Advance day
        calendar.day += 1;
        calendar.day_of_week = calendar.day_of_week.next();

        // Season roll-over every 28 days
        if calendar.day > 28 {
            calendar.day = 1;
            let next = calendar.season.next();
            season_events.send(SeasonChangeEvent { new_season: next });
            calendar.season = next;

            // Year roll-over
            if next == Season::Spring {
                calendar.year += 1;
            }
        }

        // Reset time to morning
        calendar.hour = WAKE_HOUR;
        calendar.minute = 0;
        calendar.time_of_day_secs = 0.0;

        // Restore stamina
        pilot.stamina = pilot.max_stamina;

        // Reset late warning
        config.warned_late = false;

        // Auto-save
        save_events.send(SaveRequestEvent { slot: 0 });
    }
}

/// Drain stamina over time. Flying drains faster; resting in the lounge
/// drains slower.
pub fn stamina_drain(
    time: Res<Time>,
    state: Res<State<GameState>>,
    location: Res<PlayerLocation>,
    mut pilot: ResMut<PilotState>,
) {
    let rate = match *state.get() {
        GameState::Flying => 2.0,
        _ => {
            if location.zone == MapZone::Lounge || location.zone == MapZone::CrewQuarters {
                0.2
            } else {
                0.5
            }
        }
    };
    pilot.stamina = (pilot.stamina - rate * time.delta_secs()).max(0.0);
}

/// Apply a screen-wide colour tint based on the time of day.
pub fn update_lighting_tint(
    calendar: Res<Calendar>,
    mut query: Query<&mut BackgroundColor, With<DayLightOverlay>>,
) {
    let period = TimePeriod::from_hour(calendar.hour);
    let tint = period.lighting_tint();
    let overlay = Color::srgba(tint.to_srgba().red, tint.to_srgba().green, tint.to_srgba().blue, 0.15);
    for mut bg in &mut query {
        *bg = BackgroundColor(overlay);
    }
}

/// Spawn the translucent full-screen overlay used for day/night tinting.
pub fn spawn_day_light_overlay(mut commands: Commands) {
    commands.spawn((
        DayLightOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
        GlobalZIndex(900),
    ));
}

/// Despawn the day-light overlay.
pub fn despawn_day_light_overlay(
    mut commands: Commands,
    query: Query<Entity, With<DayLightOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
