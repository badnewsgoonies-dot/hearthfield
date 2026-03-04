//! Insurance system — policies, premiums, claims for aircraft operations.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoverageType {
    Basic,    // Liability only
    Standard, // Hull + liability
    Premium,  // All-risk coverage
}

impl CoverageType {
    pub fn display_name(&self) -> &'static str {
        match self {
            CoverageType::Basic => "Basic (Liability Only)",
            CoverageType::Standard => "Standard (Hull + Liability)",
            CoverageType::Premium => "Premium (All-Risk)",
        }
    }

    pub fn base_premium(&self) -> u32 {
        match self {
            CoverageType::Basic => 50,
            CoverageType::Standard => 150,
            CoverageType::Premium => 300,
        }
    }

    pub fn deductible(&self) -> u32 {
        match self {
            CoverageType::Basic => 500,
            CoverageType::Standard => 250,
            CoverageType::Premium => 100,
        }
    }

    pub fn coverage_limit(&self) -> u32 {
        match self {
            CoverageType::Basic => 5_000,
            CoverageType::Standard => 20_000,
            CoverageType::Premium => 50_000,
        }
    }

    pub fn covers_hull(&self) -> bool {
        !matches!(self, CoverageType::Basic)
    }

    pub fn covers_weather_damage(&self) -> bool {
        matches!(self, CoverageType::Premium)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InsurancePolicy {
    pub coverage_type: CoverageType,
    pub premium: u32,
    pub deductible: u32,
    pub coverage_limit: u32,
    pub aircraft_id: String,
    pub active: bool,
}

impl InsurancePolicy {
    pub fn new(coverage_type: CoverageType, aircraft_id: &str, rank: PilotRank) -> Self {
        let rank_discount = match rank {
            PilotRank::Ace => 0.7,
            PilotRank::Captain => 0.8,
            PilotRank::Senior => 0.85,
            PilotRank::Commercial => 0.9,
            PilotRank::Private => 0.95,
            PilotRank::Student => 1.0,
        };
        let premium = (coverage_type.base_premium() as f32 * rank_discount) as u32;

        Self {
            coverage_type,
            premium,
            deductible: coverage_type.deductible(),
            coverage_limit: coverage_type.coverage_limit(),
            aircraft_id: aircraft_id.to_string(),
            active: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InsuranceClaim {
    pub day_filed: u32,
    pub damage_amount: u32,
    pub payout: u32,
    pub reason: String,
    pub resolved: bool,
}

/// Insurance state for the player's fleet.
#[derive(Resource, Clone, Debug, Default, Serialize, Deserialize)]
pub struct InsuranceState {
    pub policies: Vec<InsurancePolicy>,
    pub claims: Vec<InsuranceClaim>,
    pub total_claims_filed: u32,
    pub clean_record_days: u32,
    pub premium_multiplier: f32,
    pub last_payment_day: u32,
}

impl InsuranceState {
    pub fn policy_for(&self, aircraft_id: &str) -> Option<&InsurancePolicy> {
        self.policies
            .iter()
            .find(|p| p.aircraft_id == aircraft_id && p.active)
    }

    pub fn has_insurance(&self, aircraft_id: &str) -> bool {
        self.policy_for(aircraft_id).is_some()
    }

    pub fn total_monthly_premium(&self) -> u32 {
        self.policies
            .iter()
            .filter(|p| p.active)
            .map(|p| (p.premium as f32 * self.premium_multiplier) as u32)
            .sum()
    }

    pub fn can_fly_commercial(&self, aircraft_id: &str) -> bool {
        self.has_insurance(aircraft_id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Purchase insurance for an aircraft.
pub fn purchase_insurance(
    mut purchase_events: EventReader<PurchaseEvent>,
    pilot_state: Res<PilotState>,
    mut insurance: ResMut<InsuranceState>,
    fleet: Res<Fleet>,
    _gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in purchase_events.read() {
        // Insurance items are identified by prefix
        if !ev.item_id.starts_with("insurance_") {
            continue;
        }

        let coverage = match ev.item_id.as_str() {
            "insurance_basic" => CoverageType::Basic,
            "insurance_standard" => CoverageType::Standard,
            "insurance_premium" => CoverageType::Premium,
            _ => continue,
        };

        let Some(aircraft) = fleet.active() else {
            toast_events.send(ToastEvent {
                message: "No active aircraft to insure.".to_string(),
                duration_secs: 3.0,
            });
            continue;
        };

        // Remove existing policy for this aircraft
        insurance
            .policies
            .retain(|p| p.aircraft_id != aircraft.aircraft_id);

        let policy = InsurancePolicy::new(coverage, &aircraft.aircraft_id, pilot_state.rank);
        let premium = policy.premium;

        toast_events.send(ToastEvent {
            message: format!(
                "✅ {} insurance purchased for {}. Monthly premium: {}g",
                coverage.display_name(),
                aircraft.nickname,
                premium
            ),
            duration_secs: 4.0,
        });

        insurance.policies.push(policy);
    }
}

/// Monthly premium payments on day end.
pub fn process_premium_payments(
    mut day_end_events: EventReader<DayEndEvent>,
    calendar: Res<Calendar>,
    mut insurance: ResMut<InsuranceState>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        let day = calendar.total_days();

        // Pay monthly (every 28 days)
        if day.saturating_sub(insurance.last_payment_day) < 28 {
            continue;
        }
        insurance.last_payment_day = day;

        let total = insurance.total_monthly_premium();
        if total == 0 {
            continue;
        }

        gold_events.send(GoldChangeEvent {
            amount: -(total as i32),
            reason: "Insurance premiums".to_string(),
        });
        toast_events.send(ToastEvent {
            message: format!("Insurance premium due: -{}g", total),
            duration_secs: 3.0,
        });

        // Clean record tracking
        insurance.clean_record_days += 28;
        if insurance.clean_record_days >= 112 && insurance.premium_multiplier > 0.8 {
            insurance.premium_multiplier = (insurance.premium_multiplier - 0.05).max(0.8);
            toast_events.send(ToastEvent {
                message: "🎉 Clean record discount! Insurance premiums reduced.".to_string(),
                duration_secs: 3.0,
            });
        }
    }
}

/// File insurance claim after emergency/damage.
pub fn file_insurance_claim(
    mut emergency_events: EventReader<EmergencyEvent>,
    calendar: Res<Calendar>,
    fleet: Res<Fleet>,
    mut insurance: ResMut<InsuranceState>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in emergency_events.read() {
        let Some(aircraft) = fleet.active() else {
            continue;
        };
        let Some(policy) = insurance.policy_for(&aircraft.aircraft_id).cloned() else {
            toast_events.send(ToastEvent {
                message: "⚠ No insurance! Damage costs are out of pocket.".to_string(),
                duration_secs: 4.0,
            });
            continue;
        };

        let damage_amount = match ev.kind {
            EmergencyKind::EngineFailure => 2000,
            EmergencyKind::FuelLeak => 800,
            EmergencyKind::HydraulicFailure => 1500,
            EmergencyKind::BirdStrike => 1000,
            EmergencyKind::LightningStrike => 1200,
        };

        let payout = if damage_amount > policy.deductible {
            (damage_amount - policy.deductible).min(policy.coverage_limit)
        } else {
            0
        };

        let claim = InsuranceClaim {
            day_filed: calendar.total_days(),
            damage_amount,
            payout,
            reason: format!("{:?}", ev.kind),
            resolved: true,
        };

        insurance.claims.push(claim);
        insurance.total_claims_filed += 1;
        insurance.clean_record_days = 0;

        // Premium increase after claim
        insurance.premium_multiplier = (insurance.premium_multiplier + 0.15).min(2.0);

        if payout > 0 {
            gold_events.send(GoldChangeEvent {
                amount: payout as i32,
                reason: "Insurance claim payout".to_string(),
            });
        }

        toast_events.send(ToastEvent {
            message: format!(
                "Insurance claim filed: {}g damage, {}g payout ({}g deductible). Premiums increased.",
                damage_amount, payout, policy.deductible
            ),
            duration_secs: 5.0,
        });
    }
}

/// Check if player can fly commercial without insurance.
pub fn enforce_insurance_requirement(
    flight_state: Res<FlightState>,
    insurance: Res<InsuranceState>,
    fleet: Res<Fleet>,
    mission_board: Res<MissionBoard>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase != FlightPhase::Preflight {
        return;
    }

    // Only require insurance for passenger missions
    let is_passenger = mission_board
        .active
        .as_ref()
        .map(|m| m.mission.passenger_count > 0)
        .unwrap_or(false);

    if !is_passenger {
        return;
    }

    if let Some(aircraft) = fleet.active() {
        if !insurance.has_insurance(&aircraft.aircraft_id) {
            toast_events.send(ToastEvent {
                message: "⚠ Insurance required for passenger operations! Visit the shop."
                    .to_string(),
                duration_secs: 4.0,
            });
        }
    }
}
