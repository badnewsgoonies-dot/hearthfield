//! Annual event calendar — festivals, special events, seasonal activities.

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct CalendarEvent {
    pub name: String,
    pub day_start: u32,
    pub day_end: u32,
    pub season: Option<Season>,
    pub airports: Vec<AirportId>,
    pub effects: Vec<EventEffect>,
    pub description: String,
}

impl CalendarEvent {
    pub fn is_active(&self, day: u32, season: Season) -> bool {
        let season_match = self.season.is_none_or(|s| s == season);
        season_match && day >= self.day_start && day <= self.day_end
    }

    pub fn display_text(&self) -> String {
        let airports_str: Vec<&str> = self.airports.iter().map(|a| a.display_name()).collect();
        format!(
            "{} (Day {}-{}) — {}",
            self.name,
            self.day_start,
            self.day_end,
            airports_str.join(", ")
        )
    }
}

#[derive(Clone, Debug)]
pub enum EventEffect {
    MissionBonus { multiplier: f32 },
    PriceModifier { category: String, multiplier: f32 },
    CrewMoodBoost { amount: i32 },
    SpecialMissionsAvailable,
    WeatherGuarantee { weather: Weather },
    PassengerDemandBoost { multiplier: f32 },
    XpBonus { multiplier: f32 },
    ShopDiscount { percent: f32 },
}

