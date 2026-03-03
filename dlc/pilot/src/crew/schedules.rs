//! Crew daily schedule definitions — where each NPC is throughout the day.

use crate::shared::*;

/// Build a single schedule entry.
fn entry(hour: u32, airport: AirportId, zone: MapZone, gx: i32, gy: i32) -> ScheduleEntry {
    ScheduleEntry { hour, airport, zone, grid_x: gx, grid_y: gy }
}

/// Standard weekday schedule for a crew member who stays at their home airport.
fn weekday_schedule(crew_id: &str, home: AirportId) -> Vec<ScheduleEntry> {
    match crew_id {
        "captain_elena" => vec![
            entry(6, home, MapZone::CrewQuarters, 5, 5),
            entry(7, home, MapZone::Lounge, 4, 5),
            entry(8, home, MapZone::Terminal, 8, 6),
            entry(12, home, MapZone::Lounge, 6, 6),
            entry(13, home, MapZone::ControlTower, 5, 4),
            entry(17, home, MapZone::Lounge, 10, 5),
            entry(20, home, MapZone::CrewQuarters, 5, 5),
        ],
        "copilot_marco" => vec![
            entry(6, home, MapZone::CrewQuarters, 8, 5),
            entry(7, home, MapZone::Lounge, 10, 7),
            entry(8, home, MapZone::Hangar, 8, 8),
            entry(10, home, MapZone::Terminal, 12, 6),
            entry(12, home, MapZone::Lounge, 4, 7),
            entry(14, home, MapZone::Hangar, 14, 8),
            entry(18, home, MapZone::Lounge, 14, 5),
            entry(21, home, MapZone::CrewQuarters, 8, 5),
        ],
        "navigator_yuki" => vec![
            entry(7, home, MapZone::CrewQuarters, 10, 5),
            entry(8, home, MapZone::ControlTower, 8, 4),
            entry(12, home, MapZone::Lounge, 15, 3),
            entry(13, home, MapZone::ControlTower, 5, 4),
            entry(17, home, MapZone::Terminal, 10, 2),
            entry(19, home, MapZone::Lounge, 12, 2),
            entry(22, home, MapZone::CrewQuarters, 10, 5),
        ],
        "mechanic_hank" => vec![
            entry(5, home, MapZone::CrewQuarters, 3, 3),
            entry(6, home, MapZone::Hangar, 3, 3),
            entry(8, home, MapZone::Hangar, 3, 8),
            entry(12, home, MapZone::Lounge, 3, 2),
            entry(13, home, MapZone::Hangar, 3, 13),
            entry(17, home, MapZone::Hangar, 20, 3),
            entry(19, home, MapZone::Lounge, 8, 4),
            entry(21, home, MapZone::CrewQuarters, 3, 3),
        ],
        "attendant_sofia" => vec![
            entry(7, home, MapZone::CrewQuarters, 5, 8),
            entry(8, home, MapZone::Terminal, 6, 3),
            entry(10, home, MapZone::Terminal, 14, 3),
            entry(12, home, MapZone::CityStreet, 10, 8),
            entry(14, home, MapZone::Terminal, 8, 10),
            entry(17, home, MapZone::Lounge, 4, 5),
            entry(20, home, MapZone::CrewQuarters, 5, 8),
        ],
        "controller_raj" => vec![
            entry(6, home, MapZone::CrewQuarters, 8, 8),
            entry(7, home, MapZone::ControlTower, 5, 4),
            entry(12, home, MapZone::Lounge, 6, 6),
            entry(13, home, MapZone::ControlTower, 8, 4),
            entry(19, home, MapZone::ControlTower, 5, 4),
            entry(22, home, MapZone::CrewQuarters, 8, 8),
        ],
        "instructor_chen" => vec![
            entry(7, home, MapZone::CrewQuarters, 10, 3),
            entry(8, home, MapZone::Hangar, 8, 8),
            entry(10, home, MapZone::Runway, 10, 14),
            entry(12, home, MapZone::Lounge, 10, 5),
            entry(14, home, MapZone::Hangar, 14, 8),
            entry(16, home, MapZone::Terminal, 12, 10),
            entry(19, home, MapZone::Lounge, 4, 7),
            entry(21, home, MapZone::CrewQuarters, 10, 3),
        ],
        "veteran_pete" => vec![
            entry(8, home, MapZone::CrewQuarters, 3, 6),
            entry(9, home, MapZone::Lounge, 4, 5),
            entry(11, home, MapZone::Hangar, 3, 3),
            entry(13, home, MapZone::Lounge, 15, 3),
            entry(15, home, MapZone::Terminal, 8, 6),
            entry(18, home, MapZone::Lounge, 10, 7),
            entry(20, home, MapZone::CrewQuarters, 3, 6),
        ],
        "rookie_alex" => vec![
            entry(6, home, MapZone::CrewQuarters, 12, 5),
            entry(7, home, MapZone::Lounge, 6, 6),
            entry(8, home, MapZone::Hangar, 14, 8),
            entry(10, home, MapZone::Runway, 10, 14),
            entry(12, home, MapZone::Lounge, 4, 5),
            entry(13, home, MapZone::Terminal, 14, 3),
            entry(16, home, MapZone::Hangar, 8, 8),
            entry(19, home, MapZone::CityStreet, 10, 8),
            entry(21, home, MapZone::CrewQuarters, 12, 5),
        ],
        "charter_diana" => vec![
            entry(8, home, MapZone::CrewQuarters, 8, 3),
            entry(9, home, MapZone::Terminal, 12, 6),
            entry(11, home, MapZone::CityStreet, 10, 8),
            entry(13, home, MapZone::Lounge, 10, 5),
            entry(15, home, MapZone::Terminal, 8, 6),
            entry(18, home, MapZone::Lounge, 14, 5),
            entry(21, home, MapZone::CrewQuarters, 8, 3),
        ],
        _ => vec![
            entry(8, home, MapZone::CrewQuarters, 5, 5),
            entry(12, home, MapZone::Lounge, 5, 5),
            entry(20, home, MapZone::CrewQuarters, 5, 5),
        ],
    }
}

