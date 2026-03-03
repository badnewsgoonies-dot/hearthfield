//! Achievement definitions — all 35 achievements in Skywarden.

/// Full achievement definition with metadata for the tracker UI.
#[derive(Clone, Debug)]
pub struct AchievementEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub hidden: bool,
    pub unlock_condition: &'static str,
    pub icon_index: u32,
}

/// All achievements in the game.
pub const ALL_ACHIEVEMENTS: &[AchievementEntry] = &[
    // ── First milestones ──────────────────────────────────────────────
    AchievementEntry {
        id: "first_solo",
        title: "First Solo",
        description: "Complete your very first flight without an instructor.",
        hidden: false,
        unlock_condition: "Complete a flight with no instructor crew assigned",
        icon_index: 0,
    },
    AchievementEntry {
        id: "first_passenger",
        title: "First Passengers",
        description: "Carry passengers for the first time.",
        hidden: false,
        unlock_condition: "Complete a passenger mission",
        icon_index: 1,
    },
    AchievementEntry {
        id: "first_emergency",
        title: "Nerves of Steel",
        description: "Survive your first in-flight emergency.",
        hidden: false,
        unlock_condition: "Land safely after any EmergencyEvent",
        icon_index: 2,
    },
    // ── Flight count ──────────────────────────────────────────────────
    AchievementEntry {
        id: "flights_10",
        title: "Getting Started",
        description: "Complete 10 flights.",
        hidden: false,
        unlock_condition: "PlayStats::total_flights >= 10",
        icon_index: 3,
    },
    AchievementEntry {
        id: "flights_50",
        title: "Regular Flyer",
        description: "Complete 50 flights.",
        hidden: false,
        unlock_condition: "PlayStats::total_flights >= 50",
        icon_index: 4,
    },
    AchievementEntry {
        id: "flights_100",
        title: "Century Pilot",
        description: "Complete 100 flights.",
        hidden: false,
        unlock_condition: "PlayStats::total_flights >= 100",
        icon_index: 5,
    },
    AchievementEntry {
        id: "flights_500",
        title: "Sky Legend",
        description: "Complete 500 flights.",
        hidden: false,
        unlock_condition: "PlayStats::total_flights >= 500",
        icon_index: 6,
    },
    // ── Flight hours ──────────────────────────────────────────────────
    AchievementEntry {
        id: "hours_100",
        title: "Hundred-Hour Club",
        description: "Accumulate 100 hours of flight time.",
        hidden: false,
        unlock_condition: "PlayStats::total_flight_hours >= 100.0",
        icon_index: 7,
    },
    AchievementEntry {
        id: "hours_1000",
        title: "Thousand-Hour Veteran",
        description: "Accumulate 1,000 hours of flight time.",
        hidden: false,
        unlock_condition: "PlayStats::total_flight_hours >= 1000.0",
        icon_index: 8,
    },
    // ── Altitude ──────────────────────────────────────────────────────
    AchievementEntry {
        id: "mile_high",
        title: "Mile High Club",
        description: "Reach an altitude of 30,000 feet.",
        hidden: false,
        unlock_condition: "FlightState::altitude_ft >= 30000.0 during flight",
        icon_index: 9,
    },
    // ── Airports ──────────────────────────────────────────────────────
    AchievementEntry {
        id: "all_airports",
        title: "Globe Trotter",
        description: "Visit every airport in the network.",
        hidden: false,
        unlock_condition: "PlayStats::airports_visited contains all 10 AirportIds",
        icon_index: 10,
    },
    // ── Landings ──────────────────────────────────────────────────────
    AchievementEntry {
        id: "perfect_10",
        title: "Butter x10",
        description: "Make 10 perfect landings.",
        hidden: false,
        unlock_condition: "PlayStats::perfect_landings >= 10",
        icon_index: 11,
    },
    AchievementEntry {
        id: "perfect_streak_10",
        title: "Perfect Streak",
        description: "Land perfectly 10 times in a row without a single rough landing.",
        hidden: false,
        unlock_condition: "Track consecutive perfect landings >= 10",
        icon_index: 12,
    },
    AchievementEntry {
        id: "perfect_50",
        title: "Silk Touch",
        description: "Make 50 perfect landings.",
        hidden: false,
        unlock_condition: "PlayStats::perfect_landings >= 50",
        icon_index: 13,
    },
    // ── Night flying ─────────────────────────────────────────────────
    AchievementEntry {
        id: "night_owl",
        title: "Night Owl",
        description: "Complete 50 night flights.",
        hidden: false,
        unlock_condition: "Track night flights (departure or arrival after 20:00) >= 50",
        icon_index: 14,
    },
    // ── Weather ──────────────────────────────────────────────────────
    AchievementEntry {
        id: "storm_chaser",
        title: "Storm Chaser",
        description: "Fly through 20 storms and live to tell the tale.",
        hidden: false,
        unlock_condition: "Complete 20 flights where weather was Storm",
        icon_index: 15,
    },
    AchievementEntry {
        id: "fog_navigator",
        title: "Fog Navigator",
        description: "Land safely in fog 10 times.",
        hidden: false,
        unlock_condition: "Complete 10 flights where arrival weather was Fog",
        icon_index: 16,
    },
    // ── Crew & social ────────────────────────────────────────────────
    AchievementEntry {
        id: "all_crew_friends",
        title: "Crew Family",
        description: "Reach 'Best Friend' tier with every crew member.",
        hidden: false,
        unlock_condition: "All crew members at friendship >= 75",
        icon_index: 17,
    },
    AchievementEntry {
        id: "first_friend",
        title: "Wingmate",
        description: "Reach 'Good Friend' tier with any crew member.",
        hidden: false,
        unlock_condition: "Any crew member at friendship >= 50",
        icon_index: 18,
    },
    // ── Economy ──────────────────────────────────────────────────────
    AchievementEntry {
        id: "big_spender",
        title: "Big Spender",
        description: "Spend 500,000 gold total.",
        hidden: false,
        unlock_condition: "EconomyStats::total_spent >= 500000",
        icon_index: 19,
    },
    AchievementEntry {
        id: "penny_pincher",
        title: "Penny Pincher",
        description: "Accumulate 100,000 gold without spending any.",
        hidden: false,
        unlock_condition: "Gold::amount >= 100000 and EconomyStats::total_spent == 0 at check time",
        icon_index: 20,
    },
    AchievementEntry {
        id: "millionaire",
        title: "Sky Millionaire",
        description: "Earn 1,000,000 gold in total career earnings.",
        hidden: false,
        unlock_condition: "EconomyStats::total_earned >= 1000000",
        icon_index: 21,
    },
    // ── Speed & efficiency ───────────────────────────────────────────
    AchievementEntry {
        id: "speed_demon",
        title: "Speed Demon",
        description: "Reach maximum speed in a heavy jet.",
        hidden: false,
        unlock_condition: "FlightState::speed_knots >= aircraft max speed in HeavyJet",
        icon_index: 22,
    },
    AchievementEntry {
        id: "fuel_saver",
        title: "Eco Pilot",
        description: "Complete 10 flights using less than 70% of estimated fuel.",
        hidden: false,
        unlock_condition: "Track efficient flights where fuel_used < 0.7 * estimated",
        icon_index: 23,
    },
    // ── Rank progression ─────────────────────────────────────────────
    AchievementEntry {
        id: "rank_private",
        title: "Licensed Pilot",
        description: "Earn your Private Pilot license.",
        hidden: false,
        unlock_condition: "PilotState::rank >= Private",
        icon_index: 24,
    },
    AchievementEntry {
        id: "rank_commercial",
        title: "Going Commercial",
        description: "Reach Commercial Pilot rank.",
        hidden: false,
        unlock_condition: "PilotState::rank >= Commercial",
        icon_index: 25,
    },
    AchievementEntry {
        id: "rank_captain",
        title: "Captain!",
        description: "Reach Captain rank.",
        hidden: false,
        unlock_condition: "PilotState::rank >= Captain",
        icon_index: 26,
    },
    AchievementEntry {
        id: "rank_ace",
        title: "Ace of Aces",
        description: "Reach the legendary Ace rank.",
        hidden: false,
        unlock_condition: "PilotState::rank >= Ace",
        icon_index: 27,
    },
    // ── Mission types ────────────────────────────────────────────────
    AchievementEntry {
        id: "cargo_king",
        title: "Cargo King",
        description: "Deliver 100 cargo missions.",
        hidden: false,
        unlock_condition: "Count completed missions where type == Cargo >= 100",
        icon_index: 28,
    },
    AchievementEntry {
        id: "medical_hero",
        title: "Lifesaver",
        description: "Complete 10 medical transport flights.",
        hidden: false,
        unlock_condition: "Count completed missions where type == Medical >= 10",
        icon_index: 29,
    },
    AchievementEntry {
        id: "vip_service",
        title: "First Class",
        description: "Complete 20 VIP charter flights.",
        hidden: false,
        unlock_condition: "Count completed missions where type == VIP >= 20",
        icon_index: 30,
    },
    AchievementEntry {
        id: "rescue_5",
        title: "Guardian Angel",
        description: "Complete 5 rescue missions.",
        hidden: false,
        unlock_condition: "Count completed missions where type == Rescue >= 5",
        icon_index: 31,
    },
    // ── Aircraft ─────────────────────────────────────────────────────
    AchievementEntry {
        id: "all_aircraft",
        title: "Collector",
        description: "Own every type of aircraft.",
        hidden: false,
        unlock_condition: "Fleet contains one of every AircraftClass",
        icon_index: 32,
    },
    AchievementEntry {
        id: "all_licenses",
        title: "Full Ticket",
        description: "Earn every license type.",
        hidden: false,
        unlock_condition: "PilotState::licenses contains all LicenseType variants",
        icon_index: 33,
    },
    // ── Hidden / secret ──────────────────────────────────────────────
    AchievementEntry {
        id: "secret_canyon_run",
        title: "???",
        description: "A hidden achievement — discover it yourself!",
        hidden: true,
        unlock_condition: "Fly through Duskhollow canyon at under 200ft altitude",
        icon_index: 34,
    },
];

/// Look up an achievement by its id.
pub fn get_achievement(id: &str) -> Option<&'static AchievementEntry> {
    ALL_ACHIEVEMENTS.iter().find(|a| a.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn no_duplicate_achievement_ids() {
        let mut ids = HashSet::new();
        for a in ALL_ACHIEVEMENTS {
            assert!(ids.insert(a.id), "Duplicate achievement id: {}", a.id);
        }
    }

    #[test]
    fn at_least_30_achievements() {
        assert!(ALL_ACHIEVEMENTS.len() >= 30, "Need 30+ achievements, got {}", ALL_ACHIEVEMENTS.len());
    }

    #[test]
    fn all_fields_non_empty() {
        for a in ALL_ACHIEVEMENTS {
            assert!(!a.id.is_empty());
            assert!(!a.title.is_empty());
            assert!(!a.description.is_empty());
            assert!(!a.unlock_condition.is_empty());
        }
    }
}