impl EventEffect {
    pub fn description(&self) -> String {
        match self {
            EventEffect::MissionBonus { multiplier } => {
                format!("Mission rewards ×{:.1}", multiplier)
            }
            EventEffect::PriceModifier { category, multiplier } => {
                format!("{} prices ×{:.1}", category, multiplier)
            }
            EventEffect::CrewMoodBoost { amount } => {
                format!("Crew mood +{}", amount)
            }
            EventEffect::SpecialMissionsAvailable => {
                "Special event missions available".to_string()
            }
            EventEffect::WeatherGuarantee { weather } => {
                format!("Guaranteed {:?} weather", weather)
            }
            EventEffect::PassengerDemandBoost { multiplier } => {
                format!("Passenger demand ×{:.1}", multiplier)
            }
            EventEffect::XpBonus { multiplier } => {
                format!("XP gain ×{:.1}", multiplier)
            }
            EventEffect::ShopDiscount { percent } => {
                format!("Shop prices -{:.0}%", percent)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EVENT DATABASE
// ═══════════════════════════════════════════════════════════════════════════

/// Get all annual calendar events.
pub fn all_calendar_events() -> Vec<CalendarEvent> {
    vec![
        // ── Spring (Days 1-28) ───────────────────────────────────────────
        CalendarEvent {
            name: "New Year's Air Show".into(),
            day_start: 1,
            day_end: 3,
            season: Some(Season::Spring),
            airports: vec![AirportId::HomeBase],
            effects: vec![
                EventEffect::SpecialMissionsAvailable,
                EventEffect::XpBonus { multiplier: 1.5 },
                EventEffect::WeatherGuarantee { weather: Weather::Clear },
            ],
            description: "Clearfield Regional hosts the annual New Year's Air Show! Aerobatic displays, vintage flyovers, and special exhibition missions.".into(),
        },
        CalendarEvent {
            name: "Spring Aviation Festival".into(),
            day_start: 7,
            day_end: 10,
            season: Some(Season::Spring),
            airports: vec![AirportId::HomeBase, AirportId::Windport, AirportId::Grandcity],
            effects: vec![
                EventEffect::MissionBonus { multiplier: 1.3 },
                EventEffect::PassengerDemandBoost { multiplier: 1.5 },
                EventEffect::ShopDiscount { percent: 15.0 },
            ],
            description: "The Spring Aviation Festival celebrates the start of flying season. Increased demand, special missions, and shop discounts across multiple airports.".into(),
        },
        CalendarEvent {
            name: "Cherry Blossom Flights".into(),
            day_start: 14,
            day_end: 16,
            season: Some(Season::Spring),
            airports: vec![AirportId::Sunhaven, AirportId::Windport],
            effects: vec![
                EventEffect::PassengerDemandBoost { multiplier: 2.0 },
                EventEffect::MissionBonus { multiplier: 1.2 },
            ],
            description: "Cherry blossoms bloom along the coast. Scenic flights are in high demand as tourists flock to see the flowers from above.".into(),
        },
        CalendarEvent {
            name: "Mountain Rescue Training Week".into(),
            day_start: 20,
            day_end: 24,
            season: Some(Season::Spring),
            airports: vec![AirportId::Frostpeak, AirportId::Cloudmere],
            effects: vec![
                EventEffect::XpBonus { multiplier: 1.5 },
                EventEffect::SpecialMissionsAvailable,
            ],
            description: "Annual mountain rescue training exercises at Frostpeak and Cloudmere. Extra XP for rescue and training missions.".into(),
        },
        // ── Summer (Days 1-28) ───────────────────────────────────────────
        CalendarEvent {
            name: "Summer Beach Flights".into(),
            day_start: 1,
            day_end: 7,
            season: Some(Season::Summer),
            airports: vec![AirportId::Sunhaven],
            effects: vec![
                EventEffect::PassengerDemandBoost { multiplier: 2.0 },
                EventEffect::MissionBonus { multiplier: 1.4 },
                EventEffect::WeatherGuarantee { weather: Weather::Clear },
            ],
            description: "Peak tourist season at Sunhaven! Beach-bound flights are packed, and the weather is perfect for flying.".into(),
        },
        CalendarEvent {
            name: "Grand City Air Race".into(),
            day_start: 10,
            day_end: 12,
            season: Some(Season::Summer),
            airports: vec![AirportId::Grandcity, AirportId::HomeBase],
            effects: vec![
                EventEffect::SpecialMissionsAvailable,
                EventEffect::XpBonus { multiplier: 2.0 },
                EventEffect::CrewMoodBoost { amount: 5 },
            ],
            description: "The annual Grand City Air Race! Compete for the fastest time on a marked course. The whole crew is excited.".into(),
        },
        CalendarEvent {
            name: "Desert Sunset Festival".into(),
            day_start: 15,
            day_end: 18,
            season: Some(Season::Summer),
            airports: vec![AirportId::Duskhollow],
            effects: vec![
                EventEffect::PassengerDemandBoost { multiplier: 1.8 },
                EventEffect::MissionBonus { multiplier: 1.3 },
            ],
            description: "Duskhollow's annual sunset festival draws photographers and tourists. Scenic flight demand soars.".into(),
        },
        CalendarEvent {
            name: "Charity Flight Day".into(),
            day_start: 22,
            day_end: 22,
            season: Some(Season::Summer),
            airports: vec![AirportId::HomeBase, AirportId::Grandcity, AirportId::Sunhaven],
            effects: vec![
                EventEffect::XpBonus { multiplier: 1.5 },
                EventEffect::CrewMoodBoost { amount: 3 },
            ],
            description: "Annual charity flight day. All mission XP is boosted, and crew morale improves from the spirit of giving.".into(),
        },
        // ── Fall (Days 1-28) ─────────────────────────────────────────────
        CalendarEvent {
            name: "Harvest Cargo Rush".into(),
            day_start: 1,
            day_end: 5,
            season: Some(Season::Fall),
            airports: vec![AirportId::Ironforge, AirportId::Duskhollow],
            effects: vec![
                EventEffect::MissionBonus { multiplier: 1.5 },
                EventEffect::PriceModifier { category: "cargo".into(), multiplier: 1.4 },
            ],
            description: "Harvest season creates a cargo boom! Industrial and agricultural shipments spike between Ironforge and Duskhollow.".into(),
        },
        CalendarEvent {
            name: "Storm Season Opens".into(),
            day_start: 8,
            day_end: 14,
            season: Some(Season::Fall),
            airports: vec![AirportId::Stormwatch],
            effects: vec![
                EventEffect::SpecialMissionsAvailable,
                EventEffect::XpBonus { multiplier: 1.8 },
                EventEffect::MissionBonus { multiplier: 1.6 },
            ],
            description: "Peak storm season at Stormwatch. Weather research flights pay double, but conditions are treacherous.".into(),
        },
        CalendarEvent {
            name: "Full Moon Night Flight".into(),
            day_start: 14,
            day_end: 14,
            season: Some(Season::Fall),
            airports: vec![AirportId::HomeBase, AirportId::Cloudmere],
            effects: vec![
                EventEffect::XpBonus { multiplier: 1.3 },
                EventEffect::SpecialMissionsAvailable,
            ],
            description: "A beautiful full moon illuminates the sky. Night flights earn bonus XP tonight.".into(),
        },
        CalendarEvent {
            name: "Aviation Heritage Week".into(),
            day_start: 18,
            day_end: 22,
            season: Some(Season::Fall),
            airports: vec![AirportId::Ironforge, AirportId::Skyreach],
            effects: vec![
                EventEffect::SpecialMissionsAvailable,
                EventEffect::ShopDiscount { percent: 20.0 },
                EventEffect::CrewMoodBoost { amount: 3 },
            ],
            description: "Celebrating aviation history! Vintage aircraft displays, historical delivery missions, and heritage shop discounts.".into(),
        },
        CalendarEvent {
            name: "Foliage Scenic Flights".into(),
            day_start: 24,
            day_end: 28,
            season: Some(Season::Fall),
            airports: vec![AirportId::Frostpeak, AirportId::HomeBase, AirportId::Cloudmere],
            effects: vec![
                EventEffect::PassengerDemandBoost { multiplier: 1.6 },
            ],
            description: "Autumn foliage creates stunning views from the air. Scenic flight demand peaks as tourists want aerial leaf-peeping.".into(),
        },
        // ── Winter (Days 1-28) ───────────────────────────────────────────
        CalendarEvent {
            name: "Winter Holiday Flights".into(),
            day_start: 1,
            day_end: 5,
            season: Some(Season::Winter),
            airports: vec![
                AirportId::HomeBase, AirportId::Windport, AirportId::Sunhaven,
                AirportId::Grandcity, AirportId::Skyreach,
            ],
            effects: vec![
                EventEffect::PassengerDemandBoost { multiplier: 2.0 },
                EventEffect::MissionBonus { multiplier: 1.5 },
                EventEffect::CrewMoodBoost { amount: 5 },
            ],
            description: "The holiday travel rush! Maximum passenger demand across all major airports. Crew morale is high.".into(),
        },
        CalendarEvent {
            name: "Frostpeak Ski Season".into(),
            day_start: 5,
            day_end: 20,
            season: Some(Season::Winter),
            airports: vec![AirportId::Frostpeak],
            effects: vec![
                EventEffect::PassengerDemandBoost { multiplier: 1.8 },
                EventEffect::MissionBonus { multiplier: 1.2 },
            ],
            description: "Ski season at Frostpeak! Steady stream of passengers heading to the slopes. Watch for icing conditions.".into(),
        },
        CalendarEvent {
            name: "Pilot Appreciation Day".into(),
            day_start: 14,
            day_end: 14,
            season: Some(Season::Winter),
            airports: vec![AirportId::HomeBase],
            effects: vec![
                EventEffect::XpBonus { multiplier: 2.0 },
                EventEffect::ShopDiscount { percent: 25.0 },
                EventEffect::CrewMoodBoost { amount: 10 },
            ],
            description: "A day to celebrate pilots everywhere! Double XP, major shop discounts, and the crew throws a party.".into(),
        },
        CalendarEvent {
            name: "New Year's Eve Gala".into(),
            day_start: 28,
            day_end: 28,
            season: Some(Season::Winter),
            airports: vec![AirportId::Grandcity, AirportId::Skyreach],
            effects: vec![
                EventEffect::SpecialMissionsAvailable,
                EventEffect::MissionBonus { multiplier: 2.0 },
                EventEffect::CrewMoodBoost { amount: 5 },
            ],
            description: "VIP flights for the New Year's Eve gala at Grand City and Skyreach. Premium mission payouts all day.".into(),
        },
        // ── Monthly recurring events (any season) ────────────────────────
        CalendarEvent {
            name: "Full Moon Night Flight".into(),
            day_start: 14,
            day_end: 14,
            season: None, // Every season
            airports: vec![AirportId::HomeBase, AirportId::Cloudmere],
            effects: vec![
                EventEffect::XpBonus { multiplier: 1.3 },
            ],
            description: "Night flights under the full moon earn bonus XP.".into(),
        },
        CalendarEvent {
            name: "Air Race Day".into(),
            day_start: 21,
            day_end: 21,
            season: None,
            airports: vec![AirportId::HomeBase],
            effects: vec![
                EventEffect::SpecialMissionsAvailable,
                EventEffect::XpBonus { multiplier: 1.5 },
            ],
            description: "Monthly air race competition at Clearfield. Beat your best time!".into(),
        },
        CalendarEvent {
            name: "Charity Flight".into(),
            day_start: 7,
            day_end: 7,
            season: None,
            airports: vec![AirportId::HomeBase, AirportId::Grandcity],
            effects: vec![
                EventEffect::XpBonus { multiplier: 1.2 },
                EventEffect::CrewMoodBoost { amount: 2 },
            ],
            description: "Weekly charity flight — donate your time for a good cause and earn extra XP.".into(),
        },
    ]
}

/// Get events active on a given day and season.
pub fn active_events(day: u32, season: Season) -> Vec<CalendarEvent> {
    all_calendar_events()
        .into_iter()
        .filter(|e| e.is_active(day, season))
        .collect()
}

/// Get upcoming events within the next N days.
pub fn upcoming_events(current_day: u32, season: Season, lookahead: u32) -> Vec<CalendarEvent> {
    all_calendar_events()
        .into_iter()
        .filter(|e| {
            let season_match = e.season.is_none_or(|s| s == season);
            season_match && e.day_start > current_day && e.day_start <= current_day + lookahead
        })
        .collect()
}

/// Get total XP multiplier from active events.
pub fn event_xp_multiplier(day: u32, season: Season) -> f32 {
    let mut mult = 1.0;
    for event in active_events(day, season) {
        for effect in &event.effects {
            if let EventEffect::XpBonus { multiplier } = effect {
                mult *= multiplier;
            }
        }
    }
    mult
}

/// Get total passenger demand multiplier from active events.
pub fn event_passenger_demand(day: u32, season: Season) -> f32 {
    let mut mult = 1.0;
    for event in active_events(day, season) {
        for effect in &event.effects {
            if let EventEffect::PassengerDemandBoost { multiplier } = effect {
                mult *= multiplier;
            }
        }
    }
    mult
}
