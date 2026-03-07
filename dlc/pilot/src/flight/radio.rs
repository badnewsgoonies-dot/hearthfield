//! ATC radio communication — request clearances and receive advisories.

use crate::shared::*;
use bevy::prelude::*;

pub struct RadioPlugin;

impl Plugin for RadioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RadioState>()
            .add_event::<RadioTransmitEvent>()
            .add_event::<RadioReceiveEvent>()
            .add_systems(
                Update,
                (
                    radio_communication.run_if(in_state(GameState::RadioComm)),
                    process_radio_response.run_if(in_state(GameState::RadioComm)),
                    ambient_radio_chatter.run_if(in_state(GameState::Flying)),
                ),
            );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RadioMessage {
    RequestTakeoff,
    RequestLanding,
    ReportPosition,
    EmergencyMayday,
    WeatherUpdate,
    TrafficAdvisory,
}

impl RadioMessage {
    pub fn display_text(&self) -> &'static str {
        match self {
            RadioMessage::RequestTakeoff => "Request takeoff clearance",
            RadioMessage::RequestLanding => "Request landing clearance",
            RadioMessage::ReportPosition => "Report current position",
            RadioMessage::EmergencyMayday => "MAYDAY MAYDAY MAYDAY",
            RadioMessage::WeatherUpdate => "Request weather update",
            RadioMessage::TrafficAdvisory => "Request traffic advisory",
        }
    }

    pub fn callsign_prefix(&self) -> &'static str {
        match self {
            RadioMessage::EmergencyMayday => "MAYDAY",
            _ => "Tower",
        }
    }
}

#[derive(Clone, Debug)]
pub enum AtcResponse {
    Cleared { message: String },
    Denied { reason: String },
    Advisory { message: String },
    Acknowledged,
}

impl AtcResponse {
    pub fn text(&self) -> &str {
        match self {
            AtcResponse::Cleared { message } => message,
            AtcResponse::Denied { reason } => reason,
            AtcResponse::Advisory { message } => message,
            AtcResponse::Acknowledged => "Roger, acknowledged.",
        }
    }
}

// ── Events ───────────────────────────────────────────────────────────────

#[derive(Event, Clone, Debug)]
pub struct RadioTransmitEvent {
    pub message: RadioMessage,
}

