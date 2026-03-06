//! In-flight weather effects — turbulence, icing, crosswind, visibility, windshear.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Icing accumulation state during flight.
#[derive(Clone, Debug)]
pub struct IcingState {
    pub accumulation: f32, // 0.0 = none, 1.0 = severe
    pub de_icing_active: bool,
    pub de_icing_fuel_cost: f32,
}

impl Default for IcingState {
    fn default() -> Self {
        Self {
            accumulation: 0.0,
            de_icing_active: false,
            de_icing_fuel_cost: 0.5,
        }
    }
}

/// Crosswind correction data.
#[derive(Clone, Debug, Default)]
pub struct CrosswindState {
    pub crosswind_component: f32,
    pub headwind_component: f32,
    pub required_crab_angle: f32,
}

/// Windshear warning data.
#[derive(Clone, Debug, Default)]
pub struct WindshearState {
    pub detected: bool,
    pub severity: f32,
    pub altitude_loss_ft: f32,
}

/// Active weather effects resource.
#[derive(Resource, Clone, Debug, Default)]
pub struct WeatherEffects {
    pub icing: IcingState,
    pub crosswind: CrosswindState,
    pub windshear: WindshearState,
    pub effective_visibility_nm: f32,
    pub turbulence_offset_alt: f32,
    pub turbulence_offset_hdg: f32,
    pub mountain_wave_active: bool,
    pub mountain_wave_severity: f32,
}

// Temperature model
fn temperature_at_altitude(ground_temp_c: f32, altitude_ft: f32) -> f32 {
    // Standard lapse rate: -2°C per 1000ft
    ground_temp_c - (altitude_ft / 1000.0) * 2.0
}

// Icing conditions: between -20°C and 0°C in visible moisture
fn icing_rate(temp_c: f32, weather: Weather) -> f32 {
    if !(-20.0..=0.0).contains(&temp_c) {
        return 0.0;
    }
    let moisture = match weather {
        Weather::Clear | Weather::Windy => 0.0,
        Weather::Cloudy => 0.3,
        Weather::Rain => 0.6,
        Weather::Fog => 0.8,
        Weather::Snow => 1.0,
        Weather::Storm => 0.9,
    };
    let temp_factor = 1.0 - ((temp_c + 10.0).abs() / 10.0).min(1.0);
    moisture * temp_factor * 0.02 // accumulation per second
}

// Calculate crosswind/headwind from wind direction vs runway heading
fn wind_components(wind_dir: f32, wind_speed: f32, runway_heading: f32) -> (f32, f32) {
    let angle_rad = (wind_dir - runway_heading).to_radians();
    let crosswind = wind_speed * angle_rad.sin();
    let headwind = wind_speed * angle_rad.cos();
    (crosswind, headwind)
}

