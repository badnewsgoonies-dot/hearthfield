//! Pilot skill progression — XP-based leveling for aviation competencies.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::shared::*;
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillType {
    Navigation,
    WeatherReading,
    LandingPrecision,
    FuelManagement,
    Communication,
    EmergencyResponse,
    Leadership,
    NightFlying,
}

impl SkillType {
    pub const ALL: &'static [SkillType] = &[
        SkillType::Navigation,
        SkillType::WeatherReading,
        SkillType::LandingPrecision,
        SkillType::FuelManagement,
        SkillType::Communication,
        SkillType::EmergencyResponse,
        SkillType::Leadership,
        SkillType::NightFlying,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            SkillType::Navigation => "Navigation",
            SkillType::WeatherReading => "Weather Reading",
            SkillType::LandingPrecision => "Landing Precision",
            SkillType::FuelManagement => "Fuel Management",
            SkillType::Communication => "Communication",
            SkillType::EmergencyResponse => "Emergency Response",
            SkillType::Leadership => "Leadership",
            SkillType::NightFlying => "Night Flying",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SkillType::Navigation => "Reduces waypoint deviation and improves routing.",
            SkillType::WeatherReading => "Provides more accurate weather forecasts.",
            SkillType::LandingPrecision => "Improves landing grades and reduces damage.",
            SkillType::FuelManagement => "Decreases fuel burn rate during flights.",
            SkillType::Communication => "Unlocks better dialogue options with ATC and crew.",
            SkillType::EmergencyResponse => "Improves outcomes during emergencies.",
            SkillType::Leadership => "Increases crew morale and passenger satisfaction.",
            SkillType::NightFlying => "Reduces difficulty penalties for night operations.",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SkillState {
    pub level: u32,       // 0–100
    pub xp: u32,
    pub xp_to_next: u32,
    pub last_used_day: u32,
}

impl Default for SkillState {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_to_next: 100,
            last_used_day: 0,
        }
    }
}

impl SkillState {
    pub fn add_xp(&mut self, amount: u32) {
        self.xp += amount;
        while self.xp >= self.xp_to_next && self.level < 100 {
            self.xp -= self.xp_to_next;
            self.level += 1;
            // XP curve: each level requires progressively more
            self.xp_to_next = 100 + self.level * 20;
        }
        if self.level >= 100 {
            self.xp = 0;
        }
    }

    pub fn effectiveness(&self) -> f32 {
        self.level as f32 / 100.0
    }
}

/// Resource holding all pilot skills.
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct PilotSkills {
    pub skills: HashMap<SkillType, SkillState>,
}

impl Default for PilotSkills {
    fn default() -> Self {
        let mut skills = HashMap::new();
        for &skill in SkillType::ALL {
            skills.insert(skill, SkillState::default());
        }
        Self { skills }
    }
}

impl PilotSkills {
    pub fn level(&self, skill: SkillType) -> u32 {
        self.skills.get(&skill).map_or(1, |s| s.level)
    }

    pub fn effectiveness(&self, skill: SkillType) -> f32 {
        self.skills.get(&skill).map_or(0.01, |s| s.effectiveness())
    }

    pub fn add_xp(&mut self, skill: SkillType, amount: u32, current_day: u32) {
        if let Some(state) = self.skills.get_mut(&skill) {
            state.add_xp(amount);
            state.last_used_day = current_day;
        }
    }

    pub fn navigation_deviation_factor(&self) -> f32 {
        1.0 - self.effectiveness(SkillType::Navigation) * 0.8
    }

    pub fn forecast_accuracy_bonus(&self) -> f32 {
        self.effectiveness(SkillType::WeatherReading) * 0.5
    }

    pub fn landing_grade_bonus(&self) -> f32 {
        self.effectiveness(SkillType::LandingPrecision) * 30.0
    }

    pub fn fuel_efficiency_bonus(&self) -> f32 {
        self.effectiveness(SkillType::FuelManagement) * 0.2
    }

