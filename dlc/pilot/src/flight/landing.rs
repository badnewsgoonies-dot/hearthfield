//! Landing evaluation system.

use bevy::prelude::*;
use crate::shared::*;

#[allow(clippy::too_many_arguments)]
pub fn evaluate_landing(
    input: Res<PlayerInput>,
    mut flight_state: ResMut<FlightState>,
    mut pilot_state: ResMut<PilotState>,
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mut flight_complete_events: EventWriter<FlightCompleteEvent>,
    mut phase_events: EventWriter<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    mut xp_events: EventWriter<XpGainEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mission_board: Res<MissionBoard>,
) {
    if flight_state.phase != FlightPhase::Approach {
        return;
    }

    // Landing is triggered by gear down + low altitude + confirm
    if !flight_state.gear_down && input.gear_toggle {
        flight_state.gear_down = true;
        toast_events.send(ToastEvent {
            message: "Landing gear deployed".to_string(),
            duration_secs: 2.0,
        });
    }

    if input.flaps_toggle {
        flight_state.flaps_deployed = !flight_state.flaps_deployed;
    }

    // Auto-land when conditions met (simplified for now)
    if flight_state.gear_down && flight_state.altitude_ft < 500.0 && input.confirm {
        let grade = evaluate_landing_quality(&flight_state);
        let grade_str = format!("{:?}", grade);
        let xp_bonus = grade.xp_bonus();
        let rep_change = grade.reputation_change();

        // Apply reputation change from landing quality
        pilot_state.reputation = (pilot_state.reputation + rep_change).clamp(0.0, 100.0);

        // Calculate fuel used from aircraft burn rate and flight time
        // fuel_burn_rate is in gallons/hour, flight_time_secs is in seconds
        let fuel_burn_rate = fleet.active()
            .and_then(|ac| aircraft_registry.get(&ac.aircraft_id))
            .map_or(1.0, |def| def.fuel_burn_rate);
        let fuel_used = flight_state.flight_time_secs / 3600.0 * fuel_burn_rate;

        // Mission rewards
        let (gold, base_xp) = if let Some(ref active) = mission_board.active {
            (active.mission.reward_gold, active.mission.reward_xp)
        } else {
            (50, 20) // Free flight rewards
        };

        let total_xp = base_xp + xp_bonus;

        flight_state.phase = FlightPhase::Arrived;
        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Arrived,
        });

        flight_complete_events.send(FlightCompleteEvent {
            origin: flight_state.origin,
            destination: flight_state.destination,
            landing_grade: grade_str.clone(),
            flight_time_secs: flight_state.flight_time_secs,
            fuel_used,
            xp_earned: total_xp,
            gold_earned: gold,
        });

        xp_events.send(XpGainEvent {
            amount: total_xp,
            source: format!("Flight to {}", flight_state.destination.display_name()),
        });

        gold_events.send(GoldChangeEvent {
            amount: gold as i32,
            reason: format!("Flight completed — {}", grade_str),
        });

        toast_events.send(ToastEvent {
            message: format!(
                "Landed! Grade: {} | +{}g +{}xp",
                grade_str, gold, total_xp
            ),
            duration_secs: 5.0,
        });
    }
}

// ── Landing quality scoring constants ─────────────────────────────────

/// Speed factor thresholds (knots).
const SPEED_GOOD_THRESHOLD: f32 = 80.0;
const SPEED_OK_THRESHOLD: f32 = 120.0;

/// Speed factor values for good / ok / fast landings.
const SPEED_FACTOR_GOOD: f32 = 1.0;
const SPEED_FACTOR_OK: f32 = 0.7;
const SPEED_FACTOR_FAST: f32 = 0.3;

/// Score component weights (must sum to 1.0).
const WEIGHT_SPEED: f32 = 0.4;
const WEIGHT_FLAPS: f32 = 0.2;
const WEIGHT_PASSENGERS: f32 = 0.4;

/// Landing grade thresholds applied to the composite score.
const GRADE_PERFECT_THRESHOLD: f32 = 0.9;
const GRADE_GOOD_THRESHOLD: f32 = 0.7;
const GRADE_ACCEPTABLE_THRESHOLD: f32 = 0.5;
const GRADE_HARD_THRESHOLD: f32 = 0.3;

fn evaluate_landing_quality(state: &FlightState) -> LandingGrade {
    let speed_factor = if state.speed_knots < SPEED_GOOD_THRESHOLD {
        SPEED_FACTOR_GOOD
    } else if state.speed_knots < SPEED_OK_THRESHOLD {
        SPEED_FACTOR_OK
    } else {
        SPEED_FACTOR_FAST
    };

    let flaps_factor = if state.flaps_deployed { 1.0 } else { 0.5 };
    let passenger_factor = state.passengers_happy / 100.0;

    let score =
        speed_factor * WEIGHT_SPEED + flaps_factor * WEIGHT_FLAPS + passenger_factor * WEIGHT_PASSENGERS;

    if score > GRADE_PERFECT_THRESHOLD {
        LandingGrade::Perfect
    } else if score > GRADE_GOOD_THRESHOLD {
        LandingGrade::Good
    } else if score > GRADE_ACCEPTABLE_THRESHOLD {
        LandingGrade::Acceptable
    } else if score > GRADE_HARD_THRESHOLD {
        LandingGrade::Hard
    } else {
        LandingGrade::Rough
    }
}
