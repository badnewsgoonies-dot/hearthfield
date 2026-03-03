//! Landing evaluation system.

use bevy::prelude::*;
use crate::shared::*;

pub fn evaluate_landing(
    input: Res<PlayerInput>,
    mut flight_state: ResMut<FlightState>,
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
            fuel_used: flight_state.distance_total_nm, // simplified
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

fn evaluate_landing_quality(state: &FlightState) -> LandingGrade {
    let speed_factor = if state.speed_knots < 80.0 {
        1.0
    } else if state.speed_knots < 120.0 {
        0.7
    } else {
        0.3
    };

    let flaps_factor = if state.flaps_deployed { 1.0 } else { 0.5 };
    let passenger_factor = state.passengers_happy / 100.0;

    let score = speed_factor * 0.4 + flaps_factor * 0.2 + passenger_factor * 0.4;

    if score > 0.9 {
        LandingGrade::Perfect
    } else if score > 0.7 {
        LandingGrade::Good
    } else if score > 0.5 {
        LandingGrade::Acceptable
    } else if score > 0.3 {
        LandingGrade::Hard
    } else {
        LandingGrade::Rough
    }
}
