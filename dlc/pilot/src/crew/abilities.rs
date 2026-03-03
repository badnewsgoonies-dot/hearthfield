//! Crew special abilities — passive bonuses activated at high friendship.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AbilityId {
    ElenaXpBoost,
    MarcoFuelEfficiency,
    YukiNavPrecision,
    HankMaintenance,
    SofiaPassengerSat,
    RajAtcSpeed,
    ChenSkillXp,
    PeteSurvival,
    AlexLandingXp,
    DianaCharterPay,
}

impl AbilityId {
    pub fn npc_id(&self) -> &'static str {
        match self {
            AbilityId::ElenaXpBoost => "captain_elena",
            AbilityId::MarcoFuelEfficiency => "copilot_marco",
            AbilityId::YukiNavPrecision => "navigator_yuki",
            AbilityId::HankMaintenance => "mechanic_hank",
            AbilityId::SofiaPassengerSat => "attendant_sofia",
            AbilityId::RajAtcSpeed => "controller_raj",
            AbilityId::ChenSkillXp => "instructor_chen",
            AbilityId::PeteSurvival => "veteran_pete",
            AbilityId::AlexLandingXp => "rookie_alex",
            AbilityId::DianaCharterPay => "charter_diana",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AbilityId::ElenaXpBoost => "Veteran Wisdom",
            AbilityId::MarcoFuelEfficiency => "Efficient Copilot",
            AbilityId::YukiNavPrecision => "Precise Navigator",
            AbilityId::HankMaintenance => "Expert Mechanic",
            AbilityId::SofiaPassengerSat => "Hospitality Master",
            AbilityId::RajAtcSpeed => "Insider Knowledge",
            AbilityId::ChenSkillXp => "Good Teacher",
            AbilityId::PeteSurvival => "Storm Survivor",
            AbilityId::AlexLandingXp => "Quick Learner",
            AbilityId::DianaCharterPay => "Smooth Negotiator",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AbilityId::ElenaXpBoost => "+10% XP from all flights",
            AbilityId::MarcoFuelEfficiency => "Fuel burn reduced by 5%",
            AbilityId::YukiNavPrecision => "Navigation deviation reduced 20%",
            AbilityId::HankMaintenance => "Maintenance costs -15%, condition loss -10%",
            AbilityId::SofiaPassengerSat => "Passenger satisfaction +15%",
            AbilityId::RajAtcSpeed => "ATC clearance waiting time reduced",
            AbilityId::ChenSkillXp => "Skill XP gain +20%",
            AbilityId::PeteSurvival => "Emergency survival chance +25%",
            AbilityId::AlexLandingXp => "Landing XP bonus +10%",
            AbilityId::DianaCharterPay => "Charter mission pay +20%",
        }
    }

    pub fn all() -> &'static [AbilityId] {
        &[
            AbilityId::ElenaXpBoost,
            AbilityId::MarcoFuelEfficiency,
            AbilityId::YukiNavPrecision,
            AbilityId::HankMaintenance,
            AbilityId::SofiaPassengerSat,
            AbilityId::RajAtcSpeed,
            AbilityId::ChenSkillXp,
            AbilityId::PeteSurvival,
            AbilityId::AlexLandingXp,
            AbilityId::DianaCharterPay,
        ]
    }
}

/// Required friendship level for ability activation.
const ABILITY_THRESHOLD: i32 = 55; // CloseFriend phase starts at 55

/// Active crew ability bonuses.
#[derive(Resource, Clone, Debug, Default)]
pub struct CrewBonuses {
    pub active_abilities: Vec<AbilityId>,
    pub xp_multiplier: f32,
    pub fuel_multiplier: f32,
    pub nav_precision_bonus: f32,
    pub maintenance_discount: f32,
    pub condition_loss_reduction: f32,
    pub passenger_satisfaction_bonus: f32,
    pub atc_speed_bonus: f32,
    pub skill_xp_multiplier: f32,
    pub survival_bonus: f32,
    pub landing_xp_bonus: f32,
    pub charter_pay_bonus: f32,
}

