//! Fuel management — fuel types, burn rate calculation, reserve warnings, bingo fuel.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::shared::*;

/// Fuel types and their cost per unit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FuelType {
    AvGas,
    JetA,
}

impl FuelType {
    pub fn cost_per_unit(&self) -> u32 {
        match self {
            FuelType::AvGas => 2,
            FuelType::JetA => 3,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FuelType::AvGas => "100LL AvGas",
            FuelType::JetA => "Jet-A",
        }
    }
}

pub fn fuel_type_for_class(class: AircraftClass) -> FuelType {
    match class {
        AircraftClass::SingleProp | AircraftClass::TwinProp | AircraftClass::Seaplane => FuelType::AvGas,
        _ => FuelType::JetA,
    }
}

// ── Fuel burn calculation constants ───────────────────────────────────

/// Minimum throttle contribution to burn rate (idle).
const MIN_THROTTLE_FACTOR: f32 = 0.2;
/// Additional throttle contribution at full power.
const MAX_THROTTLE_FACTOR: f32 = 0.8;
/// Service ceiling used to normalise altitude efficiency (ft).
const SERVICE_CEILING_FT: f32 = 35_000.0;
/// Maximum fuel-efficiency gain at service ceiling.
const ALTITUDE_EFFICIENCY_FACTOR: f32 = 0.15;
/// Reference cargo weight for penalty normalisation (kg).
const CARGO_WEIGHT_REFERENCE_KG: f32 = 5_000.0;
/// Maximum fuel penalty fraction for a full cargo load.
const CARGO_WEIGHT_PENALTY_FACTOR: f32 = 0.20;

/// Calculate instantaneous fuel burn per second.
pub fn calculate_fuel_burn(base_rate: f32, throttle: f32, altitude_ft: f32, cargo_weight_kg: f32) -> f32 {
    let throttle_factor = MIN_THROTTLE_FACTOR + MAX_THROTTLE_FACTOR * throttle;
    let altitude_factor = 1.0 - (altitude_ft / SERVICE_CEILING_FT).min(1.0) * ALTITUDE_EFFICIENCY_FACTOR;
    let weight_factor = 1.0 + (cargo_weight_kg / CARGO_WEIGHT_REFERENCE_KG).min(1.0) * CARGO_WEIGHT_PENALTY_FACTOR;
    base_rate * throttle_factor * altitude_factor * weight_factor
}

const FUEL_WARN_25: f32 = 0.25;
const FUEL_WARN_15: f32 = 0.15;
const FUEL_WARN_05: f32 = 0.05;

#[derive(Resource, Default, Clone, Debug, Serialize, Deserialize)]
pub struct FuelWarnings {
    pub warned_25: bool,
    pub warned_15: bool,
    pub warned_05: bool,
    pub total_fuel_burned: f32,
}

/// Calculate bingo fuel — minimum fuel to divert to nearest airport.
pub fn bingo_fuel(current_speed_knots: f32, burn_rate: f32, distance_nm: f32) -> f32 {
    if current_speed_knots < 1.0 { return f32::MAX; }
    let time_hours = distance_nm / current_speed_knots;
    let time_secs = time_hours * 3600.0;
    burn_rate * time_secs / 60.0 * 1.1
}

pub fn nearest_divert_airport(origin: AirportId, destination: AirportId) -> (AirportId, f32) {
    let all_airports = [
        AirportId::HomeBase, AirportId::Windport, AirportId::Frostpeak,
        AirportId::Sunhaven, AirportId::Ironforge, AirportId::Cloudmere,
        AirportId::Duskhollow, AirportId::Stormwatch, AirportId::Grandcity,
        AirportId::Skyreach,
    ];
    let mut best = destination;
    let mut best_dist = f32::MAX;
    for &ap in &all_airports {
        if ap == destination { continue; }
        let d = airport_distance(origin, ap);
        if d < best_dist { best = ap; best_dist = d; }
    }
    (best, best_dist)
}

pub fn handle_refuel(
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mut fuel_warnings: ResMut<FuelWarnings>,
) {
    if let Some(ac) = fleet.active() {
        if let Some(def) = aircraft_registry.get(&ac.aircraft_id) {
            if (ac.fuel - def.fuel_capacity).abs() < 0.1 {
                fuel_warnings.warned_25 = false;
                fuel_warnings.warned_15 = false;
                fuel_warnings.warned_05 = false;
            }
        }
    }
}

/// Refuel aircraft at fuel pump (triggered by interaction).
pub fn refuel_aircraft(
    fleet: &mut Fleet,
    gold: &mut Gold,
    registry: &AircraftRegistry,
) -> Result<u32, &'static str> {
    let aircraft = fleet.active_mut().ok_or("No active aircraft")?;
    let def = registry.get(&aircraft.aircraft_id).ok_or("Unknown aircraft")?;
    let fuel_needed = def.fuel_capacity - aircraft.fuel;
    if fuel_needed <= 0.0 { return Err("Tank is full"); }

    let fuel_type = fuel_type_for_class(def.class);
    let cost_per_unit = fuel_type.cost_per_unit();
    let cost = (fuel_needed * cost_per_unit as f32) as u32;
    if gold.amount < cost { return Err("Not enough gold"); }

    gold.amount -= cost;
    aircraft.fuel = def.fuel_capacity;
    Ok(cost)
}

/// In-flight fuel monitoring system.
pub fn monitor_fuel_in_flight(
    flight_state: Res<FlightState>,
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mut fuel_warnings: ResMut<FuelWarnings>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent | FlightPhase::Approach
    ) { return; }

    let capacity = fleet.active()
        .and_then(|ac| aircraft_registry.get(&ac.aircraft_id))
        .map_or(MAX_FUEL, |def| def.fuel_capacity);
    let ratio = flight_state.fuel_remaining / capacity;

    if ratio <= FUEL_WARN_05 && !fuel_warnings.warned_05 {
        fuel_warnings.warned_05 = true;
        toast_events.send(ToastEvent {
            message: "🚨 CRITICAL: Fuel below 5%! Land immediately!".to_string(),
            duration_secs: 6.0,
        });
    } else if ratio <= FUEL_WARN_15 && !fuel_warnings.warned_15 {
        fuel_warnings.warned_15 = true;
        let (divert, _dist) = nearest_divert_airport(flight_state.origin, flight_state.destination);
        toast_events.send(ToastEvent {
            message: format!("⚠ Fuel below 15%! Consider diverting to {}", divert.display_name()),
            duration_secs: 5.0,
        });
    } else if ratio <= FUEL_WARN_25 && !fuel_warnings.warned_25 {
        fuel_warnings.warned_25 = true;
        toast_events.send(ToastEvent {
            message: "⛽ Fuel below 25%. Monitor closely.".to_string(),
            duration_secs: 3.0,
        });
    }
}