// Airports near mountains that can produce mountain waves
fn has_mountain_terrain(airport: AirportId) -> bool {
    matches!(
        airport,
        AirportId::Frostpeak | AirportId::Cloudmere | AirportId::Stormwatch
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Apply random turbulence bumps during flight.
pub fn apply_turbulence(
    time: Res<Time>,
    weather: Res<WeatherState>,
    mut flight_state: ResMut<FlightState>,
    mut effects: ResMut<WeatherEffects>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent | FlightPhase::Approach
    ) {
        effects.turbulence_offset_alt = 0.0;
        effects.turbulence_offset_hdg = 0.0;
        return;
    }

    let dt = time.delta_secs();
    let severity = match weather.turbulence_level {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 1.0,
        TurbulenceLevel::Moderate => 3.0,
        TurbulenceLevel::Severe => 6.0,
    };

    if severity == 0.0 {
        effects.turbulence_offset_alt = 0.0;
        effects.turbulence_offset_hdg = 0.0;
        return;
    }

    // Pseudo-random bumps using elapsed time
    let t = time.elapsed_secs();
    let alt_bump = (t * 2.7).sin() * severity * 50.0 * dt;
    let hdg_bump = (t * 1.9).cos() * severity * 2.0 * dt;

    effects.turbulence_offset_alt = alt_bump;
    effects.turbulence_offset_hdg = hdg_bump;

    flight_state.altitude_ft += alt_bump;
    flight_state.heading_deg = (flight_state.heading_deg + hdg_bump).rem_euclid(360.0);
    flight_state.turbulence_shake = severity;

    // Mountain wave amplification
    if effects.mountain_wave_active {
        let wave_bump = (t * 0.8).sin() * effects.mountain_wave_severity * 80.0 * dt;
        flight_state.altitude_ft += wave_bump;
    }
}

/// Accumulate ice on wings in cold/wet conditions; de-icing reduces it.
pub fn update_icing(
    time: Res<Time>,
    weather: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut effects: ResMut<WeatherEffects>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase == FlightPhase::Idle || flight_state.phase == FlightPhase::Arrived {
        effects.icing.accumulation = 0.0;
        return;
    }

    let dt = time.delta_secs();
    let ground_temp = match weather.current {
        Weather::Snow => -5.0,
        Weather::Storm => 5.0,
        Weather::Clear => 15.0,
        _ => 10.0,
    };
    let temp = temperature_at_altitude(ground_temp, flight_state.altitude_ft);
    let rate = icing_rate(temp, weather.current);

    let prev_level = effects.icing.accumulation;

    if effects.icing.de_icing_active {
        effects.icing.accumulation = (effects.icing.accumulation - 0.05 * dt).max(0.0);
    } else {
        effects.icing.accumulation = (effects.icing.accumulation + rate * dt).min(1.0);
    }

    // Threshold warnings
    if prev_level < 0.3 && effects.icing.accumulation >= 0.3 {
        toast_events.send(ToastEvent {
            message: "⚠ Light icing detected. Consider activating de-icing.".to_string(),
            duration_secs: 4.0,
        });
    }
    if prev_level < 0.6 && effects.icing.accumulation >= 0.6 {
        toast_events.send(ToastEvent {
            message: "⚠ Moderate icing! Lift degraded. Activate de-icing immediately!".to_string(),
            duration_secs: 5.0,
        });
    }
    if prev_level < 0.9 && effects.icing.accumulation >= 0.9 {
        toast_events.send(ToastEvent {
            message: "🚨 SEVERE ICING! Critical lift loss. Change altitude NOW!".to_string(),
            duration_secs: 5.0,
        });
    }
}

/// Toggle de-icing system with flaps key (reused binding).
pub fn toggle_de_icing(
    input: Res<PlayerInput>,
    flight_state: Res<FlightState>,
    mut effects: ResMut<WeatherEffects>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase == FlightPhase::Idle {
        return;
    }
    // De-icing toggled with hotbar_3
    if !input.hotbar_3 {
        return;
    }

    effects.icing.de_icing_active = !effects.icing.de_icing_active;
    let status = if effects.icing.de_icing_active {
        "ON — increased fuel burn"
    } else {
        "OFF"
    };
    toast_events.send(ToastEvent {
        message: format!("De-icing system: {}", status),
        duration_secs: 2.0,
    });
}

/// De-icing fuel penalty.
pub fn de_icing_fuel_cost(
    time: Res<Time>,
    effects: Res<WeatherEffects>,
    mut flight_state: ResMut<FlightState>,
) {
    if effects.icing.de_icing_active
        && !matches!(flight_state.phase, FlightPhase::Idle | FlightPhase::Arrived)
    {
        flight_state.fuel_remaining -= effects.icing.de_icing_fuel_cost * time.delta_secs() / 60.0;
    }
}

/// Calculate crosswind components during approach/takeoff.
pub fn update_crosswind(
    weather: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut effects: ResMut<WeatherEffects>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Takeoff | FlightPhase::Approach | FlightPhase::Landing
    ) {
        effects.crosswind = CrosswindState::default();
        return;
    }

    let runway_heading = flight_state.heading_deg;
    let (cross, head) = wind_components(
        weather.wind_direction_deg,
        weather.wind_speed_knots,
        runway_heading,
    );

    let prev_cross = effects.crosswind.crosswind_component.abs();
    effects.crosswind.crosswind_component = cross;
    effects.crosswind.headwind_component = head;
    effects.crosswind.required_crab_angle = if weather.wind_speed_knots > 0.0 {
        (cross / flight_state.speed_knots.max(60.0))
            .atan()
            .to_degrees()
    } else {
        0.0
    };

    // Warn on strong crosswind
    if prev_cross < 15.0 && cross.abs() >= 15.0 {
        toast_events.send(ToastEvent {
            message: format!(
                "⚠ Strong crosswind: {:.0} knots. Apply correction.",
                cross.abs()
            ),
            duration_secs: 4.0,
        });
    }
}

/// Update effective visibility from weather conditions.
pub fn update_visibility(weather: Res<WeatherState>, mut effects: ResMut<WeatherEffects>) {
    effects.effective_visibility_nm = weather.visibility_nm * weather.current.visibility_modifier();
}

/// Detect windshear near ground during approach/takeoff.
pub fn detect_windshear(
    time: Res<Time>,
    weather: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut effects: ResMut<WeatherEffects>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Takeoff | FlightPhase::Approach | FlightPhase::Landing
    ) {
        effects.windshear = WindshearState::default();
        return;
    }

    // Windshear is more likely in storms and at low altitude
    let storm_factor = match weather.current {
        Weather::Storm => 0.8,
        Weather::Rain => 0.2,
        _ => 0.0,
    };
    let alt_factor = if flight_state.altitude_ft < 1500.0 {
        1.0
    } else {
        0.0
    };

    let t = time.elapsed_secs();
    let shear_chance = storm_factor * alt_factor;

    // Check periodically
    if shear_chance > 0.0 && (t * 0.3).sin().abs() > (1.0 - shear_chance) {
        if !effects.windshear.detected {
            effects.windshear.detected = true;
            effects.windshear.severity = shear_chance;
            effects.windshear.altitude_loss_ft = shear_chance * 200.0;

            toast_events.send(ToastEvent {
                message: "🚨 WINDSHEAR WARNING! Go around recommended!".to_string(),
                duration_secs: 5.0,
            });
        }
    } else {
        effects.windshear.detected = false;
        effects.windshear.severity = 0.0;
    }
}

/// Check for mountain wave turbulence near high-elevation airports.
pub fn check_mountain_waves(
    weather: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut effects: ResMut<WeatherEffects>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase == FlightPhase::Idle {
        effects.mountain_wave_active = false;
        return;
    }

    let near_mountains =
        has_mountain_terrain(flight_state.origin) || has_mountain_terrain(flight_state.destination);

    let strong_wind = weather.wind_speed_knots > 20.0;
    let was_active = effects.mountain_wave_active;

    effects.mountain_wave_active = near_mountains && strong_wind;
    effects.mountain_wave_severity = if effects.mountain_wave_active {
        (weather.wind_speed_knots - 20.0) / 30.0
    } else {
        0.0
    };

    if effects.mountain_wave_active && !was_active {
        toast_events.send(ToastEvent {
            message: "⚠ Mountain wave turbulence ahead. Expect altitude variations.".to_string(),
            duration_secs: 4.0,
        });
    }
}
