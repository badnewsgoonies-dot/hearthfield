//! Mission tracking — active mission progress, completion handling.

use bevy::prelude::*;
use crate::shared::*;

pub fn track_active_mission(
    flight_state: Res<FlightState>,
    mut mission_board: ResMut<MissionBoard>,
) {
    if let Some(ref mut active) = mission_board.active {
        // Check bonus conditions during flight
        if flight_state.phase == FlightPhase::Arrived {
            for (i, condition) in active.mission.bonus_conditions.iter().enumerate() {
                match condition {
                    BonusCondition::PerfectLanding => {
                        // Checked in flight complete handler
                    }
                    BonusCondition::OnTime => {
                        active.bonuses_met[i] = true; // Simplified
                    }
                    BonusCondition::NoTurbulenceDamage => {
                        active.bonuses_met[i] = flight_state.passengers_happy > 70.0;
                    }
                    BonusCondition::LowFuelUsage => {
                        active.bonuses_met[i] = flight_state.fuel_remaining
                            > flight_state.distance_total_nm * 0.1;
                    }
                    _ => {}
                }
            }
        }
    }
}

pub fn handle_mission_complete(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut mission_board: ResMut<MissionBoard>,
    mut mission_log: ResMut<MissionLog>,
    mut mission_complete_events: EventWriter<MissionCompletedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    calendar: Res<Calendar>,
) {
    for ev in flight_complete_events.read() {
        let active = match mission_board.active.take() {
            Some(a) if a.mission.destination == ev.destination => a,
            Some(a) => {
                mission_board.active = Some(a);
                continue;
            }
            None => continue,
        };

        let bonus_gold: u32 = active
            .bonuses_met
            .iter()
            .enumerate()
            .filter(|(_, met)| **met)
            .map(|(i, _)| match &active.mission.bonus_conditions[i] {
                BonusCondition::PerfectLanding => 100,
                BonusCondition::OnTime => 50,
                BonusCondition::NoTurbulenceDamage => 75,
                BonusCondition::LowFuelUsage => 60,
                BonusCondition::NightFlight => 80,
                BonusCondition::BadWeatherFlight => 120,
            })
            .sum();

        let total_gold = ev.gold_earned + bonus_gold;
        let total_xp = ev.xp_earned;

        mission_log.completed.push(CompletedMission {
            mission_id: active.mission.id.clone(),
            day_completed: calendar.total_days(),
            landing_grade: ev.landing_grade.clone(),
            gold_earned: total_gold,
            xp_earned: total_xp,
            flight_time_minutes: ev.flight_time_secs / 60.0,
        });

        mission_board.completed_ids.push(active.mission.id.clone());

        mission_complete_events.send(MissionCompletedEvent {
            mission_id: active.mission.id.clone(),
            gold_earned: total_gold,
            xp_earned: total_xp,
        });

        if bonus_gold > 0 {
            toast_events.send(ToastEvent {
                message: format!("Bonus conditions met! +{}g extra", bonus_gold),
                duration_secs: 4.0,
            });
        }
    }
}
