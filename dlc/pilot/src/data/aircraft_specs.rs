//! Detailed aircraft specifications — performance tables, V-speeds, history blurbs.
//!
//! Complements the base `AircraftDef` in `shared` with deeper technical data
//! that feeds into flight mechanics and the in-game aircraft encyclopedia.

use crate::shared::*;
use std::collections::HashMap;

// ─── V-Speeds ────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct VSpeeds {
    /// Stall speed (clean config), knots
    pub vs: f32,
    /// Takeoff decision speed, knots
    pub v1: f32,
    /// Rotation speed, knots
    pub vr: f32,
    /// Takeoff safety speed, knots
    pub v2: f32,
    /// Never exceed speed, knots
    pub vne: f32,
}

// ─── Performance Table Entry ─────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PerfEntry {
    pub altitude_ft: u32,
    pub speed_knots: f32,
    pub fuel_burn_rate: f32,
}

// ─── Operating Limitations ───────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct OperatingLimits {
    /// Maximum demonstrated crosswind component, knots
    pub max_crosswind_knots: f32,
    /// Minimum runway length for takeoff, feet
    pub min_runway_ft: u32,
    /// Minimum operating temperature, °C
    pub cold_weather_limit_c: i32,
    /// Requires de-icing below this temperature, °C
    pub de_ice_required_below_c: i32,
}

// ─── Full Spec Sheet ─────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AircraftSpec {
    pub aircraft_id: String,
    pub wingspan_ft: f32,
    pub length_ft: f32,
    pub height_ft: f32,
    pub empty_weight_lbs: f32,
    pub max_takeoff_weight_lbs: f32,
    pub fuel_capacity_gal: f32,
    pub fuel_burn_gph: f32,
    pub v_speeds: VSpeeds,
    pub ceiling_ft: u32,
    pub rate_of_climb_fpm: u32,
    pub stall_speed_knots: f32,
    pub approach_speed_knots: f32,
    pub performance_table: Vec<PerfEntry>,
    pub limits: OperatingLimits,
    pub cockpit_description: &'static str,
    pub history_blurb: &'static str,
}

// ─── Spec Registry ───────────────────────────────────────────────────────

