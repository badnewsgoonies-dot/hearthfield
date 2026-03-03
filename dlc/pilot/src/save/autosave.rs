//! Autosave system — triggered on day end, periodic timer, and flight completion.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════════

/// Autosave configuration.
#[derive(Resource)]
pub struct AutosaveConfig {
    /// Autosave slot number (separate from manual save slots).
    pub slot: usize,
    /// How many game-days between periodic autosaves.
    pub interval_days: u32,
    /// Counter tracking days since last periodic autosave.
    pub days_since_save: u32,
    /// Whether autosave is enabled.
    pub enabled: bool,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            slot: 0,
            interval_days: 3,
            days_since_save: 0,
            enabled: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Autosave at the end of each day.
pub fn autosave_on_day_end(
    mut day_events: EventReader<DayEndEvent>,
    config: Res<AutosaveConfig>,
    mut save_events: EventWriter<SaveRequestEvent>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !config.enabled {
        for _ev in day_events.read() {}
        return;
    }

    for _ev in day_events.read() {
        save_events.send(SaveRequestEvent { slot: config.slot });
        toast.send(ToastEvent {
            message: "Game saved".into(),
            duration_secs: 1.5,
        });
    }
}

/// Periodic autosave every N game-days.
pub fn autosave_timer(
    mut day_events: EventReader<DayEndEvent>,
    mut config: ResMut<AutosaveConfig>,
    mut save_events: EventWriter<SaveRequestEvent>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !config.enabled {
        for _ev in day_events.read() {}
        return;
    }

    for _ev in day_events.read() {
        config.days_since_save += 1;
        if config.days_since_save >= config.interval_days {
            config.days_since_save = 0;
            save_events.send(SaveRequestEvent { slot: config.slot });
            toast.send(ToastEvent {
                message: "Periodic autosave complete".into(),
                duration_secs: 1.5,
            });
        }
    }
}

/// Autosave after completing a flight.
pub fn autosave_on_flight_complete(
    mut flight_events: EventReader<FlightCompleteEvent>,
    config: Res<AutosaveConfig>,
    mut save_events: EventWriter<SaveRequestEvent>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !config.enabled {
        for _ev in flight_events.read() {}
        return;
    }

    for _ev in flight_events.read() {
        save_events.send(SaveRequestEvent { slot: config.slot });
        toast.send(ToastEvent {
            message: "Game saved".into(),
            duration_secs: 1.5,
        });
    }
}
