//! Takeoff system.
//!
//! Two systems: `transition_taxi_to_takeoff` auto-advances from Taxi after a
//! short delay, and `handle_takeoff` manages the runway roll, speed callouts,
//! rotation, liftoff, gear-up prompt, abort logic, and crosswind drift.

use bevy::prelude::*;
use crate::shared::*;

// ── Constants ────────────────────────────────────────────────────────────

const V1_SPEED: f32 = 100.0;
const VR_SPEED: f32 = 120.0;
const V2_SPEED: f32 = 140.0;
const CLIMB_TRANSITION_ALT: f32 = 500.0;
const ACCELERATION_RATE: f32 = 40.0;
const ROTATION_CLIMB_RATE: f32 = 800.0;
const BRAKE_DECEL: f32 = 60.0;
const TAXI_DELAY_SECS: f32 = 3.0;

// ── Per-flight takeoff state ─────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct TakeoffState {
    callout_60: bool,
    callout_v1: bool,
    callout_rotate: bool,
    callout_v2: bool,
    callout_positive_climb: bool,
    gear_up_prompted: bool,
    aborted: bool,
    liftoff: bool,
    crosswind_drift: f32,
}

pub fn reset_takeoff_state(mut state: ResMut<TakeoffState>) {
    *state = TakeoffState::default();
}

// ── Taxi → Takeoff transition ────────────────────────────────────────────

pub fn transition_taxi_to_takeoff(
    time: Res<Time>,
    mut flight_state: ResMut<FlightState>,
    mut phase_events: EventWriter<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    mut taxi_timer: Local<f32>,
) {
    if flight_state.phase != FlightPhase::Taxi {
        *taxi_timer = 0.0;
        return;
    }

    *taxi_timer += time.delta_secs();
    if *taxi_timer >= TAXI_DELAY_SECS {
        flight_state.phase = FlightPhase::Takeoff;
        flight_state.speed_knots = 0.0;
        flight_state.altitude_ft = 0.0;
        flight_state.gear_down = true;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Takeoff,
        });
        toast_events.send(ToastEvent {
            message: "Runway cleared for takeoff — throttle up!".into(),
            duration_secs: 3.0,
        });
        *taxi_timer = 0.0;
    }
}

// ── Main takeoff system ──────────────────────────────────────────────────

pub fn handle_takeoff(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut flight_state: ResMut<FlightState>,
    weather_state: Res<WeatherState>,
    mut phase_events: EventWriter<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    mut state: ResMut<TakeoffState>,
) {
    if flight_state.phase != FlightPhase::Takeoff {
        return;
    }

    let dt = time.delta_secs();

    // ── Abort logic ──────────────────────────────────────────────────
    if input.throttle_down
        && flight_state.speed_knots < V1_SPEED
        && flight_state.speed_knots > 10.0
        && !state.aborted
    {
        state.aborted = true;
        toast_events.send(ToastEvent {
            message: "ABORT! Braking…".into(),
            duration_secs: 4.0,
        });
    }

    if state.aborted {
        flight_state.speed_knots = (flight_state.speed_knots - BRAKE_DECEL * dt).max(0.0);
        if flight_state.speed_knots <= 0.0 {
            flight_state.phase = FlightPhase::Idle;
            phase_events.send(FlightPhaseChangeEvent {
                new_phase: FlightPhase::Idle,
            });
            toast_events.send(ToastEvent {
                message: "Takeoff aborted — returning to gate.".into(),
                duration_secs: 3.0,
            });
            *state = TakeoffState::default();
        }
        return;
    }

    // ── Runway roll ──────────────────────────────────────────────────
    if input.throttle_up {
        flight_state.speed_knots += ACCELERATION_RATE * dt;
        flight_state.throttle = (flight_state.throttle + 0.5 * dt).min(1.0);
    }

    // ── Crosswind drift ──────────────────────────────────────────────
    let crosswind =
        weather_state.wind_speed_knots * weather_state.wind_direction_deg.to_radians().sin();
    state.crosswind_drift += crosswind * 0.01 * dt;
    flight_state.heading_deg += state.crosswind_drift * dt;
    // Keep heading in 0..360
    flight_state.heading_deg = flight_state.heading_deg.rem_euclid(360.0);

    // ── Speed callouts ───────────────────────────────────────────────
    if flight_state.speed_knots >= 60.0 && !state.callout_60 {
        state.callout_60 = true;
        toast_events.send(ToastEvent {
            message: "60 knots".into(),
            duration_secs: 1.5,
        });
    }

    if flight_state.speed_knots >= V1_SPEED && !state.callout_v1 {
        state.callout_v1 = true;
        toast_events.send(ToastEvent {
            message: "V1 — commit to takeoff".into(),
            duration_secs: 2.0,
        });
    }

    if flight_state.speed_knots >= VR_SPEED && !state.callout_rotate {
        state.callout_rotate = true;
        state.liftoff = true;
        flight_state.gear_down = true;
        toast_events.send(ToastEvent {
            message: "Rotate!".into(),
            duration_secs: 2.0,
        });
    }

    // ── Post-rotation climb ──────────────────────────────────────────
    if state.liftoff {
        flight_state.altitude_ft += ROTATION_CLIMB_RATE * flight_state.throttle * dt;
    }

    if flight_state.speed_knots >= V2_SPEED && !state.callout_v2 {
        state.callout_v2 = true;
        toast_events.send(ToastEvent {
            message: "V2 — safe climb speed".into(),
            duration_secs: 2.0,
        });
    }

    if flight_state.altitude_ft > 200.0 && !state.callout_positive_climb {
        state.callout_positive_climb = true;
        toast_events.send(ToastEvent {
            message: "Positive climb established".into(),
            duration_secs: 2.0,
        });
    }

    // ── Gear-up prompt ───────────────────────────────────────────────
    if state.callout_positive_climb && !state.gear_up_prompted {
        state.gear_up_prompted = true;
        toast_events.send(ToastEvent {
            message: "Gear up — press B".into(),
            duration_secs: 3.0,
        });
    }

    if input.gear_toggle && flight_state.gear_down && state.liftoff {
        flight_state.gear_down = false;
        toast_events.send(ToastEvent {
            message: "Gear up ✓".into(),
            duration_secs: 2.0,
        });
    }

    // ── Transition to Climb phase ────────────────────────────────────
    if flight_state.altitude_ft >= CLIMB_TRANSITION_ALT {
        flight_state.phase = FlightPhase::Climb;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Climb,
        });
        toast_events.send(ToastEvent {
            message: "Airborne! Climbing to cruise altitude…".into(),
            duration_secs: 3.0,
        });
        *state = TakeoffState::default();
    }
}
