//! Detailed instrument panel simulation with lag, damping, and failures.

use bevy::prelude::*;
use crate::shared::*;

pub struct InstrumentPlugin;

impl Plugin for InstrumentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InstrumentPanel>()
            .add_systems(
                Update,
                (
                    update_instruments,
                    apply_instrument_failures,
                    update_visibility,
                )
                    .run_if(in_state(GameState::Flying)),
            );
    }
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug)]
pub struct InstrumentPanel {
    // Displayed (lagged) readings
    pub altimeter: f32,
    pub airspeed_indicator: f32,
    pub heading_indicator: f32,
    pub vertical_speed: f32,
    pub turn_coordinator: f32,
    pub attitude_pitch: f32,
    pub attitude_roll: f32,

    // Instrument reliability
    pub altimeter_reliable: bool,
    pub airspeed_reliable: bool,
    pub heading_reliable: bool,
    pub vsi_reliable: bool,
    pub attitude_reliable: bool,

    // Visibility
    pub visibility_multiplier: f32,
    pub night_penalty_active: bool,

    // Internal tracking for VSI
    previous_altitude: f32,
    lag_factor: f32,
    error_amplitude: f32,
    error_timer: f32,
}

impl Default for InstrumentPanel {
    fn default() -> Self {
        Self {
            altimeter: 0.0,
            airspeed_indicator: 0.0,
            heading_indicator: 0.0,
            vertical_speed: 0.0,
            turn_coordinator: 0.0,
            attitude_pitch: 0.0,
            attitude_roll: 0.0,
            altimeter_reliable: true,
            airspeed_reliable: true,
            heading_reliable: true,
            vsi_reliable: true,
            attitude_reliable: true,
            visibility_multiplier: 1.0,
            night_penalty_active: false,
            previous_altitude: 0.0,
            lag_factor: 0.85,
            error_amplitude: 0.0,
            error_timer: 0.0,
        }
    }
}

impl InstrumentPanel {
    pub fn all_reliable(&self) -> bool {
        self.altimeter_reliable
            && self.airspeed_reliable
            && self.heading_reliable
            && self.vsi_reliable
            && self.attitude_reliable
    }

    pub fn reset_reliability(&mut self) {
        self.altimeter_reliable = true;
        self.airspeed_reliable = true;
        self.heading_reliable = true;
        self.vsi_reliable = true;
        self.attitude_reliable = true;
        self.error_amplitude = 0.0;
    }

    fn lerp(current: f32, target: f32, factor: f32) -> f32 {
        current + (target - current) * (1.0 - factor)
    }

    fn instrument_error(&self, base: f32) -> f32 {
        if self.error_amplitude <= 0.0 {
            return 0.0;
        }
        // Deterministic wobble based on timer
        let wobble = (self.error_timer * 3.7).sin() * self.error_amplitude;
        wobble * base.abs().max(1.0) * 0.05
    }
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn update_instruments(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    mut panel: ResMut<InstrumentPanel>,
) {
    let dt = time.delta_secs();
    panel.error_timer += dt;

    let lag = panel.lag_factor;

    // Altimeter — lags behind actual altitude
    let alt_target = flight_state.altitude_ft;
    let alt_error = if panel.altimeter_reliable {
        0.0
    } else {
        panel.instrument_error(alt_target)
    };
    panel.altimeter = InstrumentPanel::lerp(panel.altimeter, alt_target + alt_error, lag);

    // Airspeed indicator
    let spd_target = flight_state.speed_knots;
    let spd_error = if panel.airspeed_reliable {
        0.0
    } else {
        panel.instrument_error(spd_target)
    };
    panel.airspeed_indicator =
        InstrumentPanel::lerp(panel.airspeed_indicator, spd_target + spd_error, lag);

    // Heading indicator
    let hdg_target = flight_state.heading_deg;
    let hdg_error = if panel.heading_reliable {
        0.0
    } else {
        panel.instrument_error(hdg_target) * 2.0
    };
    panel.heading_indicator =
        InstrumentPanel::lerp(panel.heading_indicator, hdg_target + hdg_error, lag);
    // Wrap heading
    if panel.heading_indicator < 0.0 {
        panel.heading_indicator += 360.0;
    }
    if panel.heading_indicator >= 360.0 {
        panel.heading_indicator -= 360.0;
    }

    // Vertical speed indicator (ft/min from altitude delta)
    let raw_vsi = (flight_state.altitude_ft - panel.previous_altitude) / dt.max(0.001) * 60.0;
    let vsi_error = if panel.vsi_reliable {
        0.0
    } else {
        panel.instrument_error(raw_vsi)
    };
    panel.vertical_speed = InstrumentPanel::lerp(
        panel.vertical_speed,
        raw_vsi + vsi_error,
        lag * 0.95, // VSI lags a bit more
    );
    panel.previous_altitude = flight_state.altitude_ft;

    // Turn coordinator — based on heading change rate
    let hdg_rate = (flight_state.heading_deg - panel.heading_indicator).abs();
    panel.turn_coordinator = InstrumentPanel::lerp(panel.turn_coordinator, hdg_rate, lag);

    // Attitude indicator
    let pitch_target = match flight_state.phase {
        FlightPhase::Climb => 10.0,
        FlightPhase::Descent | FlightPhase::Approach => -5.0,
        FlightPhase::Takeoff => 15.0,
        FlightPhase::Landing => -3.0,
        _ => 0.0,
    };
    let pitch_error = if panel.attitude_reliable {
        0.0
    } else {
        panel.instrument_error(pitch_target) * 3.0
    };
    panel.attitude_pitch =
        InstrumentPanel::lerp(panel.attitude_pitch, pitch_target + pitch_error, lag);

    // Roll from turbulence
    let roll_target = flight_state.turbulence_shake * 2.0 * (panel.error_timer * 1.3).sin();
    panel.attitude_roll = InstrumentPanel::lerp(panel.attitude_roll, roll_target, lag);
}

pub fn apply_instrument_failures(
    flight_state: Res<FlightState>,
    mut panel: ResMut<InstrumentPanel>,
) {
    if flight_state.phase != FlightPhase::Emergency {
        if !panel.all_reliable() {
            panel.reset_reliability();
        }
        return;
    }

    // During emergencies, some instruments become unreliable
    panel.altimeter_reliable = false;
    panel.heading_reliable = false;
    panel.error_amplitude = 3.0;
}

pub fn update_visibility(
    weather: Res<WeatherState>,
    calendar: Res<Calendar>,
    mut panel: ResMut<InstrumentPanel>,
) {
    let weather_vis = weather.current.visibility_modifier();
    let night_vis = if calendar.is_night() { 0.6 } else { 1.0 };
    panel.night_penalty_active = calendar.is_night();
    panel.visibility_multiplier = weather_vis * night_vis;
}
