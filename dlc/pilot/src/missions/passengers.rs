//! Passenger satisfaction — boarding, in-flight comfort, deplaning ratings.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PassengerType {
    Business,
    Tourist,
    Family,
    VIP,
}

impl PassengerType {
    pub fn display_name(&self) -> &'static str {
        match self {
            PassengerType::Business => "Business",
            PassengerType::Tourist => "Tourist",
            PassengerType::Family => "Family",
            PassengerType::VIP => "VIP",
        }
    }

    pub fn patience_factor(&self) -> f32 {
        match self {
            PassengerType::Business => 0.6, // impatient
            PassengerType::Tourist => 1.0,
            PassengerType::Family => 0.8,
            PassengerType::VIP => 0.4, // very demanding
        }
    }

    pub fn tip_multiplier(&self) -> f32 {
        match self {
            PassengerType::Business => 1.5,
            PassengerType::Tourist => 1.0,
            PassengerType::Family => 0.8,
            PassengerType::VIP => 3.0,
        }
    }

    pub fn turbulence_sensitivity(&self) -> f32 {
        match self {
            PassengerType::Business => 1.0,
            PassengerType::Tourist => 0.8,
            PassengerType::Family => 1.5, // families hate turbulence
            PassengerType::VIP => 1.2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PassengerGroup {
    pub count: u32,
    pub passenger_type: PassengerType,
    pub satisfaction: f32, // 0.0–100.0
    pub patience: f32,     // 0.0–100.0
    pub boarded: bool,
}

impl PassengerGroup {
    pub fn new(count: u32, passenger_type: PassengerType) -> Self {
        Self {
            count,
            passenger_type,
            satisfaction: 80.0,
            patience: 100.0 * passenger_type.patience_factor(),
            boarded: false,
        }
    }

    pub fn tip_gold(&self) -> u32 {
        let base_tip = (self.satisfaction / 100.0) * 5.0 * self.count as f32;
        (base_tip * self.passenger_type.tip_multiplier()) as u32
    }

    pub fn reputation_impact(&self) -> f32 {
        (self.satisfaction - 50.0) / 25.0 // ranges roughly -2 to +2
    }
}

/// All passenger groups on the current flight.
#[derive(Resource, Clone, Debug, Default)]
pub struct PassengerManifest {
    pub groups: Vec<PassengerGroup>,
    pub boarding_progress: f32, // 0.0–1.0
    pub all_boarded: bool,
}

impl PassengerManifest {
    pub fn total_passengers(&self) -> u32 {
        self.groups.iter().map(|g| g.count).sum()
    }

    pub fn average_satisfaction(&self) -> f32 {
        if self.groups.is_empty() {
            return 100.0;
        }
        let total: f32 = self
            .groups
            .iter()
            .map(|g| g.satisfaction * g.count as f32)
            .sum();
        let count: f32 = self.groups.iter().map(|g| g.count as f32).sum();
        if count > 0.0 {
            total / count
        } else {
            100.0
        }
    }

    pub fn total_tips(&self) -> u32 {
        self.groups.iter().map(|g| g.tip_gold()).sum()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Board passengers during preflight.
pub fn board_passengers(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    mut manifest: ResMut<PassengerManifest>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase != FlightPhase::Preflight || manifest.all_boarded {
        return;
    }

    if manifest.groups.is_empty() {
        manifest.all_boarded = true;
        return;
    }

    let total = manifest.total_passengers();
    let board_speed = 10.0; // passengers per second
    manifest.boarding_progress += (board_speed / total.max(1) as f32) * time.delta_secs();

    if manifest.boarding_progress >= 1.0 {
        manifest.boarding_progress = 1.0;
        manifest.all_boarded = true;
        for group in &mut manifest.groups {
            group.boarded = true;
        }
        toast_events.send(ToastEvent {
            message: format!("All {} passengers boarded.", total),
            duration_secs: 2.0,
        });
    }
}

/// Update passenger satisfaction during flight — affected by turbulence, delays, comfort.
pub fn update_satisfaction(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    weather_state: Res<WeatherState>,
    mut manifest: ResMut<PassengerManifest>,
) {
    if flight_state.phase == FlightPhase::Idle || flight_state.phase == FlightPhase::Arrived {
        return;
    }
    if manifest.groups.is_empty() {
        return;
    }

    let turbulence_penalty = match weather_state.turbulence_level {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 0.5,
        TurbulenceLevel::Moderate => 2.0,
        TurbulenceLevel::Severe => 5.0,
    };

    // Patience drains slowly during flight
    let patience_drain = 0.3;

    let dt = time.delta_secs();

    for group in &mut manifest.groups {
        if !group.boarded {
            continue;
        }

        let turb_impact = turbulence_penalty * group.passenger_type.turbulence_sensitivity();
        group.satisfaction = (group.satisfaction - turb_impact * dt).max(0.0);
        group.patience = (group.patience - patience_drain * dt).max(0.0);

        // Low patience accelerates satisfaction decline
        if group.patience < 20.0 {
            group.satisfaction = (group.satisfaction - 1.0 * dt).max(0.0);
        }

        // Smooth cruise slightly increases satisfaction
        if weather_state.turbulence_level == TurbulenceLevel::None
            && flight_state.phase == FlightPhase::Cruise
        {
            group.satisfaction = (group.satisfaction + 0.2 * dt).min(100.0);
        }
    }
}

/// Deplane passengers after landing — calculate tips and reputation.
pub fn deplane_passengers(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut manifest: ResMut<PassengerManifest>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in flight_complete_events.read() {
        if manifest.groups.is_empty() {
            continue;
        }

        // Landing quality affects final satisfaction
        let landing_bonus = match ev.landing_grade.as_str() {
            "Perfect" => 10.0,
            "Good" => 5.0,
            "Acceptable" => 0.0,
            "Hard" => -10.0,
            "Rough" => -25.0,
            _ => 0.0,
        };

        for group in &mut manifest.groups {
            group.satisfaction = (group.satisfaction + landing_bonus).clamp(0.0, 100.0);
        }

        let avg_sat = manifest.average_satisfaction();
        let tips = manifest.total_tips();

        if tips > 0 {
            gold_events.send(GoldChangeEvent {
                amount: tips as i32,
                reason: "Passenger tips".to_string(),
            });
        }

        let rating = match avg_sat as u32 {
            80..=100 => "Delighted",
            60..=79 => "Satisfied",
            40..=59 => "Neutral",
            20..=39 => "Unhappy",
            _ => "Furious",
        };

        toast_events.send(ToastEvent {
            message: format!(
                "Passengers {} ({:.0}% avg satisfaction) — {}g tips",
                rating, avg_sat, tips
            ),
            duration_secs: 4.0,
        });

        // Reset for next flight
        manifest.groups.clear();
        manifest.boarding_progress = 0.0;
        manifest.all_boarded = false;
    }
}

/// Generate passenger groups when a passenger mission is accepted.
pub fn generate_passengers_for_mission(
    mut mission_accepted_events: EventReader<MissionAcceptedEvent>,
    mission_board: Res<MissionBoard>,
    mut manifest: ResMut<PassengerManifest>,
) {
    for _ev in mission_accepted_events.read() {
        if let Some(active) = &mission_board.active {
            if active.mission.passenger_count == 0 {
                continue;
            }

            manifest.groups.clear();
            manifest.boarding_progress = 0.0;
            manifest.all_boarded = false;

            let total = active.mission.passenger_count;
            let ptype = match active.mission.mission_type {
                MissionType::VIP => PassengerType::VIP,
                MissionType::Charter => PassengerType::Business,
                _ => PassengerType::Tourist,
            };

            // Split into 1-3 groups
            if total <= 5 {
                manifest.groups.push(PassengerGroup::new(total, ptype));
            } else {
                let half = total / 2;
                manifest.groups.push(PassengerGroup::new(half, ptype));
                manifest
                    .groups
                    .push(PassengerGroup::new(total - half, PassengerType::Tourist));
            }
        }
    }
}
