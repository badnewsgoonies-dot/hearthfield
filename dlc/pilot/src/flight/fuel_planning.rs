//! Pre-flight fuel planning — trip fuel, reserves, weight-and-balance.
//!
//! Calculates legal fuel requirements and checks aircraft weight limits before
//! each flight. A FuelPlan is generated during the preflight briefing.

use crate::shared::*;
use bevy::prelude::*;

// ─── Fuel Plan ───────────────────────────────────────────────────────────

/// Result of a fuel planning calculation.
#[derive(Clone, Debug, Default)]
pub struct FuelPlan {
    /// Fuel required for the planned route at cruise burn rate.
    pub trip_fuel: f32,
    /// Legal minimum reserve (45 min VFR day, 30 min IFR).
    pub reserve_fuel: f32,
    /// Fuel to reach an alternate airport if a diversion is needed.
    pub alternate_fuel: f32,
    /// 5 % contingency on trip fuel.
    pub contingency_fuel: f32,
    /// Sum of all fuel components.
    pub total_required: f32,
    /// Extra fuel the pilot chooses to carry (tankering).
    pub tankering_fuel: f32,
    /// Final fuel to uplift.
    pub total_uplift: f32,
}

/// Flight rules for reserve calculation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FlightRules {
    VfrDay,
    VfrNight,
    Ifr,
}

impl FlightRules {
    /// Reserve time in hours.
    pub fn reserve_minutes(&self) -> f32 {
        match self {
            Self::VfrDay => 45.0,
            Self::VfrNight => 45.0,
            Self::Ifr => 30.0,
        }
    }
}

// ─── Fuel Calculation ────────────────────────────────────────────────────

/// Calculate a full fuel plan for a given route.
pub fn calculate_fuel_plan(
    origin: AirportId,
    destination: AirportId,
    aircraft: &AircraftDef,
    weather: &WeatherState,
    rules: FlightRules,
    tankering_extra: f32,
) -> FuelPlan {
    let distance = airport_distance(origin, destination);

    // Headwind/tailwind factor from weather
    let wind_factor = 1.0 + weather.wind_speed_knots * 0.002;
    let effective_speed = (aircraft.speed_knots / wind_factor).max(60.0);

    // Trip time in hours
    let trip_time_hrs = distance / effective_speed;
    let trip_fuel = trip_time_hrs * aircraft.fuel_burn_rate;

    // Reserve
    let reserve_time_hrs = rules.reserve_minutes() / 60.0;
    let reserve_fuel = reserve_time_hrs * aircraft.fuel_burn_rate;

    // Alternate — assume closest alternate is ~80 NM away
    let alternate_distance = 80.0_f32;
    let alternate_time = alternate_distance / effective_speed;
    let alternate_fuel = alternate_time * aircraft.fuel_burn_rate;

    // Contingency: 5 % of trip fuel
    let contingency_fuel = trip_fuel * 0.05;

    let total_required = trip_fuel + reserve_fuel + alternate_fuel + contingency_fuel;
    let tankering_fuel = tankering_extra.max(0.0);
    let total_uplift = total_required + tankering_fuel;

    FuelPlan {
        trip_fuel,
        reserve_fuel,
        alternate_fuel,
        contingency_fuel,
        total_required,
        tankering_fuel,
        total_uplift,
    }
}

// ─── Weight & Balance ────────────────────────────────────────────────────

/// Simplified weight-and-balance check.
#[derive(Clone, Debug)]
pub struct WeightBalance {
    pub empty_weight: f32,
    pub fuel_weight: f32,
    pub passenger_weight: f32,
    pub cargo_weight: f32,
    pub total_weight: f32,
    pub max_takeoff_weight: f32,
    pub within_limits: bool,
    pub overweight_by: f32,
}

/// Standard weight per passenger (including carry-on) in kg.
const PASSENGER_WEIGHT_KG: f32 = 85.0;
/// Fuel density — roughly 0.72 kg per unit (avgas-equivalent).
const FUEL_WEIGHT_PER_UNIT: f32 = 2.7;

