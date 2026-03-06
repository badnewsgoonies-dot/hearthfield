//! Autopilot system — automatic heading, altitude, and speed control.

use crate::shared::*;
use bevy::prelude::*;

pub struct AutopilotPlugin;

impl Plugin for AutopilotPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AutopilotState>().add_systems(
            Update,
            (toggle_autopilot, run_autopilot, autopilot_warnings)
                .run_if(in_state(GameState::Flying)),
        );
    }
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug)]
pub struct AutopilotState {
    pub engaged: bool,
    pub target_heading: f32,
    pub target_altitude: f32,
    pub target_speed: f32,
    pub heading_hold: bool,
    pub altitude_hold: bool,
    pub speed_hold: bool,
    pub available: bool,
}

impl Default for AutopilotState {
    fn default() -> Self {
        Self {
            engaged: false,
            target_heading: 0.0,
            target_altitude: 10000.0,
            target_speed: 200.0,
            heading_hold: true,
            altitude_hold: true,
            speed_hold: true,
            available: false,
        }
    }
}

impl AutopilotState {
    pub fn disengage(&mut self) {
        self.engaged = false;
    }

    pub fn capture_current(&mut self, flight: &FlightState) {
        self.target_heading = flight.heading_deg;
        self.target_altitude = flight.altitude_ft;
        self.target_speed = flight.speed_knots;
    }
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn toggle_autopilot(
    input: Res<PlayerInput>,
    pilot_state: Res<PilotState>,
    flight_state: Res<FlightState>,
    mut autopilot: ResMut<AutopilotState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    // Autopilot unlocked at Commercial rank
    autopilot.available = pilot_state.rank >= PilotRank::Commercial;

    if !input.confirm {
        return;
    }

    // Only toggle in cruise phases
    if !matches!(flight_state.phase, FlightPhase::Climb | FlightPhase::Cruise) {
        return;
    }

    // Use the `autopilot` field on FlightState as the toggle trigger
    if !autopilot.available {
        toast_events.send(ToastEvent {
            message: "Autopilot requires Commercial rank.".to_string(),
            duration_secs: 3.0,
        });
        return;
    }

    if autopilot.engaged {
        autopilot.disengage();
        toast_events.send(ToastEvent {
            message: "Autopilot disengaged.".to_string(),
            duration_secs: 2.0,
        });
    } else {
        autopilot.capture_current(&flight_state);
        autopilot.engaged = true;
        toast_events.send(ToastEvent {
            message: format!(
                "Autopilot engaged — HDG {:.0}° ALT {:.0}ft SPD {:.0}kts",
                autopilot.target_heading, autopilot.target_altitude, autopilot.target_speed
            ),
            duration_secs: 3.0,
        });
    }
}

pub fn run_autopilot(
    time: Res<Time>,
    autopilot: Res<AutopilotState>,
    mut flight_state: ResMut<FlightState>,
) {
    if !autopilot.engaged {
        return;
    }

    let dt = time.delta_secs();
    let rate = 0.5; // convergence rate per second

    // Heading hold
    if autopilot.heading_hold {
        let mut diff = autopilot.target_heading - flight_state.heading_deg;
        if diff > 180.0 {
            diff -= 360.0;
        }
        if diff < -180.0 {
            diff += 360.0;
        }
        let adjust = diff.clamp(-30.0, 30.0) * rate * dt;
        flight_state.heading_deg = (flight_state.heading_deg + adjust) % 360.0;
        if flight_state.heading_deg < 0.0 {
            flight_state.heading_deg += 360.0;
        }
    }

    // Altitude hold
    if autopilot.altitude_hold {
        let alt_diff = autopilot.target_altitude - flight_state.altitude_ft;
        let climb_rate = alt_diff.clamp(-1500.0, 1500.0) * rate * dt;
        flight_state.altitude_ft += climb_rate;
    }

    // Speed hold via throttle adjustment
    if autopilot.speed_hold {
        let speed_diff = autopilot.target_speed - flight_state.speed_knots;
        let throttle_adj = (speed_diff / 200.0).clamp(-0.5, 0.5) * rate * dt;
        flight_state.throttle = (flight_state.throttle + throttle_adj).clamp(0.0, 1.0);
    }

    flight_state.autopilot = true;
}

pub fn autopilot_warnings(
    weather: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut autopilot: ResMut<AutopilotState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !autopilot.engaged {
        return;
    }

    // Disengage on severe turbulence
    if weather.turbulence_level == TurbulenceLevel::Severe {
        autopilot.disengage();
        toast_events.send(ToastEvent {
            message: "⚠ Autopilot disconnected — severe turbulence!".to_string(),
            duration_secs: 4.0,
        });
        return;
    }

    // Disengage on emergency
    if flight_state.phase == FlightPhase::Emergency {
        autopilot.disengage();
        toast_events.send(ToastEvent {
            message: "⚠ Autopilot disconnected — emergency!".to_string(),
            duration_secs: 4.0,
        });
        return;
    }

    // Disengage on approach/landing
    if matches!(
        flight_state.phase,
        FlightPhase::Approach | FlightPhase::Landing | FlightPhase::Descent
    ) {
        autopilot.disengage();
        toast_events.send(ToastEvent {
            message: "Autopilot disengaged for approach.".to_string(),
            duration_secs: 3.0,
        });
    }
}