/// Weekend schedules are more relaxed — later start, more lounge time.
fn weekend_schedule(crew_id: &str, home: AirportId) -> Vec<ScheduleEntry> {
    match crew_id {
        "captain_elena" => vec![
            entry(9, home, MapZone::CrewQuarters, 5, 5),
            entry(10, home, MapZone::Lounge, 4, 5),
            entry(13, home, MapZone::CityStreet, 10, 8),
            entry(16, home, MapZone::Lounge, 10, 5),
            entry(21, home, MapZone::CrewQuarters, 5, 5),
        ],
        "mechanic_hank" => vec![
            entry(7, home, MapZone::CrewQuarters, 3, 3),
            entry(8, home, MapZone::Hangar, 3, 8),
            entry(12, home, MapZone::Lounge, 3, 2),
            entry(14, home, MapZone::CityStreet, 10, 8),
            entry(18, home, MapZone::Lounge, 8, 4),
            entry(21, home, MapZone::CrewQuarters, 3, 3),
        ],
        "controller_raj" => vec![
            entry(8, home, MapZone::CrewQuarters, 8, 8),
            entry(9, home, MapZone::ControlTower, 5, 4),
            entry(14, home, MapZone::Lounge, 6, 6),
            entry(18, home, MapZone::CityStreet, 10, 8),
            entry(22, home, MapZone::CrewQuarters, 8, 8),
        ],
        _ => {
            vec![
                entry(9, home, MapZone::CrewQuarters, 5, 5),
                entry(10, home, MapZone::Lounge, 5, 7),
                entry(13, home, MapZone::CityStreet, 10, 8),
                entry(17, home, MapZone::Lounge, 10, 5),
                entry(21, home, MapZone::CrewQuarters, 5, 5),
            ]
        }
    }
}

/// Return the home airport for each crew member.
fn crew_home_airport(crew_id: &str) -> AirportId {
    match crew_id {
        "captain_elena" | "copilot_marco" | "mechanic_hank" | "rookie_alex" => AirportId::HomeBase,
        "navigator_yuki" => AirportId::Windport,
        "attendant_sofia" => AirportId::Sunhaven,
        "controller_raj" => AirportId::HomeBase,
        "instructor_chen" => AirportId::HomeBase,
        "veteran_pete" => AirportId::Frostpeak,
        "charter_diana" => AirportId::Grandcity,
        _ => AirportId::HomeBase,
    }
}

/// Build the full schedule for a crew member, with distinct weekday/weekend.
pub fn get_schedule(crew_id: &str) -> NpcSchedule {
    let home = crew_home_airport(crew_id);
    NpcSchedule {
        weekday: weekday_schedule(crew_id, home),
        weekend: weekend_schedule(crew_id, home),
    }
}

/// Look up which zone a crew member should be in at a given hour/day.
pub fn get_crew_current_zone(crew_id: &str, hour: u32, day: &DayOfWeek) -> (AirportId, MapZone, i32, i32) {
    let schedule = get_schedule(crew_id);
    let entries = if matches!(day, DayOfWeek::Saturday | DayOfWeek::Sunday) {
        &schedule.weekend
    } else {
        &schedule.weekday
    };

    let active = entries.iter().rev().find(|e| e.hour <= hour);

    match active {
        Some(e) => (e.airport, e.zone, e.grid_x, e.grid_y),
        None => {
            let home = crew_home_airport(crew_id);
            (home, MapZone::CrewQuarters, 5, 5)
        }
    }
}

/// Check whether a given crew member should be visible at the player's location.
pub fn is_crew_present(crew_id: &str, hour: u32, day: &DayOfWeek, airport: AirportId, zone: MapZone) -> bool {
    let (a, z, _, _) = get_crew_current_zone(crew_id, hour, day);
    a == airport && z == zone
}