pub fn calculate_weight_balance(
    aircraft: &AircraftDef,
    fuel_amount: f32,
    passenger_count: u32,
    cargo_kg: f32,
) -> WeightBalance {
    // Aircraft base empty weight derived from durability + class heuristic
    let empty_weight = match aircraft.class {
        AircraftClass::SingleProp => 750.0,
        AircraftClass::TwinProp => 1200.0,
        AircraftClass::Turboprop => 3800.0,
        AircraftClass::LightJet => 4500.0,
        AircraftClass::MediumJet => 12000.0,
        AircraftClass::HeavyJet => 30000.0,
        AircraftClass::Cargo => 18000.0,
        AircraftClass::Seaplane => 1100.0,
    };

    let max_takeoff_weight = match aircraft.class {
        AircraftClass::SingleProp => 1200.0,
        AircraftClass::TwinProp => 2100.0,
        AircraftClass::Turboprop => 6800.0,
        AircraftClass::LightJet => 8000.0,
        AircraftClass::MediumJet => 25000.0,
        AircraftClass::HeavyJet => 70000.0,
        AircraftClass::Cargo => 35000.0,
        AircraftClass::Seaplane => 1800.0,
    };

    let fuel_weight = fuel_amount * FUEL_WEIGHT_PER_UNIT;
    let passenger_weight = passenger_count as f32 * PASSENGER_WEIGHT_KG;
    let total_weight = empty_weight + fuel_weight + passenger_weight + cargo_kg;
    let overweight_by = (total_weight - max_takeoff_weight).max(0.0);

    WeightBalance {
        empty_weight,
        fuel_weight,
        passenger_weight,
        cargo_weight: cargo_kg,
        total_weight,
        max_takeoff_weight,
        within_limits: total_weight <= max_takeoff_weight,
        overweight_by,
    }
}

// ─── Tankering decision helper ───────────────────────────────────────────

/// Returns recommended tankering amount (extra fuel from a cheap airport).
/// Positive if the destination fuel price is significantly higher.
pub fn recommended_tankering(
    origin_fuel_price: f32,
    destination_fuel_price: f32,
    aircraft: &AircraftDef,
    plan: &FuelPlan,
) -> f32 {
    if destination_fuel_price <= origin_fuel_price * 1.15 {
        return 0.0; // not worth it
    }
    // Carry enough extra for the return trip, up to capacity minus required
    let capacity_remaining = aircraft.fuel_capacity - plan.total_required;
    capacity_remaining
        .max(0.0)
        .min(aircraft.fuel_capacity * 0.3)
}

// ─── Fuel Plan Display Helper ────────────────────────────────────────────

impl FuelPlan {
    pub fn summary_lines(&self) -> Vec<String> {
        vec![
            format!("Trip fuel:        {:.1}", self.trip_fuel),
            format!("Reserve:          {:.1}", self.reserve_fuel),
            format!("Alternate:        {:.1}", self.alternate_fuel),
            format!("Contingency (5%): {:.1}", self.contingency_fuel),
            format!("─────────────────────"),
            format!("Required:         {:.1}", self.total_required),
            format!("Tankering:        {:.1}", self.tankering_fuel),
            format!("Total uplift:     {:.1}", self.total_uplift),
        ]
    }

    /// Returns true if the aircraft can carry enough fuel.
    pub fn fits_in_tanks(&self, fuel_capacity: f32) -> bool {
        self.total_uplift <= fuel_capacity
    }
}

// ─── System: display fuel plan during preflight ──────────────────────────

pub fn display_fuel_plan_on_preflight(
    flight_state: Res<FlightState>,
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    weather: Res<WeatherState>,
    mut toast: EventWriter<ToastEvent>,
) {
    if flight_state.phase != FlightPhase::Preflight {
        return;
    }

    let Some(owned) = fleet.active() else { return };
    let Some(def) = aircraft_registry.get(&owned.aircraft_id) else {
        return;
    };

    let plan = calculate_fuel_plan(
        flight_state.origin,
        flight_state.destination,
        def,
        &weather,
        if weather.visibility_nm < 3.0 {
            FlightRules::Ifr
        } else {
            FlightRules::VfrDay
        },
        0.0,
    );

    if !plan.fits_in_tanks(def.fuel_capacity) {
        toast.send(ToastEvent {
            message: format!(
                "⚠ Fuel plan {:.0} exceeds capacity {:.0}!",
                plan.total_uplift, def.fuel_capacity
            ),
            duration_secs: 5.0,
        });
    }
}
