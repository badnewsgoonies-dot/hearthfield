//! Cruise flight simulation.

use bevy::prelude::*;
use crate::shared::*;

#[allow(clippy::too_many_arguments)]
pub fn update_flight(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut flight_state: ResMut<FlightState>,
    weather_state: Res<WeatherState>,
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mut phase_events: EventWriter<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent
    ) {
        return;
    }

    let dt = time.delta_secs();
    flight_state.flight_time_secs += dt;

    // Throttle control
    if input.throttle_up {
        flight_state.throttle = (flight_state.throttle + 0.3 * dt).min(1.0);
    }
    if input.throttle_down {
        flight_state.throttle = (flight_state.throttle - 0.3 * dt).max(0.0);
    }

    // Heading control
    if input.yaw_left {
        flight_state.heading_deg = (flight_state.heading_deg - 45.0 * dt) % 360.0;
        if flight_state.heading_deg < 0.0 {
            flight_state.heading_deg += 360.0;
        }
    }
    if input.yaw_right {
        flight_state.heading_deg = (flight_state.heading_deg + 45.0 * dt) % 360.0;
    }

    // Fuel consumption
    let burn_rate = if let Some(ac) = fleet.active() {
        if let Some(def) = aircraft_registry.get(&ac.aircraft_id) {
            def.fuel_burn_rate
        } else {
            1.0
        }
    } else {
        1.0
    };
    flight_state.fuel_remaining -= burn_rate * flight_state.throttle * dt / 60.0;

    // Distance progress
    let speed_nm_per_sec = flight_state.speed_knots / 3600.0;
    flight_state.distance_remaining_nm -= speed_nm_per_sec * dt;

    // Turbulence effects
    let turb = weather_state.turbulence_level;
    flight_state.turbulence_shake = match turb {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 1.0,
        TurbulenceLevel::Moderate => 3.0,
        TurbulenceLevel::Severe => 6.0,
    };

    // Passenger happiness affected by turbulence
    if flight_state.turbulence_shake > 2.0 {
        flight_state.passengers_happy =
            (flight_state.passengers_happy - 5.0 * dt).max(0.0);
    }

    // Phase transitions
    if flight_state.phase == FlightPhase::Climb && flight_state.altitude_ft >= 10000.0 {
        flight_state.phase = FlightPhase::Cruise;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Cruise,
        });
    }

    if flight_state.distance_remaining_nm <= flight_state.distance_total_nm * 0.2
        && flight_state.phase == FlightPhase::Cruise
    {
        flight_state.phase = FlightPhase::Descent;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Descent,
        });
        toast_events.send(ToastEvent {
            message: "Beginning descent...".to_string(),
            duration_secs: 3.0,
        });
    }

    if flight_state.distance_remaining_nm <= 10.0 && flight_state.phase == FlightPhase::Descent
    {
        flight_state.phase = FlightPhase::Approach;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Approach,
        });
    }

    // Emergency: out of fuel
    if flight_state.fuel_remaining <= 0.0 {
        flight_state.fuel_remaining = 0.0;
        flight_state.phase = FlightPhase::Emergency;
        toast_events.send(ToastEvent {
            message: "EMERGENCY: Fuel depleted!".to_string(),
            duration_secs: 5.0,
        });
    }

    // Altitude changes during climb/descent
    match flight_state.phase {
        FlightPhase::Climb => {
            flight_state.altitude_ft += 2000.0 * flight_state.throttle * dt;
        }
        FlightPhase::Descent => {
            flight_state.altitude_ft =
                (flight_state.altitude_ft - 1500.0 * dt).max(2000.0);
        }
        _ => {}
    }

    // Speed from throttle + aircraft base speed
    let base_speed = if let Some(ac) = fleet.active() {
        if let Some(def) = aircraft_registry.get(&ac.aircraft_id) {
            def.speed_knots
        } else {
            FLIGHT_SPEED_BASE
        }
    } else {
        FLIGHT_SPEED_BASE
    };
    flight_state.speed_knots = base_speed * flight_state.throttle;
}