impl CrewBonuses {
    pub fn recalculate(&mut self) {
        self.xp_multiplier = 1.0;
        self.fuel_multiplier = 1.0;
        self.nav_precision_bonus = 0.0;
        self.maintenance_discount = 0.0;
        self.condition_loss_reduction = 0.0;
        self.passenger_satisfaction_bonus = 0.0;
        self.atc_speed_bonus = 0.0;
        self.skill_xp_multiplier = 1.0;
        self.survival_bonus = 0.0;
        self.landing_xp_bonus = 0.0;
        self.charter_pay_bonus = 0.0;

        for ability in &self.active_abilities {
            match ability {
                AbilityId::ElenaXpBoost => self.xp_multiplier += 0.10,
                AbilityId::MarcoFuelEfficiency => self.fuel_multiplier -= 0.05,
                AbilityId::YukiNavPrecision => self.nav_precision_bonus += 0.20,
                AbilityId::HankMaintenance => {
                    self.maintenance_discount += 0.15;
                    self.condition_loss_reduction += 0.10;
                }
                AbilityId::SofiaPassengerSat => self.passenger_satisfaction_bonus += 0.15,
                AbilityId::RajAtcSpeed => self.atc_speed_bonus += 0.30,
                AbilityId::ChenSkillXp => self.skill_xp_multiplier += 0.20,
                AbilityId::PeteSurvival => self.survival_bonus += 0.25,
                AbilityId::AlexLandingXp => self.landing_xp_bonus += 0.10,
                AbilityId::DianaCharterPay => self.charter_pay_bonus += 0.20,
            }
        }
    }

    pub fn active_count(&self) -> usize {
        self.active_abilities.len()
    }

    pub fn is_active(&self, ability: AbilityId) -> bool {
        self.active_abilities.contains(&ability)
    }

    pub fn summary_lines(&self) -> Vec<String> {
        self.active_abilities
            .iter()
            .map(|a| format!("{}: {}", a.display_name(), a.description()))
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Evaluate which crew abilities should be active based on friendship levels.
pub fn evaluate_crew_abilities(
    relationships: Res<Relationships>,
    mut bonuses: ResMut<CrewBonuses>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let prev_active: Vec<AbilityId> = bonuses.active_abilities.clone();
    bonuses.active_abilities.clear();

    for ability in AbilityId::all() {
        let npc_id = ability.npc_id();
        let friendship = relationships.friendship_level(npc_id);

        if friendship >= ABILITY_THRESHOLD {
            bonuses.active_abilities.push(*ability);
        }
    }

    bonuses.recalculate();

    // Notify on newly activated abilities
    for ability in &bonuses.active_abilities {
        if !prev_active.contains(ability) {
            toast_events.send(ToastEvent {
                message: format!(
                    "🔓 {} activated: {} — {}",
                    ability.npc_id(),
                    ability.display_name(),
                    ability.description()
                ),
                duration_secs: 5.0,
            });
        }
    }

    // Notify on deactivated abilities
    for ability in &prev_active {
        if !bonuses.active_abilities.contains(ability) {
            toast_events.send(ToastEvent {
                message: format!(
                    "🔒 {} deactivated: {} (friendship dropped below threshold)",
                    ability.npc_id(),
                    ability.display_name()
                ),
                duration_secs: 4.0,
            });
        }
    }
}

/// Apply XP multiplier from crew bonuses to XP gain events.
pub fn apply_xp_bonus(
    bonuses: Res<CrewBonuses>,
    mut xp_events: EventReader<XpGainEvent>,
    mut boosted_xp: EventWriter<XpGainEvent>,
) {
    if bonuses.xp_multiplier <= 1.0 && bonuses.landing_xp_bonus <= 0.0 {
        return;
    }

    // Note: In a real implementation, this would modify the XP before it's applied.
    // Here we just consume events to show the pattern — actual XP application
    // is in the progression system which should read CrewBonuses.
    for _ev in xp_events.read() {
        // Events are consumed; the progression system reads CrewBonuses directly
    }
}

/// Apply fuel efficiency bonus from Marco's ability.
pub fn apply_fuel_bonus(
    bonuses: Res<CrewBonuses>,
    time: Res<Time>,
    mut flight_state: ResMut<FlightState>,
) {
    if bonuses.fuel_multiplier >= 1.0 {
        return;
    }
    if matches!(
        flight_state.phase,
        FlightPhase::Idle | FlightPhase::Arrived | FlightPhase::Preflight
    ) {
        return;
    }

    // Reduce fuel consumption slightly
    let savings = (1.0 - bonuses.fuel_multiplier) * 0.01 * time.delta_secs();
    flight_state.fuel_remaining += savings;
}
