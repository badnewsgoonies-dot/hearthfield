//! Air Traffic Control simulation — clearances, runway assignment, sequencing.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClearanceStatus {
    #[default]
    None,
    Requested,
    Cleared,
    HoldShortRunway,
    HoldPosition,
    Denied,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ApproachType {
    Visual,
    ILS,
    VOR,
    RNAV,
}

impl ApproachType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ApproachType::Visual => "Visual",
            ApproachType::ILS => "ILS",
            ApproachType::VOR => "VOR",
            ApproachType::RNAV => "RNAV",
        }
    }

    pub fn minimum_visibility_nm(&self) -> f32 {
        match self {
            ApproachType::Visual => 3.0,
            ApproachType::ILS => 0.5,
            ApproachType::VOR => 1.0,
            ApproachType::RNAV => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AtcFrequency {
    Ground,
    Tower,
    Departure,
    Approach,
    Center,
}

impl AtcFrequency {
    pub fn display_name(&self) -> &'static str {
        match self {
            AtcFrequency::Ground => "Ground",
            AtcFrequency::Tower => "Tower",
            AtcFrequency::Departure => "Departure",
            AtcFrequency::Approach => "Approach",
            AtcFrequency::Center => "Center",
        }
    }

    pub fn frequency_mhz(&self) -> &'static str {
        match self {
            AtcFrequency::Ground => "121.9",
            AtcFrequency::Tower => "118.7",
            AtcFrequency::Departure => "124.3",
            AtcFrequency::Approach => "119.1",
            AtcFrequency::Center => "132.5",
        }
    }
}

#[derive(Clone, Debug)]
pub struct HoldingPattern {
    pub active: bool,
    pub fix_name: String,
    pub altitude_ft: f32,
    pub laps_completed: u32,
    pub max_laps: u32,
    pub timer: f32,
}

impl Default for HoldingPattern {
    fn default() -> Self {
        Self {
            active: false,
            fix_name: String::new(),
            altitude_ft: 5000.0,
            laps_completed: 0,
            max_laps: 3,
            timer: 0.0,
        }
    }
}

// ── ATC State ────────────────────────────────────────────────────────────

#[derive(Resource, Clone, Debug)]
pub struct AtcState {
    pub clearance_status: ClearanceStatus,
    pub assigned_runway: String,
    pub altitude_restriction: Option<f32>,
    pub speed_restriction: Option<f32>,
    pub holding_pattern: HoldingPattern,
    pub approach_type: Option<ApproachType>,
    pub current_frequency: AtcFrequency,
    pub queue_position: u32,
    pub traffic_density: TrafficDensity,
    pub go_around_issued: bool,
    pub handoff_pending: bool,
    pub squawk_code: u16,
}

