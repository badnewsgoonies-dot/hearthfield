//! Airline reputation system — per-airport and global reputation tracking.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReputationLevel {
    Unknown,
    Local,
    Regional,
    National,
    International,
    Legendary,
}

impl ReputationLevel {
    pub fn display_name(&self) -> &'static str {
        match self {
            ReputationLevel::Unknown => "Unknown",
            ReputationLevel::Local => "Local Pilot",
            ReputationLevel::Regional => "Regional Name",
            ReputationLevel::National => "National Recognition",
            ReputationLevel::International => "International Star",
            ReputationLevel::Legendary => "Living Legend",
        }
    }

    pub fn from_score(score: f32) -> Self {
        match score as u32 {
            0..=19 => ReputationLevel::Unknown,
            20..=39 => ReputationLevel::Local,
            40..=59 => ReputationLevel::Regional,
            60..=79 => ReputationLevel::National,
            80..=94 => ReputationLevel::International,
            _ => ReputationLevel::Legendary,
        }
    }

    pub fn mission_unlock_threshold(&self) -> bool {
        matches!(
            self,
            ReputationLevel::National | ReputationLevel::International | ReputationLevel::Legendary
        )
    }
}

/// Reputation resource — tracked per airport and globally.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Reputation {
    pub global_score: f32,
    pub airport_scores: HashMap<AirportId, f32>,
    pub last_flight_day: u32,
}

impl Default for Reputation {
    fn default() -> Self {
        Self {
            global_score: 30.0,
            airport_scores: HashMap::new(),
            last_flight_day: 0,
        }
    }
}

impl Reputation {
    pub fn global_level(&self) -> ReputationLevel {
        ReputationLevel::from_score(self.global_score)
    }

    pub fn airport_level(&self, airport: AirportId) -> ReputationLevel {
        let score = self.airport_scores.get(&airport).copied().unwrap_or(0.0);
        ReputationLevel::from_score(score)
    }

    pub fn airport_score(&self, airport: AirportId) -> f32 {
        self.airport_scores.get(&airport).copied().unwrap_or(0.0)
    }

    pub fn adjust_airport(&mut self, airport: AirportId, delta: f32) {
        let score = self.airport_scores.entry(airport).or_insert(20.0);
        *score = (*score + delta).clamp(0.0, 100.0);
        self.recalculate_global();
    }

    fn recalculate_global(&mut self) {
        if self.airport_scores.is_empty() {
            return;
        }
        let sum: f32 = self.airport_scores.values().sum();
        self.global_score = (sum / self.airport_scores.len() as f32).clamp(0.0, 100.0);
    }

    pub fn vip_missions_unlocked(&self) -> bool {
        self.global_level().mission_unlock_threshold()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Update reputation after each completed flight.
pub fn update_reputation_on_flight(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut reputation: ResMut<Reputation>,
    calendar: Res<Calendar>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in flight_complete_events.read() {
        reputation.last_flight_day = calendar.total_days();

        // Landing quality factor
        let landing_rep = match ev.landing_grade.as_str() {
            "Perfect" => 3.0,
            "Good" => 1.5,
            "Acceptable" => 0.5,
            "Hard" => -1.0,
            "Rough" => -3.0,
            _ => 0.0,
        };

        // On-time bonus (simplified: all flights currently count as on-time)
        let on_time_rep = 1.0;

        let total_delta = landing_rep + on_time_rep;
        reputation.adjust_airport(ev.destination, total_delta);
        reputation.adjust_airport(ev.origin, total_delta * 0.5);

        let level = reputation.global_level();
        toast_events.send(ToastEvent {
            message: format!(
                "Reputation: {} ({:.0})",
                level.display_name(),
                reputation.global_score
            ),
            duration_secs: 2.5,
        });
    }
}

/// Daily reputation decay if not flying regularly.
pub fn reputation_decay(
    mut day_end_events: EventReader<DayEndEvent>,
    mut reputation: ResMut<Reputation>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let days_since = calendar
            .total_days()
            .saturating_sub(reputation.last_flight_day);
        if days_since > 7 {
            let decay = (days_since as f32 - 7.0) * 0.3;
            let airports: Vec<AirportId> = reputation.airport_scores.keys().copied().collect();
            for airport in airports {
                reputation.adjust_airport(airport, -decay);
            }
        }
    }
}