#[derive(Event, Clone, Debug)]
pub struct RadioReceiveEvent {
    pub response_text: String,
    pub is_clearance: bool,
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct RadioState {
    pub frequency: String,
    pub selected_option: usize,
    pub available_messages: Vec<RadioMessage>,
    pub last_response: Option<AtcResponse>,
    pub takeoff_cleared: bool,
    pub landing_cleared: bool,
    pub chatter_timer: f32,
}

impl RadioState {
    pub fn build_options(&mut self, flight_phase: FlightPhase) {
        self.available_messages.clear();
        match flight_phase {
            FlightPhase::Taxi | FlightPhase::Preflight => {
                self.available_messages.push(RadioMessage::RequestTakeoff);
                self.available_messages.push(RadioMessage::WeatherUpdate);
            }
            FlightPhase::Approach | FlightPhase::Descent => {
                self.available_messages.push(RadioMessage::RequestLanding);
                self.available_messages.push(RadioMessage::ReportPosition);
                self.available_messages.push(RadioMessage::WeatherUpdate);
            }
            FlightPhase::Emergency => {
                self.available_messages.push(RadioMessage::EmergencyMayday);
                self.available_messages.push(RadioMessage::ReportPosition);
            }
            _ => {
                self.available_messages.push(RadioMessage::ReportPosition);
                self.available_messages.push(RadioMessage::WeatherUpdate);
                self.available_messages.push(RadioMessage::TrafficAdvisory);
            }
        }
        self.selected_option = 0;
    }
}

// ── Ambient messages ─────────────────────────────────────────────────────

const AMBIENT_CHATTER: &[&str] = &[
    "Skywarden 42, turn left heading 270, maintain flight level 350.",
    "All stations, wind check: 310 at 15 gusting 22.",
    "Skybird 7, cleared for visual approach runway 28L.",
    "Ground, Delta 515, request pushback gate A12.",
    "Advisory: moderate turbulence reported at FL280.",
    "Cessna 9-Alpha-Bravo, squawk 4521, radar contact.",
    "Attention all aircraft, temporary flight restriction in effect.",
    "Skywarden tower, ATIS information Golf is current.",
];

// ── Systems ──────────────────────────────────────────────────────────────

pub fn radio_communication(
    input: Res<PlayerInput>,
    flight_state: Res<FlightState>,
    mut radio: ResMut<RadioState>,
    mut transmit_events: EventWriter<RadioTransmitEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    if radio.available_messages.is_empty() {
        radio.build_options(flight_state.phase);
    }

    // Navigate options
    if input.menu_up && radio.selected_option > 0 {
        radio.selected_option -= 1;
    }
    if input.menu_down && radio.selected_option + 1 < radio.available_messages.len() {
        radio.selected_option += 1;
    }

    // Transmit
    if input.menu_confirm {
        if let Some(msg) = radio.available_messages.get(radio.selected_option).cloned() {
            transmit_events.send(RadioTransmitEvent { message: msg });
        }
    }

    // Close radio
    if input.cancel {
        game_state.set(GameState::Flying);
    }
}

pub fn process_radio_response(
    mut transmit_events: EventReader<RadioTransmitEvent>,
    mut radio: ResMut<RadioState>,
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut receive_events: EventWriter<RadioReceiveEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in transmit_events.read() {
        let response = match &ev.message {
            RadioMessage::RequestTakeoff => {
                if weather.current.is_flyable() {
                    radio.takeoff_cleared = true;
                    AtcResponse::Cleared {
                        message: format!(
                            "{}, cleared for takeoff runway 28. Wind {} at {:.0} knots.",
                            flight_state.origin.icao_code(),
                            weather.wind_direction_deg as u32,
                            weather.wind_speed_knots
                        ),
                    }
                } else {
                    AtcResponse::Denied {
                        reason: "Takeoff denied — weather below minimums.".to_string(),
                    }
                }
            }
            RadioMessage::RequestLanding => {
                radio.landing_cleared = true;
                AtcResponse::Cleared {
                    message: format!(
                        "{}, cleared ILS approach runway 28. Winds {} at {:.0}.",
                        flight_state.destination.icao_code(),
                        weather.wind_direction_deg as u32,
                        weather.wind_speed_knots
                    ),
                }
            }
            RadioMessage::ReportPosition => AtcResponse::Acknowledged,
            RadioMessage::EmergencyMayday => {
                radio.landing_cleared = true;
                AtcResponse::Cleared {
                    message: "Roger MAYDAY. All traffic cleared. Emergency services standing by."
                        .to_string(),
                }
            }
            RadioMessage::WeatherUpdate => AtcResponse::Advisory {
                message: format!(
                    "Current conditions: {:?}, visibility {:.1} nm, ceiling {} ft. Turbulence: {:?}.",
                    weather.current,
                    weather.visibility_nm,
                    weather.ceiling_ft,
                    weather.turbulence_level
                ),
            },
            RadioMessage::TrafficAdvisory => AtcResponse::Advisory {
                message: "No conflicting traffic observed in your vicinity.".to_string(),
            },
        };

        let is_clearance = matches!(&response, AtcResponse::Cleared { .. });

        receive_events.send(RadioReceiveEvent {
            response_text: response.text().to_string(),
            is_clearance,
        });
        toast_events.send(ToastEvent {
            message: format!("ATC: {}", response.text()),
            duration_secs: 4.0,
        });

        radio.last_response = Some(response);
    }
}

pub fn ambient_radio_chatter(
    time: Res<Time>,
    mut radio: ResMut<RadioState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    radio.chatter_timer += time.delta_secs();

    // Ambient chatter every ~45 seconds
    if radio.chatter_timer >= 45.0 {
        radio.chatter_timer = 0.0;
        let idx = (time.elapsed_secs() * 7.0) as usize % AMBIENT_CHATTER.len();
        toast_events.send(ToastEvent {
            message: format!("📻 {}", AMBIENT_CHATTER[idx]),
            duration_secs: 3.0,
        });
    }
}