impl Default for AtcState {
    fn default() -> Self {
        Self {
            clearance_status: ClearanceStatus::None,
            assigned_runway: "28".to_string(),
            altitude_restriction: None,
            speed_restriction: None,
            holding_pattern: HoldingPattern::default(),
            approach_type: None,
            current_frequency: AtcFrequency::Ground,
            queue_position: 0,
            traffic_density: TrafficDensity::Light,
            go_around_issued: false,
            handoff_pending: false,
            squawk_code: 1200,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrafficDensity {
    Light,
    Moderate,
    Heavy,
}

impl TrafficDensity {
    pub fn queue_delay_secs(&self) -> f32 {
        match self {
            TrafficDensity::Light => 0.0,
            TrafficDensity::Moderate => 30.0,
            TrafficDensity::Heavy => 90.0,
        }
    }

    pub fn from_airport(airport: AirportId) -> Self {
        match airport {
            AirportId::HomeBase | AirportId::Duskhollow => TrafficDensity::Light,
            AirportId::Windport | AirportId::Frostpeak | AirportId::Cloudmere => {
                TrafficDensity::Moderate
            }
            AirportId::Sunhaven
            | AirportId::Ironforge
            | AirportId::Stormwatch
            | AirportId::Grandcity
            | AirportId::Skyreach => TrafficDensity::Heavy,
        }
    }
}

// Runway assignments per airport
fn assigned_runway_for(airport: AirportId, wind_dir: f32) -> &'static str {
    let primary = match airport {
        AirportId::HomeBase => ("28", "10"),
        AirportId::Windport => ("27L", "09R"),
        AirportId::Frostpeak => ("36", "18"),
        AirportId::Sunhaven => ("25", "07"),
        AirportId::Ironforge => ("30", "12"),
        AirportId::Cloudmere => ("34", "16"),
        AirportId::Duskhollow => ("22", "04"),
        AirportId::Stormwatch => ("27", "09"),
        AirportId::Grandcity => ("28L", "10R"),
        AirportId::Skyreach => ("35L", "17R"),
    };
    if wind_dir >= 180.0 { primary.0 } else { primary.1 }
}

// ATC phraseology
fn takeoff_clearance_msg(icao: &str, runway: &str, wind_dir: u32, wind_spd: f32) -> String {
    format!(
        "{}, cleared for takeoff runway {}. Wind {} at {:.0} knots.",
        icao, runway, wind_dir, wind_spd
    )
}

fn landing_clearance_msg(
    icao: &str,
    runway: &str,
    approach: &ApproachType,
    wind_dir: u32,
    wind_spd: f32,
) -> String {
    format!(
        "{}, cleared {} approach runway {}. Wind {} at {:.0}.",
        icao,
        approach.display_name(),
        runway,
        wind_dir,
        wind_spd
    )
}

fn hold_short_msg(icao: &str, runway: &str, queue: u32) -> String {
    format!(
        "{}, hold short runway {}. Traffic in the pattern, number {} for departure.",
        icao, runway, queue
    )
}

fn altitude_restriction_msg(icao: &str, alt: f32) -> String {
    format!("{}, maintain {:.0}, traffic at your altitude.", icao, alt)
}

fn go_around_msg(icao: &str) -> String {
    format!(
        "{}, go around! Climb and maintain 3000, fly heading 270.",
        icao
    )
}

fn handoff_msg(icao: &str, _from: &AtcFrequency, to: &AtcFrequency) -> String {
    format!(
        "{}, contact {} on {}. Good day.",
        icao,
        to.display_name(),
        to.frequency_mhz()
    )
}

fn holding_msg(icao: &str, fix: &str, alt: f32) -> String {
    format!(
        "{}, hold as published at {}, maintain {:.0}. Expect further clearance in 5 minutes.",
        icao, fix, alt
    )
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Assign runway and traffic density when a flight begins.
pub fn setup_atc_on_flight_start(
    mut flight_start_events: EventReader<FlightStartEvent>,
    weather: Res<WeatherState>,
    mut atc: ResMut<AtcState>,
) {
    for ev in flight_start_events.read() {
        let runway = assigned_runway_for(ev.origin, weather.wind_direction_deg);
        atc.assigned_runway = runway.to_string();
        atc.clearance_status = ClearanceStatus::None;
        atc.traffic_density = TrafficDensity::from_airport(ev.origin);
        atc.queue_position = match atc.traffic_density {
            TrafficDensity::Light => 0,
            TrafficDensity::Moderate => 1,
            TrafficDensity::Heavy => 3,
        };
        atc.current_frequency = AtcFrequency::Ground;
        atc.go_around_issued = false;
        atc.handoff_pending = false;
        atc.approach_type = None;
        atc.holding_pattern = HoldingPattern::default();
        atc.altitude_restriction = None;
        atc.speed_restriction = None;
        atc.squawk_code = 1200 + (ev.origin as u16 * 100);
    }
}

/// Process takeoff clearance requests — manage queue and clearance.
pub fn process_takeoff_clearance(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut atc: ResMut<AtcState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(flight_state.phase, FlightPhase::Taxi | FlightPhase::Preflight) {
        return;
    }

    if atc.clearance_status != ClearanceStatus::Requested {
        return;
    }

    // Queue processing
    if atc.queue_position > 0 {
        atc.holding_pattern.timer += time.delta_secs();
        let delay_per = atc.traffic_density.queue_delay_secs().max(10.0);
        if atc.holding_pattern.timer >= delay_per {
            atc.holding_pattern.timer = 0.0;
            atc.queue_position -= 1;

            let icao = flight_state.origin.icao_code();
            toast_events.send(ToastEvent {
                message: hold_short_msg(icao, &atc.assigned_runway, atc.queue_position),
                duration_secs: 4.0,
            });
        }
        return;
    }

    // Check weather
    if !weather.current.is_flyable() {
        atc.clearance_status = ClearanceStatus::Denied;
        toast_events.send(ToastEvent {
            message: format!(
                "ATC: {}, takeoff denied. Weather below minimums.",
                flight_state.origin.icao_code()
            ),
            duration_secs: 4.0,
        });
        return;
    }

    // Clear for takeoff
    atc.clearance_status = ClearanceStatus::Cleared;
    atc.current_frequency = AtcFrequency::Tower;
    let msg = takeoff_clearance_msg(
        flight_state.origin.icao_code(),
        &atc.assigned_runway,
        weather.wind_direction_deg as u32,
        weather.wind_speed_knots,
    );
    toast_events.send(ToastEvent {
        message: format!("ATC: {}", msg),
        duration_secs: 5.0,
    });
}

/// Handle frequency handoffs during flight phases.
pub fn handle_frequency_handoffs(
    flight_state: Res<FlightState>,
    mut atc: ResMut<AtcState>,
    mut phase_events: EventReader<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in phase_events.read() {
        let icao = flight_state.origin.icao_code();
        let (from, to) = match ev.new_phase {
            FlightPhase::Climb => (AtcFrequency::Tower, AtcFrequency::Departure),
            FlightPhase::Cruise => (AtcFrequency::Departure, AtcFrequency::Center),
            FlightPhase::Descent => (AtcFrequency::Center, AtcFrequency::Approach),
            FlightPhase::Approach => (AtcFrequency::Approach, AtcFrequency::Tower),
            _ => continue,
        };

        if atc.current_frequency == from || atc.current_frequency != to {
            let msg = handoff_msg(icao, &from, &to);
            atc.current_frequency = to;
            toast_events.send(ToastEvent {
                message: format!("ATC: {}", msg),
                duration_secs: 4.0,
            });
        }
    }
}

/// Assign approach type based on weather/visibility.
pub fn assign_approach(
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut atc: ResMut<AtcState>,
    mut phase_events: EventReader<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in phase_events.read() {
        if ev.new_phase != FlightPhase::Approach {
            continue;
        }

        let approach = if weather.visibility_nm >= 3.0 {
            ApproachType::Visual
        } else if weather.visibility_nm >= 1.0 {
            ApproachType::VOR
        } else {
            ApproachType::ILS
        };

        atc.approach_type = Some(approach);

        let runway = assigned_runway_for(flight_state.destination, weather.wind_direction_deg);
        atc.assigned_runway = runway.to_string();
        atc.clearance_status = ClearanceStatus::Cleared;

        let msg = landing_clearance_msg(
            flight_state.destination.icao_code(),
            runway,
            &approach,
            weather.wind_direction_deg as u32,
            weather.wind_speed_knots,
        );
        toast_events.send(ToastEvent {
            message: format!("ATC: {}", msg),
            duration_secs: 5.0,
        });

        // Speed/altitude restrictions on approach
        atc.speed_restriction = Some(180.0);
        atc.altitude_restriction = Some(3000.0);
    }
}

/// Issue go-around if approach is unstable.
pub fn evaluate_go_around(
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut atc: ResMut<AtcState>,
    mut toast_events: EventWriter<ToastEvent>,
    mut phase_events: EventWriter<FlightPhaseChangeEvent>,
) {
    if flight_state.phase != FlightPhase::Approach || atc.go_around_issued {
        return;
    }

    let approach = atc.approach_type.unwrap_or(ApproachType::Visual);
    let min_vis = approach.minimum_visibility_nm();

    // Unstable approach conditions
    let too_fast = flight_state.speed_knots > 200.0;
    let below_vis = weather.visibility_nm < min_vis;
    let too_high = flight_state.altitude_ft > 4000.0
        && flight_state.distance_remaining_nm < 5.0;

    if too_fast || below_vis || too_high {
        atc.go_around_issued = true;
        atc.clearance_status = ClearanceStatus::None;
        atc.approach_type = None;

        let icao = flight_state.destination.icao_code();
        let msg = go_around_msg(icao);
        toast_events.send(ToastEvent {
            message: format!("⚠ ATC: {}", msg),
            duration_secs: 5.0,
        });

        phase_events.send(FlightPhaseChangeEvent {
            new_phase: FlightPhase::Climb,
        });
    }
}

/// Enforce altitude restrictions.
pub fn enforce_altitude_restriction(
    flight_state: Res<FlightState>,
    atc: Res<AtcState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent
    ) {
        return;
    }

    if let Some(restriction) = atc.altitude_restriction {
        let deviation = (flight_state.altitude_ft - restriction).abs();
        // Warn if significantly off assigned altitude
        if deviation > 500.0 && flight_state.altitude_ft > restriction + 500.0 {
            let icao = flight_state.origin.icao_code();
            let msg = altitude_restriction_msg(icao, restriction);
            toast_events.send(ToastEvent {
                message: format!("ATC: {}", msg),
                duration_secs: 3.0,
            });
        }
    }
}

/// Handle holding patterns at busy airports.
pub fn update_holding_pattern(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    mut atc: ResMut<AtcState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !atc.holding_pattern.active {
        // Initiate holding if approaching a busy airport
        if flight_state.phase == FlightPhase::Descent
            && atc.traffic_density == TrafficDensity::Heavy
            && !atc.holding_pattern.active
        {
            atc.holding_pattern.active = true;
            atc.holding_pattern.fix_name = format!("{} VOR", flight_state.destination.icao_code());
            atc.holding_pattern.altitude_ft = 5000.0;
            atc.holding_pattern.laps_completed = 0;
            atc.holding_pattern.timer = 0.0;

            let msg = holding_msg(
                flight_state.destination.icao_code(),
                &atc.holding_pattern.fix_name,
                atc.holding_pattern.altitude_ft,
            );
            toast_events.send(ToastEvent {
                message: format!("ATC: {}", msg),
                duration_secs: 5.0,
            });
        }
        return;
    }

    atc.holding_pattern.timer += time.delta_secs();

    // Each lap ~60 seconds
    if atc.holding_pattern.timer >= 60.0 {
        atc.holding_pattern.timer = 0.0;
        atc.holding_pattern.laps_completed += 1;

        if atc.holding_pattern.laps_completed >= atc.holding_pattern.max_laps {
            atc.holding_pattern.active = false;
            toast_events.send(ToastEvent {
                message: format!(
                    "ATC: {}, cleared to leave holding. Proceed direct for approach.",
                    flight_state.destination.icao_code()
                ),
                duration_secs: 4.0,
            });
        } else {
            toast_events.send(ToastEvent {
                message: format!(
                    "ATC: {}, continue holding. Lap {}/{}.",
                    flight_state.destination.icao_code(),
                    atc.holding_pattern.laps_completed,
                    atc.holding_pattern.max_laps
                ),
                duration_secs: 3.0,
            });
        }
    }
}

/// Request takeoff clearance (called from radio or auto-request).
pub fn request_takeoff_clearance(
    input: Res<PlayerInput>,
    flight_state: Res<FlightState>,
    mut atc: ResMut<AtcState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(flight_state.phase, FlightPhase::Taxi) {
        return;
    }
    if atc.clearance_status != ClearanceStatus::None {
        return;
    }
    if !input.radio {
        return;
    }

    atc.clearance_status = ClearanceStatus::Requested;
    atc.holding_pattern.timer = 0.0;

    toast_events.send(ToastEvent {
        message: format!(
            "You: {}, ready for departure runway {}.",
            flight_state.origin.icao_code(),
            atc.assigned_runway
        ),
        duration_secs: 3.0,
    });
}

/// Reset ATC state when flight completes.
pub fn reset_atc_on_arrival(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut atc: ResMut<AtcState>,
) {
    for _ev in flight_complete_events.read() {
        *atc = AtcState::default();
    }
}