    pub fn night_penalty_reduction(&self) -> f32 {
        self.effectiveness(SkillType::NightFlying) * 0.7
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Award skill XP after a completed flight based on flight conditions.
pub fn award_flight_skill_xp(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut skills: ResMut<PilotSkills>,
    weather_state: Res<WeatherState>,
    calendar: Res<Calendar>,
    flight_state: Res<FlightState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in flight_complete_events.read() {
        let day = calendar.total_days();

        // Navigation XP — always awarded
        skills.add_xp(SkillType::Navigation, 15, day);

        // Landing precision — grade-dependent
        let landing_xp = match ev.landing_grade.as_str() {
            "Perfect" => 30,
            "Good" => 20,
            "Acceptable" => 10,
            _ => 5,
        };
        skills.add_xp(SkillType::LandingPrecision, landing_xp, day);

        // Fuel management — based on fuel efficiency
        let fuel_efficiency = if flight_state.distance_total_nm > 0.0 {
            ev.fuel_used / flight_state.distance_total_nm
        } else {
            1.0
        };
        if fuel_efficiency < 0.5 {
            skills.add_xp(SkillType::FuelManagement, 20, day);
        } else {
            skills.add_xp(SkillType::FuelManagement, 8, day);
        }

        // Weather reading — bonus in bad weather
        if weather_state.current.flight_difficulty() > 0.3 {
            skills.add_xp(SkillType::WeatherReading, 20, day);
        }

        // Night flying
        if calendar.is_night() {
            skills.add_xp(SkillType::NightFlying, 25, day);
        }

        // Communication — always a little
        skills.add_xp(SkillType::Communication, 10, day);

        toast_events.send(ToastEvent {
            message: "✦ Skills improved from flight experience.".to_string(),
            duration_secs: 2.5,
        });
    }
}

/// Award emergency response XP when handling emergencies.
pub fn award_emergency_xp(
    mut emergency_events: EventReader<EmergencyEvent>,
    mut skills: ResMut<PilotSkills>,
    calendar: Res<Calendar>,
) {
    for _ev in emergency_events.read() {
        skills.add_xp(SkillType::EmergencyResponse, 40, calendar.total_days());
    }
}

/// Daily practice bonus — recently-used skills get a small XP bump at day end.
pub fn daily_practice_bonus(
    mut day_end_events: EventReader<DayEndEvent>,
    mut skills: ResMut<PilotSkills>,
    calendar: Res<Calendar>,
) {
    for _ev in day_end_events.read() {
        let today = calendar.total_days();
        let recent: Vec<SkillType> = skills
            .skills
            .iter()
            .filter(|(_, state)| today.saturating_sub(state.last_used_day) <= 2)
            .map(|(&skill, _)| skill)
            .collect();

        for skill in recent {
            skills.add_xp(skill, 5, today);
        }
    }
}

/// Training at flight schools — triggered via interaction (placeholder hook).
pub fn flight_school_training(
    player_input: Res<PlayerInput>,
    location: Res<PlayerLocation>,
    mut skills: ResMut<PilotSkills>,
    mut gold: ResMut<Gold>,
    calendar: Res<Calendar>,
    mut toast_events: EventWriter<ToastEvent>,
    mut claimed: ResMut<InteractionClaimed>,
) {
    if !player_input.interact || claimed.0 {
        return;
    }
    // Flight school available in control tower zone
    if location.zone != MapZone::ControlTower {
        return;
    }

    let training_cost: u32 = 200;
    if gold.amount < training_cost {
        return;
    }

    claimed.0 = true;
    gold.amount -= training_cost;

    let day = calendar.total_days();
    for &skill in SkillType::ALL {
        skills.add_xp(skill, 10, day);
    }

    toast_events.send(ToastEvent {
        message: format!("Flight school training complete! (-{}g)", training_cost),
        duration_secs: 3.0,
    });
}
