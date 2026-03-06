//! Cruise flight simulation.

use crate::shared::*;
use bevy::prelude::*;

/// Reference cargo weight for performance penalty normalisation (kg).
const CARGO_WEIGHT_REF_KG: f32 = 5_000.0;
/// Maximum climb-rate reduction fraction at full cargo load.
const CARGO_CLIMB_PENALTY: f32 = 0.30;
/// Maximum cruise-speed reduction fraction at full cargo load.
const CARGO_SPEED_PENALTY: f32 = 0.10;

/// Wind component along the flight heading.
/// Positive = headwind (slows us), negative = tailwind (speeds us up).
fn headwind_component(wind_speed: f32, wind_dir_deg: f32, heading_deg: f32) -> f32 {
    let relative_deg = wind_dir_deg - heading_deg;
    wind_speed * relative_deg.to_radians().cos()
}

#[allow(clippy::too_many_arguments)]
pub fn update_flight(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut flight_state: ResMut<FlightState>,
    weather_state: Res<WeatherState>,
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mission_board: Res<MissionBoard>,
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

    // Cargo weight from the active mission (0 if no mission)
    let cargo_kg = mission_board
        .active
        .as_ref()
        .map_or(0.0, |m| m.mission.cargo_kg);
    let cargo_ratio = (cargo_kg / CARGO_WEIGHT_REF_KG).min(1.0);

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

    // ── Weather: wind effect on ground speed ────────────────────────────
    // Headwind reduces ground speed; tailwind increases it.
    let headwind = headwind_component(
        weather_state.wind_speed_knots,
        weather_state.wind_direction_deg,
        flight_state.heading_deg,
    );
    // Wind factor: headwind (positive cos) slows us, tailwind speeds us up.
    // Clamp so wind can't reverse or more than double ground speed.
    let wind_factor = (1.0 - headwind / FLIGHT_SPEED_BASE.max(1.0)).clamp(0.5, 1.5);

    // ── Weather: turbulence altitude variance ───────────────────────────
    let turbulence_intensity = match weather_state.turbulence_level {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 1.0,
        TurbulenceLevel::Moderate => 3.0,
        TurbulenceLevel::Severe => 6.0,
    };
    flight_state.turbulence_shake = turbulence_intensity;

    if turbulence_intensity > 0.0 {
        // Altitude jitter proportional to turbulence intensity (±feet)
        let t = time.elapsed_secs();
        let alt_variance = (t * 7.3).sin() * turbulence_intensity * 15.0;
        flight_state.altitude_ft = (flight_state.altitude_ft + alt_variance * dt).max(0.0);
    }

    // Distance progress (ground speed accounts for wind)
    let speed_nm_per_sec = flight_state.speed_knots * wind_factor / 3600.0;
    flight_state.distance_remaining_nm -= speed_nm_per_sec * dt;

    // Passenger happiness affected by turbulence
    if flight_state.turbulence_shake > 2.0 {
        flight_state.passengers_happy = (flight_state.passengers_happy - 5.0 * dt).max(0.0);
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

    if flight_state.distance_remaining_nm <= 10.0 && flight_state.phase == FlightPhase::Descent {
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
    // Rates are in ft/min; divide by 60.0 to get ft/sec for per-frame delta.
    const CLIMB_RATE_FT_PER_MIN: f32 = 2000.0;
    const DESCENT_RATE_FT_PER_MIN: f32 = 1500.0;

    match flight_state.phase {
        FlightPhase::Climb => {
            // Cargo weight reduces climb rate (heavier = slower climb)
            let cargo_climb_factor = 1.0 - cargo_ratio * CARGO_CLIMB_PENALTY;
            flight_state.altitude_ft +=
                (CLIMB_RATE_FT_PER_MIN / 60.0) * flight_state.throttle * cargo_climb_factor * dt;
        }
        FlightPhase::Descent => {
            flight_state.altitude_ft =
                (flight_state.altitude_ft - (DESCENT_RATE_FT_PER_MIN / 60.0) * dt).max(2000.0);
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

    // Cargo weight reduces cruise speed slightly
    let cargo_speed_factor = 1.0 - cargo_ratio * CARGO_SPEED_PENALTY;
    flight_state.speed_knots = base_speed * flight_state.throttle * cargo_speed_factor;
}
