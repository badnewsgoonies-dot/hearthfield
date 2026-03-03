//! Mission board — refresh available missions, accept missions.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

/// Refresh the mission board daily with new missions.
pub fn refresh_mission_board(
    mut day_end_events: EventReader<DayEndEvent>,
    mut mission_board: ResMut<MissionBoard>,
    pilot_state: Res<PilotState>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let mut rng = rand::thread_rng();
        mission_board.available.clear();

        let num_missions = match pilot_state.rank {
            PilotRank::Student => 2,
            PilotRank::Private => 3,
            PilotRank::Commercial => 4,
            PilotRank::Senior => 5,
            PilotRank::Captain => 6,
            PilotRank::Ace => 8,
        };

        let available_airports: Vec<AirportId> = vec![
            AirportId::HomeBase,
            AirportId::Windport,
            AirportId::Frostpeak,
            AirportId::Sunhaven,
            AirportId::Ironforge,
            AirportId::Cloudmere,
            AirportId::Duskhollow,
            AirportId::Stormwatch,
            AirportId::Grandcity,
            AirportId::Skyreach,
        ]
        .into_iter()
        .filter(|a| a.unlock_rank() <= pilot_state.rank)
        .collect();

        let mission_types = [
            MissionType::Passenger,
            MissionType::Cargo,
            MissionType::Medical,
            MissionType::Charter,
            MissionType::Training,
            MissionType::Delivery,
        ];

        for i in 0..num_missions {
            if available_airports.len() < 2 {
                break;
            }
            let origin = pilot_state.current_airport;
            let dest_idx = rng.gen_range(0..available_airports.len());
            let mut dest = available_airports[dest_idx];
            if dest == origin {
                dest = available_airports[(dest_idx + 1) % available_airports.len()];
            }

            let mtype = mission_types[rng.gen_range(0..mission_types.len())];
            let distance = airport_distance(origin, dest);
            let base_reward = (distance * 2.0) as u32 + 50;
            let xp_reward = (distance * 0.5) as u32 + 10;

            let difficulty = if distance < 200.0 {
                MissionDifficulty::Easy
            } else if distance < 400.0 {
                MissionDifficulty::Medium
            } else if distance < 600.0 {
                MissionDifficulty::Hard
            } else {
                MissionDifficulty::Expert
            };

            mission_board.available.push(MissionDef {
                id: format!("mission_d{}_{}", calendar.total_days(), i),
                title: format!("{} to {}", mtype.display_name(), dest.display_name()),
                description: format!(
                    "Fly from {} to {}. Distance: {:.0} nm.",
                    origin.display_name(),
                    dest.display_name(),
                    distance
                ),
                mission_type: mtype,
                origin,
                destination: dest,
                reward_gold: base_reward,
                reward_xp: xp_reward,
                time_limit_hours: None,
                required_rank: dest.unlock_rank(),
                required_aircraft_class: None,
                passenger_count: if matches!(mtype, MissionType::Passenger | MissionType::VIP | MissionType::Charter) {
                    rng.gen_range(1..30)
                } else {
                    0
                },
                cargo_kg: if matches!(mtype, MissionType::Cargo | MissionType::Delivery) {
                    rng.gen_range(100.0..5000.0)
                } else {
                    0.0
                },
                bonus_conditions: vec![BonusCondition::PerfectLanding, BonusCondition::OnTime],
                difficulty,
            });
        }
    }
}

pub fn handle_mission_accepted(
    mut events: EventReader<MissionAcceptedEvent>,
    mut mission_board: ResMut<MissionBoard>,
    mut toast_events: EventWriter<ToastEvent>,
    calendar: Res<Calendar>,
) {
    for ev in events.read() {
        if let Some(idx) = mission_board
            .available
            .iter()
            .position(|m| m.id == ev.mission_id)
        {
            let mission = mission_board.available.remove(idx);
            toast_events.send(ToastEvent {
                message: format!("Mission accepted: {}", mission.title),
                duration_secs: 3.0,
            });
            mission_board.active = Some(ActiveMission {
                bonuses_met: vec![false; mission.bonus_conditions.len()],
                mission,
                accepted_day: calendar.total_days(),
            });
        }
    }
}