pub fn build_specs_registry() -> HashMap<String, AircraftSpec> {
    let mut map = HashMap::new();

    // 1. Cessna 172 Skyhawk
    map.insert("cessna_172".into(), AircraftSpec {
        aircraft_id: "cessna_172".into(),
        wingspan_ft: 36.1,
        length_ft: 27.2,
        height_ft: 8.9,
        empty_weight_lbs: 1691.0,
        max_takeoff_weight_lbs: 2550.0,
        fuel_capacity_gal: 56.0,
        fuel_burn_gph: 8.5,
        v_speeds: VSpeeds { vs: 48.0, v1: 55.0, vr: 55.0, v2: 62.0, vne: 163.0 },
        ceiling_ft: 14000,
        rate_of_climb_fpm: 730,
        stall_speed_knots: 48.0,
        approach_speed_knots: 65.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 122.0, fuel_burn_rate: 8.5 },
            PerfEntry { altitude_ft: 5000, speed_knots: 118.0, fuel_burn_rate: 8.0 },
            PerfEntry { altitude_ft: 10000, speed_knots: 110.0, fuel_burn_rate: 7.2 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 15.0, min_runway_ft: 1500, cold_weather_limit_c: -30, de_ice_required_below_c: 2 },
        cockpit_description: "Simple analog panel with six-pack instruments and a single GPS unit.",
        history_blurb: "The Cessna 172 is the most-produced aircraft in history, a beloved trainer and utility aircraft known for forgiving handling and reliability.",
    });

    // 2. Piper Seneca
    map.insert("piper_seneca".into(), AircraftSpec {
        aircraft_id: "piper_seneca".into(),
        wingspan_ft: 38.9,
        length_ft: 28.6,
        height_ft: 9.9,
        empty_weight_lbs: 3382.0,
        max_takeoff_weight_lbs: 4750.0,
        fuel_capacity_gal: 123.0,
        fuel_burn_gph: 22.0,
        v_speeds: VSpeeds { vs: 62.0, v1: 80.0, vr: 80.0, v2: 88.0, vne: 204.0 },
        ceiling_ft: 25000,
        rate_of_climb_fpm: 1300,
        stall_speed_knots: 62.0,
        approach_speed_knots: 85.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 186.0, fuel_burn_rate: 22.0 },
            PerfEntry { altitude_ft: 10000, speed_knots: 180.0, fuel_burn_rate: 19.0 },
            PerfEntry { altitude_ft: 20000, speed_knots: 168.0, fuel_burn_rate: 16.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 17.0, min_runway_ft: 2200, cold_weather_limit_c: -35, de_ice_required_below_c: 0 },
        cockpit_description: "Twin-engine panel with full IFR avionics and dual NAV/COM radios.",
        history_blurb: "The Piper Seneca bridges single and multi-engine flying, offering excellent multi-engine training while remaining practical for personal transport.",
    });

    // 3. Beech King Air 350
    map.insert("king_air".into(), AircraftSpec {
        aircraft_id: "king_air".into(),
        wingspan_ft: 57.9,
        length_ft: 46.7,
        height_ft: 14.3,
        empty_weight_lbs: 9100.0,
        max_takeoff_weight_lbs: 15000.0,
        fuel_capacity_gal: 544.0,
        fuel_burn_gph: 90.0,
        v_speeds: VSpeeds { vs: 93.0, v1: 106.0, vr: 106.0, v2: 115.0, vne: 263.0 },
        ceiling_ft: 35000,
        rate_of_climb_fpm: 2730,
        stall_speed_knots: 93.0,
        approach_speed_knots: 110.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 280.0, fuel_burn_rate: 90.0 },
            PerfEntry { altitude_ft: 15000, speed_knots: 290.0, fuel_burn_rate: 78.0 },
            PerfEntry { altitude_ft: 30000, speed_knots: 270.0, fuel_burn_rate: 65.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 25.0, min_runway_ft: 3300, cold_weather_limit_c: -40, de_ice_required_below_c: 5 },
        cockpit_description: "Glass cockpit with Collins Pro Line avionics, dual FMS, and weather radar.",
        history_blurb: "The King Air 350 is the workhorse of corporate aviation and regional airlines, combining turboprop efficiency with pressurised comfort for up to 11 passengers.",
    });

    // 4. Citation CJ4 (light jet)
    map.insert("citation_cj4".into(), AircraftSpec {
        aircraft_id: "citation_cj4".into(),
        wingspan_ft: 50.8,
        length_ft: 53.3,
        height_ft: 15.3,
        empty_weight_lbs: 10280.0,
        max_takeoff_weight_lbs: 17110.0,
        fuel_capacity_gal: 727.0,
        fuel_burn_gph: 160.0,
        v_speeds: VSpeeds { vs: 96.0, v1: 108.0, vr: 108.0, v2: 117.0, vne: 305.0 },
        ceiling_ft: 45000,
        rate_of_climb_fpm: 3854,
        stall_speed_knots: 96.0,
        approach_speed_knots: 118.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 340.0, fuel_burn_rate: 160.0 },
            PerfEntry { altitude_ft: 20000, speed_knots: 360.0, fuel_burn_rate: 140.0 },
            PerfEntry { altitude_ft: 40000, speed_knots: 380.0, fuel_burn_rate: 120.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 30.0, min_runway_ft: 3410, cold_weather_limit_c: -54, de_ice_required_below_c: 5 },
        cockpit_description: "Collins Pro Line 21 flight deck with single-pilot certification.",
        history_blurb: "The Citation CJ4 is Cessna's flagship light jet, popular for owner-flown operations with transcontinental range and turbofan efficiency.",
    });

    // 5. Challenger 350 (medium jet)
    map.insert("challenger_350".into(), AircraftSpec {
        aircraft_id: "challenger_350".into(),
        wingspan_ft: 69.0,
        length_ft: 68.7,
        height_ft: 20.4,
        empty_weight_lbs: 24450.0,
        max_takeoff_weight_lbs: 40600.0,
        fuel_capacity_gal: 1810.0,
        fuel_burn_gph: 250.0,
        v_speeds: VSpeeds { vs: 108.0, v1: 130.0, vr: 133.0, v2: 140.0, vne: 350.0 },
        ceiling_ft: 45000,
        rate_of_climb_fpm: 4030,
        stall_speed_knots: 108.0,
        approach_speed_knots: 130.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 400.0, fuel_burn_rate: 250.0 },
            PerfEntry { altitude_ft: 20000, speed_knots: 430.0, fuel_burn_rate: 220.0 },
            PerfEntry { altitude_ft: 41000, speed_knots: 460.0, fuel_burn_rate: 200.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 32.0, min_runway_ft: 4835, cold_weather_limit_c: -54, de_ice_required_below_c: 5 },
        cockpit_description: "Bombardier Vision flight deck with HUD, EVS, and autothrottle.",
        history_blurb: "The Challenger 350 is a super-midsize jet renowned for its wide cabin, smooth ride, and coast-to-coast range — a favorite among charter operators.",
    });

    // 6. Global 7500 (heavy jet)
    map.insert("global_7500".into(), AircraftSpec {
        aircraft_id: "global_7500".into(),
        wingspan_ft: 104.0,
        length_ft: 111.0,
        height_ft: 25.5,
        empty_weight_lbs: 55900.0,
        max_takeoff_weight_lbs: 104800.0,
        fuel_capacity_gal: 5775.0,
        fuel_burn_gph: 400.0,
        v_speeds: VSpeeds { vs: 115.0, v1: 145.0, vr: 148.0, v2: 155.0, vne: 370.0 },
        ceiling_ft: 51000,
        rate_of_climb_fpm: 3900,
        stall_speed_knots: 115.0,
        approach_speed_knots: 140.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 460.0, fuel_burn_rate: 400.0 },
            PerfEntry { altitude_ft: 25000, speed_knots: 490.0, fuel_burn_rate: 360.0 },
            PerfEntry { altitude_ft: 47000, speed_knots: 510.0, fuel_burn_rate: 340.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 35.0, min_runway_ft: 5800, cold_weather_limit_c: -54, de_ice_required_below_c: 5 },
        cockpit_description: "Bombardier Vision with combined synthetic vision, HUD, and four large displays.",
        history_blurb: "The Global 7500 is the world's largest purpose-built business jet, capable of flying non-stop from New York to Hong Kong with a four-zone cabin.",
    });

    // 7. DHC-6 Twin Otter (seaplane)
    map.insert("twin_otter".into(), AircraftSpec {
        aircraft_id: "twin_otter".into(),
        wingspan_ft: 65.0,
        length_ft: 51.8,
        height_ft: 19.5,
        empty_weight_lbs: 7400.0,
        max_takeoff_weight_lbs: 12500.0,
        fuel_capacity_gal: 378.0,
        fuel_burn_gph: 65.0,
        v_speeds: VSpeeds { vs: 58.0, v1: 70.0, vr: 70.0, v2: 78.0, vne: 170.0 },
        ceiling_ft: 25000,
        rate_of_climb_fpm: 1600,
        stall_speed_knots: 58.0,
        approach_speed_knots: 75.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 160.0, fuel_burn_rate: 65.0 },
            PerfEntry { altitude_ft: 10000, speed_knots: 150.0, fuel_burn_rate: 58.0 },
            PerfEntry { altitude_ft: 20000, speed_knots: 140.0, fuel_burn_rate: 50.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 20.0, min_runway_ft: 1200, cold_weather_limit_c: -50, de_ice_required_below_c: 0 },
        cockpit_description: "Rugged analog cockpit with float operation instruments and water rudder controls.",
        history_blurb: "The Twin Otter is the ultimate bush and float plane — capable of landing on water, snow, ice, or gravel with legendary STOL performance.",
    });

    // 8. ATR 72-600 (cargo turboprop)
    map.insert("atr_72".into(), AircraftSpec {
        aircraft_id: "atr_72".into(),
        wingspan_ft: 88.8,
        length_ft: 89.1,
        height_ft: 25.1,
        empty_weight_lbs: 29000.0,
        max_takeoff_weight_lbs: 50700.0,
        fuel_capacity_gal: 1340.0,
        fuel_burn_gph: 120.0,
        v_speeds: VSpeeds { vs: 95.0, v1: 110.0, vr: 113.0, v2: 120.0, vne: 250.0 },
        ceiling_ft: 25000,
        rate_of_climb_fpm: 1300,
        stall_speed_knots: 95.0,
        approach_speed_knots: 115.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 260.0, fuel_burn_rate: 120.0 },
            PerfEntry { altitude_ft: 10000, speed_knots: 270.0, fuel_burn_rate: 110.0 },
            PerfEntry { altitude_ft: 20000, speed_knots: 250.0, fuel_burn_rate: 95.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 35.0, min_runway_ft: 3936, cold_weather_limit_c: -45, de_ice_required_below_c: 5 },
        cockpit_description: "Thales avionics suite with multi-function displays and fly-by-wire.",
        history_blurb: "The ATR 72 is the world's best-selling regional turboprop, prized for fuel efficiency on short routes. The cargo variant hauls freight across remote routes.",
    });

    // 9. Pilatus PC-12 (single turboprop, mapped to Turboprop class)
    map.insert("pc_12".into(), AircraftSpec {
        aircraft_id: "pc_12".into(),
        wingspan_ft: 53.3,
        length_ft: 47.3,
        height_ft: 14.0,
        empty_weight_lbs: 6600.0,
        max_takeoff_weight_lbs: 10450.0,
        fuel_capacity_gal: 402.0,
        fuel_burn_gph: 60.0,
        v_speeds: VSpeeds { vs: 67.0, v1: 85.0, vr: 85.0, v2: 94.0, vne: 240.0 },
        ceiling_ft: 30000,
        rate_of_climb_fpm: 1720,
        stall_speed_knots: 67.0,
        approach_speed_knots: 90.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 260.0, fuel_burn_rate: 60.0 },
            PerfEntry { altitude_ft: 15000, speed_knots: 270.0, fuel_burn_rate: 52.0 },
            PerfEntry { altitude_ft: 28000, speed_knots: 255.0, fuel_burn_rate: 45.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 25.0, min_runway_ft: 2500, cold_weather_limit_c: -45, de_ice_required_below_c: 3 },
        cockpit_description: "Honeywell Apex avionics with four large screens and integrated autopilot.",
        history_blurb: "The Pilatus PC-12 is the Swiss Army knife of aviation — equally at home on paved runways and dirt strips, combining turbine power with single-engine simplicity.",
    });

    // 10. Beechcraft Baron 58 (twin prop, second offering)
    map.insert("baron_58".into(), AircraftSpec {
        aircraft_id: "baron_58".into(),
        wingspan_ft: 37.8,
        length_ft: 29.8,
        height_ft: 9.7,
        empty_weight_lbs: 3520.0,
        max_takeoff_weight_lbs: 5400.0,
        fuel_capacity_gal: 166.0,
        fuel_burn_gph: 30.0,
        v_speeds: VSpeeds { vs: 69.0, v1: 84.0, vr: 84.0, v2: 92.0, vne: 223.0 },
        ceiling_ft: 20700,
        rate_of_climb_fpm: 1670,
        stall_speed_knots: 69.0,
        approach_speed_knots: 90.0,
        performance_table: vec![
            PerfEntry { altitude_ft: 0, speed_knots: 197.0, fuel_burn_rate: 30.0 },
            PerfEntry { altitude_ft: 10000, speed_knots: 190.0, fuel_burn_rate: 27.0 },
            PerfEntry { altitude_ft: 18000, speed_knots: 178.0, fuel_burn_rate: 24.0 },
        ],
        limits: OperatingLimits { max_crosswind_knots: 17.0, min_runway_ft: 2400, cold_weather_limit_c: -35, de_ice_required_below_c: 0 },
        cockpit_description: "Garmin G1000 glass panel with dual screens, traffic, and terrain awareness.",
        history_blurb: "The Baron 58 has been in production since 1969, offering twin-engine safety with responsive handling — a natural step up from singles.",
    });

    map
}
