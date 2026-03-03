//! Cargo management — types, manifests, loading, condition tracking, weight effects.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CargoType {
    General,
    Fragile,
    Perishable,
    Hazardous,
    Livestock,
    Mail,
    Medical,
}

impl CargoType {
    pub fn display_name(&self) -> &'static str {
        match self {
            CargoType::General => "General Cargo",
            CargoType::Fragile => "Fragile",
            CargoType::Perishable => "Perishable Goods",
            CargoType::Hazardous => "Hazardous Materials",
            CargoType::Livestock => "Livestock",
            CargoType::Mail => "Mail & Parcels",
            CargoType::Medical => "Medical Supplies",
        }
    }

    pub fn handling_difficulty(&self) -> f32 {
        match self {
            CargoType::General => 0.0,
            CargoType::Fragile => 0.6,
            CargoType::Perishable => 0.4,
            CargoType::Hazardous => 0.8,
            CargoType::Livestock => 0.5,
            CargoType::Mail => 0.1,
            CargoType::Medical => 0.7,
        }
    }

    /// How quickly quality degrades per minute of flight.
    pub fn decay_rate(&self) -> f32 {
        match self {
            CargoType::General => 0.0,
            CargoType::Fragile => 0.1,
            CargoType::Perishable => 0.3,
            CargoType::Hazardous => 0.05,
            CargoType::Livestock => 0.2,
            CargoType::Mail => 0.0,
            CargoType::Medical => 0.15,
        }
    }

    pub fn bonus_pay_multiplier(&self) -> f32 {
        match self {
            CargoType::General => 1.0,
            CargoType::Fragile => 1.5,
            CargoType::Perishable => 1.3,
            CargoType::Hazardous => 2.0,
            CargoType::Livestock => 1.4,
            CargoType::Mail => 1.1,
            CargoType::Medical => 1.8,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CargoItem {
    pub name: String,
    pub cargo_type: CargoType,
    pub weight_kg: f32,
    pub quality: f32, // 0.0–100.0; starts at 100
}

/// Manifest of all cargo on the current flight.
#[derive(Resource, Clone, Debug, Default)]
pub struct CargoManifest {
    pub items: Vec<CargoItem>,
    pub loading_progress: f32, // 0.0–1.0 during loading phase
    pub loaded: bool,
}

impl CargoManifest {
    pub fn total_weight_kg(&self) -> f32 {
        self.items.iter().map(|c| c.weight_kg).sum()
    }

    pub fn average_quality(&self) -> f32 {
        if self.items.is_empty() {
            return 100.0;
        }
        let sum: f32 = self.items.iter().map(|c| c.quality).sum();
        sum / self.items.len() as f32
    }

    pub fn delivery_rating(&self) -> &'static str {
        match self.average_quality() as u32 {
            90..=100 => "Excellent",
            70..=89 => "Good",
            50..=69 => "Fair",
            25..=49 => "Poor",
            _ => "Damaged",
        }
    }

    pub fn weight_performance_factor(&self, max_cargo_kg: f32) -> f32 {
        if max_cargo_kg <= 0.0 {
            return 1.0;
        }
        let ratio = self.total_weight_kg() / max_cargo_kg;
        // Heavier load = more fuel burn, slower climb, reduced speed
        1.0 + ratio * 0.3
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Load cargo during preflight — loading time based on weight and cargo type.
pub fn load_cargo(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    mut manifest: ResMut<CargoManifest>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase != FlightPhase::Preflight || manifest.loaded {
        return;
    }

    if manifest.items.is_empty() {
        manifest.loaded = true;
        return;
    }

    let weight = manifest.total_weight_kg();
    let load_speed = 200.0; // kg per second at well-equipped airport
    manifest.loading_progress += (load_speed / weight.max(1.0)) * time.delta_secs();

    if manifest.loading_progress >= 1.0 {
        manifest.loading_progress = 1.0;
        manifest.loaded = true;
        toast_events.send(ToastEvent {
            message: format!("Cargo loaded: {:.0} kg", weight),
            duration_secs: 2.0,
        });
    }
}

/// Cargo quality degrades during flight — turbulence and time affect fragile goods.
pub fn cargo_condition(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    weather_state: Res<WeatherState>,
    mut manifest: ResMut<CargoManifest>,
) {
    if flight_state.phase == FlightPhase::Idle || flight_state.phase == FlightPhase::Arrived {
        return;
    }
    if manifest.items.is_empty() {
        return;
    }

    let turbulence_factor = match weather_state.turbulence_level {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 0.5,
        TurbulenceLevel::Moderate => 1.5,
        TurbulenceLevel::Severe => 3.0,
    };

    let dt = time.delta_secs() / 60.0; // convert to minutes

    for item in &mut manifest.items {
        let base_decay = item.cargo_type.decay_rate();
        let turb_decay = base_decay * turbulence_factor;
        let total_decay = (base_decay + turb_decay) * dt;
        item.quality = (item.quality - total_decay).max(0.0);
    }
}

/// On flight completion, rate cargo delivery and adjust pay/reputation.
pub fn rate_cargo_delivery(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut manifest: ResMut<CargoManifest>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in flight_complete_events.read() {
        if manifest.items.is_empty() {
            continue;
        }

        let rating = manifest.delivery_rating();
        let quality_bonus = match rating {
            "Excellent" => 1.5,
            "Good" => 1.2,
            "Fair" => 1.0,
            "Poor" => 0.7,
            _ => 0.4,
        };

        let type_bonus: f32 = manifest
            .items
            .iter()
            .map(|c| c.cargo_type.bonus_pay_multiplier())
            .sum::<f32>()
            / manifest.items.len().max(1) as f32;

        let bonus_gold = (50.0 * quality_bonus * type_bonus) as i32;

        gold_events.send(GoldChangeEvent {
            amount: bonus_gold,
            reason: format!("Cargo delivery ({})", rating),
        });

        toast_events.send(ToastEvent {
            message: format!("Cargo delivery: {} — +{}g bonus", rating, bonus_gold),
            duration_secs: 3.0,
        });

        // Reset manifest for next flight
        manifest.items.clear();
        manifest.loading_progress = 0.0;
        manifest.loaded = false;
    }
}

/// Generate cargo for a mission (called when mission is accepted).
pub fn generate_cargo_for_mission(
    mut mission_accepted_events: EventReader<MissionAcceptedEvent>,
    mission_board: Res<MissionBoard>,
    mut manifest: ResMut<CargoManifest>,
) {
    for ev in mission_accepted_events.read() {
        if let Some(active) = &mission_board.active {
            if active.mission.cargo_kg > 0.0 {
                let mut rng = rand::thread_rng();
                let cargo_type = match active.mission.mission_type {
                    MissionType::Cargo => CargoType::General,
                    MissionType::Medical => CargoType::Medical,
                    MissionType::Delivery => CargoType::Mail,
                    _ => CargoType::General,
                };

                let num_items = rng.gen_range(1..=4);
                let weight_per = active.mission.cargo_kg / num_items as f32;

                manifest.items.clear();
                for i in 0..num_items {
                    manifest.items.push(CargoItem {
                        name: format!("{} #{}", cargo_type.display_name(), i + 1),
                        cargo_type,
                        weight_kg: weight_per,
                        quality: 100.0,
                    });
                }
                manifest.loading_progress = 0.0;
                manifest.loaded = false;
            }
        }
    }
}
