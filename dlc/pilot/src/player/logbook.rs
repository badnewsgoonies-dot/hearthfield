//! Flight logbook — records every completed flight for stat tracking and rank advancement.

use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::shared::*;
use std::collections::HashSet;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LogbookEntry {
    pub day: u32,
    pub season: Season,
    pub year: u32,
    pub origin: AirportId,
    pub destination: AirportId,
    pub aircraft_id: String,
    pub aircraft_class: AircraftClass,
    pub duration_minutes: f32,
    pub distance_nm: f32,
    pub weather: Weather,
    pub landing_grade: String,
    pub passengers: u32,
    pub cargo_kg: f32,
    pub fuel_used: f32,
    pub notes: String,
    pub was_night_flight: bool,
}

impl Default for LogbookEntry {
    fn default() -> Self {
        Self {
            day: 0,
            season: Season::Spring,
            year: 0,
            origin: AirportId::default(),
            destination: AirportId::default(),
            aircraft_id: String::new(),
            aircraft_class: AircraftClass::SingleProp,
            duration_minutes: 0.0,
            distance_nm: 0.0,
            weather: Weather::default(),
            landing_grade: String::new(),
            passengers: 0,
            cargo_kg: 0.0,
            fuel_used: 0.0,
            notes: String::new(),
            was_night_flight: false,
        }
    }
}

/// Persistent flight logbook resource.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Logbook {
    pub entries: Vec<LogbookEntry>,
    pub total_hours: f32,
    pub total_distance_nm: f32,
    pub total_passengers: u32,
    pub total_cargo_kg: f32,
}

impl Logbook {
    pub fn record(&mut self, entry: LogbookEntry) {
        self.total_hours += entry.duration_minutes / 60.0;
        self.total_distance_nm += entry.distance_nm;
        self.total_passengers += entry.passengers;
        self.total_cargo_kg += entry.cargo_kg;
        self.entries.push(entry);
    }

    pub fn best_landing(&self) -> Option<&LogbookEntry> {
        self.entries.iter().find(|e| e.landing_grade == "Perfect")
    }

    pub fn longest_flight(&self) -> Option<&LogbookEntry> {
        self.entries
            .iter()
            .max_by(|a, b| a.duration_minutes.partial_cmp(&b.duration_minutes).unwrap_or(std::cmp::Ordering::Equal))
    }

    pub fn most_passengers_flight(&self) -> Option<&LogbookEntry> {
        self.entries.iter().max_by_key(|e| e.passengers)
    }

    pub fn airports_visited(&self) -> HashSet<AirportId> {
        let mut set = HashSet::new();
        for e in &self.entries {
            set.insert(e.origin);
            set.insert(e.destination);
        }
        set
    }

    pub fn hours_in_class(&self, class: AircraftClass) -> f32 {
        self.entries
            .iter()
            .filter(|e| e.aircraft_class == class)
            .map(|e| e.duration_minutes / 60.0)
            .sum()
    }

    pub fn perfect_landing_count(&self) -> usize {
        self.entries.iter().filter(|e| e.landing_grade == "Perfect").count()
    }

    pub fn total_flights(&self) -> usize {
        self.entries.len()
    }

    pub fn flights_this_season(&self, season: Season, year: u32) -> usize {
        self.entries
            .iter()
            .filter(|e| e.season == season && e.year == year)
            .count()
    }

    /// Minimum hours required per aircraft class for rank advancement.
    pub fn meets_rank_hours(&self, rank: PilotRank) -> bool {
        match rank {
            PilotRank::Student => true,
            PilotRank::Private => self.total_hours >= 5.0,
            PilotRank::Commercial => {
                self.total_hours >= 20.0
                    && self.hours_in_class(AircraftClass::TwinProp) >= 3.0
            }
            PilotRank::Senior => {
                self.total_hours >= 50.0
                    && self.hours_in_class(AircraftClass::Turboprop) >= 5.0
            }
            PilotRank::Captain => {
                self.total_hours >= 100.0
                    && self.hours_in_class(AircraftClass::MediumJet) >= 10.0
            }
            PilotRank::Ace => {
                self.total_hours >= 200.0
                    && self.hours_in_class(AircraftClass::HeavyJet) >= 20.0
            }
        }
    }

    /// Recent flight count (last 7 game-days).
    pub fn recent_flights(&self, current_day: u32) -> usize {
        self.entries
            .iter()
            .filter(|e| current_day.saturating_sub(e.day) <= 7)
            .count()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Record a logbook entry after each completed flight.
pub fn record_flight(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut logbook: ResMut<Logbook>,
    calendar: Res<Calendar>,
    fleet: Res<Fleet>,
    weather_state: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in flight_complete_events.read() {
        let (aircraft_id, aircraft_class) = fleet
            .active()
            .map(|a| (a.aircraft_id.clone(), AircraftClass::SingleProp)) // class from registry in real impl
            .unwrap_or_else(|| ("unknown".to_string(), AircraftClass::SingleProp));

        let entry = LogbookEntry {
            day: calendar.total_days(),
            season: calendar.season,
            year: calendar.year,
            origin: ev.origin,
            destination: ev.destination,
            aircraft_id,
            aircraft_class,
            duration_minutes: ev.flight_time_secs / 60.0,
            distance_nm: flight_state.distance_total_nm,
            weather: weather_state.current,
            landing_grade: ev.landing_grade.clone(),
            passengers: flight_state.passengers_happy as u32, // simplified
            cargo_kg: 0.0,
            fuel_used: ev.fuel_used,
            notes: String::new(),
            was_night_flight: calendar.is_night(),
        };

        logbook.record(entry);

        toast_events.send(ToastEvent {
            message: format!(
                "✈ Flight logged — total hours: {:.1}",
                logbook.total_hours
            ),
            duration_secs: 3.0,
        });
    }
}

/// Enforce logbook requirements for rank advancement.
pub fn check_logbook_rank_requirement(
    logbook: Res<Logbook>,
    _pilot_state: Res<PilotState>,
    mut toast_events: EventWriter<ToastEvent>,
    mut rank_up_events: EventReader<RankUpEvent>,
) {
    for ev in rank_up_events.read() {
        if !logbook.meets_rank_hours(ev.new_rank) {
            toast_events.send(ToastEvent {
                message: "Need more logged hours for this rank!".to_string(),
                duration_secs: 3.0,
            });
        }
    }
}
