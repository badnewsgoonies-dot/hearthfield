//! Preflight checklist system.
//!
//! Interactive preflight sequence: player presses F to verify each check.
//! Checks fuel, aircraft condition, instruments, weather, cargo/passengers,
//! and the filed flight plan. Transitions to Taxi on completion.

use bevy::prelude::*;
use crate::shared::*;

// ── Progress tracking ────────────────────────────────────────────────────

#[derive(Resource)]
pub struct PreflightProgress {
    pub current_step: usize,
    pub total_steps: usize,
    pub all_passed: bool,
}

impl Default for PreflightProgress {
    fn default() -> Self {
        Self {
            current_step: 0,
            total_steps: 6,
            all_passed: false,
        }
    }
}

// ── Main system ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn run_preflight_checklist(
    input: Res<PlayerInput>,
    mut flight_state: ResMut<FlightState>,
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    weather_state: Res<WeatherState>,
    mission_board: Res<MissionBoard>,
    mut toast_events: EventWriter<ToastEvent>,
    mut phase_events: EventWriter<FlightPhaseChangeEvent>,
    mut progress: ResMut<PreflightProgress>,
) {
    if flight_state.phase != FlightPhase::Preflight {
        return;
    }

    if !input.interact {
        return;
    }

    // Final confirmation after all checks pass
    if progress.all_passed {
        flight_state.phase = FlightPhase::Taxi;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Taxi,
        });
        toast_events.send(ToastEvent {
            message: "✓ Preflight complete — taxiing to runway.".into(),
            duration_secs: 3.0,
        });
        *progress = PreflightProgress::default();
        return;
    }

    // Run the current check
    let (passed, msg) = match progress.current_step {
        0 => check_fuel(&fleet, &aircraft_registry, &flight_state),
        1 => check_aircraft_condition(&fleet),
        2 => check_instruments(),
        3 => check_weather(&weather_state),
        4 => check_cargo_passengers(&mission_board),
        5 => check_flight_plan(&flight_state),
        _ => (true, "✓ Check complete".into()),
    };

    toast_events.send(ToastEvent {
        message: msg,
        duration_secs: if passed { 2.5 } else { 4.0 },
    });

    if passed {
        progress.current_step += 1;
        if progress.current_step >= progress.total_steps {
            progress.all_passed = true;
            toast_events.send(ToastEvent {
                message: "All checks passed! Press F to taxi.".into(),
                duration_secs: 3.0,
            });
        }
    }
    // On failure the step does not advance; player can retry with F.
}

// ── Reset on enter ───────────────────────────────────────────────────────

pub fn reset_preflight_on_enter(mut progress: ResMut<PreflightProgress>) {
    *progress = PreflightProgress::default();
}

// ── Individual checks ────────────────────────────────────────────────────

fn check_fuel(
    fleet: &Fleet,
    registry: &AircraftRegistry,
    fs: &FlightState,
) -> (bool, String) {
    let Some(ac) = fleet.active() else {
        return (false, "✗ Fuel: No aircraft selected!".into());
    };
    let capacity = registry
        .get(&ac.aircraft_id)
        .map(|d| d.fuel_capacity)
        .unwrap_or(100.0);
    let reserve = capacity * 0.15;
    let needed = fs.distance_total_nm * 0.1 + reserve;
    let required = needed.min(capacity);

    if ac.fuel >= required {
        (
            true,
            format!(
                "✓ Fuel: {:.0}/{:.0} — sufficient for {:.0} nm + reserve",
                ac.fuel, capacity, fs.distance_total_nm
            ),
        )
    } else {
        (
            false,
            format!(
                "✗ Fuel: {:.0}/{:.0} — need {:.0} for flight + reserve!",
                ac.fuel, capacity, required
            ),
        )
    }
}

fn check_aircraft_condition(fleet: &Fleet) -> (bool, String) {
    let Some(ac) = fleet.active() else {
        return (false, "✗ Aircraft: No aircraft in fleet!".into());
    };
    if ac.condition > 20.0 {
        (
            true,
            format!("✓ Engines: Condition {:.0}% — airworthy", ac.condition),
        )
    } else {
        (
            false,
            format!(
                "✗ Engines: Condition {:.0}% — below minimum 20%!",
                ac.condition
            ),
        )
    }
}

fn check_instruments() -> (bool, String) {
    (true, "✓ Instruments: All gauges nominal".into())
}

fn check_weather(weather: &WeatherState) -> (bool, String) {
    if weather.current.is_flyable() {
        (
            true,
            format!("✓ Weather: {:?} — cleared for flight", weather.current),
        )
    } else {
        (
            false,
            format!(
                "✗ Weather: {:?} — conditions unsafe for flight!",
                weather.current
            ),
        )
    }
}

fn check_cargo_passengers(board: &MissionBoard) -> (bool, String) {
    if let Some(ref active) = board.active {
        let m = &active.mission;
        (
            true,
            format!(
                "✓ Cargo/Pax: {} pax, {:.0} kg cargo loaded",
                m.passenger_count, m.cargo_kg
            ),
        )
    } else {
        (true, "✓ Cargo/Pax: No mission cargo — ferry flight".into())
    }
}

fn check_flight_plan(fs: &FlightState) -> (bool, String) {
    (
        true,
        format!(
            "✓ Flight plan: {} → {} ({:.0} nm) filed",
            fs.origin.display_name(),
            fs.destination.display_name(),
            fs.distance_total_nm,
        ),
    )
}
