//! Ground operations — taxi, pushback, taxiway routing, de-icing, parking, turnaround.

use bevy::prelude::*;
use crate::shared::*;

// ─── Taxiway System ──────────────────────────────────────────────────────

/// Named taxiway segments at an airport.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaxiwayLetter {
    Alpha,
    Bravo,
    Charlie,
    Delta,
    Echo,
    Foxtrot,
}

impl TaxiwayLetter {
    pub fn display(&self) -> &'static str {
        match self {
            Self::Alpha => "Alpha",
            Self::Bravo => "Bravo",
            Self::Charlie => "Charlie",
            Self::Delta => "Delta",
            Self::Echo => "Echo",
            Self::Foxtrot => "Foxtrot",
        }
    }

    pub fn short(&self) -> &'static str {
        match self {
            Self::Alpha => "A",
            Self::Bravo => "B",
            Self::Charlie => "C",
            Self::Delta => "D",
            Self::Echo => "E",
            Self::Foxtrot => "F",
        }
    }
}

/// A step in a taxi route.
#[derive(Clone, Debug)]
pub struct TaxiStep {
    pub taxiway: TaxiwayLetter,
    pub hold_short: bool,
    pub instruction: String,
}

/// A full taxi route from point A to point B.
#[derive(Clone, Debug)]
pub struct TaxiRoute {
    pub steps: Vec<TaxiStep>,
    pub estimated_time_secs: f32,
}

/// Build a simple taxi route for a given airport.
pub fn build_taxi_route(airport: AirportId, to_runway: bool) -> TaxiRoute {
    let steps = match (airport, to_runway) {
        (AirportId::HomeBase, true) => vec![
            TaxiStep { taxiway: TaxiwayLetter::Alpha, hold_short: false, instruction: "Taxi via Alpha".into() },
            TaxiStep { taxiway: TaxiwayLetter::Bravo, hold_short: true, instruction: "Hold short Runway 27 on Bravo".into() },
        ],
        (AirportId::Grandcity, true) => vec![
            TaxiStep { taxiway: TaxiwayLetter::Charlie, hold_short: false, instruction: "Taxi via Charlie".into() },
            TaxiStep { taxiway: TaxiwayLetter::Delta, hold_short: false, instruction: "Continue Delta".into() },
            TaxiStep { taxiway: TaxiwayLetter::Echo, hold_short: true, instruction: "Hold short Runway 09L on Echo".into() },
        ],
        (_, true) => vec![
            TaxiStep { taxiway: TaxiwayLetter::Alpha, hold_short: false, instruction: "Taxi via Alpha".into() },
            TaxiStep { taxiway: TaxiwayLetter::Bravo, hold_short: true, instruction: "Hold short runway on Bravo".into() },
        ],
        (AirportId::Grandcity, false) => vec![
            TaxiStep { taxiway: TaxiwayLetter::Foxtrot, hold_short: false, instruction: "Exit runway via Foxtrot".into() },
            TaxiStep { taxiway: TaxiwayLetter::Delta, hold_short: false, instruction: "Taxi to gate via Delta".into() },
        ],
        (_, false) => vec![
            TaxiStep { taxiway: TaxiwayLetter::Alpha, hold_short: false, instruction: "Exit runway via Alpha".into() },
            TaxiStep { taxiway: TaxiwayLetter::Bravo, hold_short: false, instruction: "Taxi to ramp via Bravo".into() },
        ],
    };

    let estimated = steps.len() as f32 * 45.0; // ~45 secs per taxiway segment

    TaxiRoute {
        steps,
        estimated_time_secs: estimated,
    }
}

// ─── Pushback ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PushbackState {
    NotNeeded,
    Requested,
    InProgress,
    Complete,
}

/// Ground operations state resource.
#[derive(Resource)]
pub struct GroundOpsState {
    pub pushback: PushbackState,
    pub taxi_route: Option<TaxiRoute>,
    pub current_taxi_step: usize,
    pub taxi_timer: f32,
    pub de_icing_needed: bool,
    pub de_icing_complete: bool,
    pub assigned_gate: Option<String>,
    pub turnaround_remaining_secs: f32,
}

impl Default for GroundOpsState {
    fn default() -> Self {
        Self {
            pushback: PushbackState::NotNeeded,
            taxi_route: None,
            current_taxi_step: 0,
            taxi_timer: 0.0,
            de_icing_needed: false,
            de_icing_complete: false,
            assigned_gate: None,
            turnaround_remaining_secs: 0.0,
        }
    }
}

impl GroundOpsState {
    /// Current taxi instruction text.
    pub fn current_instruction(&self) -> Option<&str> {
        self.taxi_route
            .as_ref()
            .and_then(|r| r.steps.get(self.current_taxi_step))
            .map(|s| s.instruction.as_str())
    }

    pub fn taxi_complete(&self) -> bool {
        self.taxi_route
            .as_ref()
            .is_none_or(|r| self.current_taxi_step >= r.steps.len())
    }

    pub fn turnaround_complete(&self) -> bool {
        self.turnaround_remaining_secs <= 0.0
    }
}

// ─── Turnaround time by airport tier ─────────────────────────────────────

fn turnaround_time_secs(airport: AirportId) -> f32 {
    match airport {
        AirportId::HomeBase => 120.0,
        AirportId::Windport | AirportId::Frostpeak | AirportId::Sunhaven => 180.0,
        AirportId::Ironforge | AirportId::Cloudmere | AirportId::Duskhollow => 240.0,
        AirportId::Stormwatch | AirportId::Grandcity => 300.0,
        AirportId::Skyreach => 360.0,
    }
}

// ─── Gate assignment ─────────────────────────────────────────────────────

fn assign_gate(airport: AirportId) -> String {
    let gate_count = match airport {
        AirportId::HomeBase => 2,
        AirportId::Windport | AirportId::Frostpeak => 4,
        AirportId::Sunhaven | AirportId::Ironforge => 6,
        AirportId::Cloudmere | AirportId::Duskhollow => 5,
        AirportId::Stormwatch => 4,
        AirportId::Grandcity => 20,
        AirportId::Skyreach => 8,
    };
    let gate_num = (rand::random::<u32>() % gate_count) + 1;
    format!("Gate {gate_num}")
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Set up ground ops when arriving at an airport.
pub fn setup_ground_ops_on_arrival(
    mut arrival_events: EventReader<AirportArrivalEvent>,
    weather: Res<WeatherState>,
    calendar: Res<Calendar>,
    mut ground_ops: ResMut<GroundOpsState>,
    mut toast: EventWriter<ToastEvent>,
) {
    for evt in arrival_events.read() {
        let route = build_taxi_route(evt.airport, false);
        let gate = assign_gate(evt.airport);
        let turnaround = turnaround_time_secs(evt.airport);

        let de_ice = calendar.season == Season::Winter
            && matches!(weather.current, Weather::Snow | Weather::Storm);

        toast.send(ToastEvent {
            message: format!(
                "Welcome to {}. Assigned to {}.",
                evt.airport.display_name(),
                gate
            ),
            duration_secs: 3.0,
        });

        if de_ice {
            toast.send(ToastEvent {
                message: "De-icing required — proceed to de-ice pad.".into(),
                duration_secs: 3.0,
            });
        }

        *ground_ops = GroundOpsState {
            pushback: PushbackState::NotNeeded,
            taxi_route: Some(route),
            current_taxi_step: 0,
            taxi_timer: 0.0,
            de_icing_needed: de_ice,
            de_icing_complete: false,
            assigned_gate: Some(gate),
            turnaround_remaining_secs: turnaround,
        };
    }
}

/// Tick turnaround timer while on ground.
pub fn update_turnaround(
    time: Res<Time>,
    mut ground_ops: ResMut<GroundOpsState>,
) {
    if ground_ops.turnaround_remaining_secs > 0.0 {
        ground_ops.turnaround_remaining_secs =
            (ground_ops.turnaround_remaining_secs - time.delta_secs()).max(0.0);
    }
}

/// Advance taxi steps over time (automatic taxi simulation).
pub fn update_taxi_progress(
    time: Res<Time>,
    mut ground_ops: ResMut<GroundOpsState>,
    mut toast: EventWriter<ToastEvent>,
) {
    if ground_ops.taxi_complete() {
        return;
    }

    ground_ops.taxi_timer += time.delta_secs();

    // Each step takes ~45 seconds of game time
    if ground_ops.taxi_timer >= 45.0 {
        ground_ops.taxi_timer = 0.0;

        // Check hold-short before advancing
        if let Some(step) = ground_ops
            .taxi_route
            .as_ref()
            .and_then(|r| r.steps.get(ground_ops.current_taxi_step))
        {
            if step.hold_short {
                toast.send(ToastEvent {
                    message: format!("Hold short on {} — waiting for traffic", step.taxiway.display()),
                    duration_secs: 3.0,
                });
            }
        }

        ground_ops.current_taxi_step += 1;

        if let Some(next) = ground_ops.current_instruction() {
            toast.send(ToastEvent {
                message: next.to_string(),
                duration_secs: 3.0,
            });
        }

        if ground_ops.taxi_complete() {
            toast.send(ToastEvent {
                message: "Taxi complete — at parking position.".into(),
                duration_secs: 2.5,
            });
        }
    }
}

/// Request pushback before departure (player presses interact at gate).
pub fn request_pushback(
    input: Res<PlayerInput>,
    flight_state: Res<FlightState>,
    mut ground_ops: ResMut<GroundOpsState>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !input.interact || flight_state.phase != FlightPhase::Preflight {
        return;
    }
    if ground_ops.pushback != PushbackState::NotNeeded {
        return;
    }

    ground_ops.pushback = PushbackState::Requested;
    toast.send(ToastEvent {
        message: "Pushback requested — ground crew en route.".into(),
        duration_secs: 3.0,
    });
}

/// Simulate pushback progress.
pub fn update_pushback(
    time: Res<Time>,
    mut ground_ops: ResMut<GroundOpsState>,
    mut toast: EventWriter<ToastEvent>,
) {
    match ground_ops.pushback {
        PushbackState::Requested => {
            ground_ops.pushback = PushbackState::InProgress;
            ground_ops.taxi_timer = 0.0;
        }
        PushbackState::InProgress => {
            ground_ops.taxi_timer += time.delta_secs();
            if ground_ops.taxi_timer >= 30.0 {
                ground_ops.pushback = PushbackState::Complete;
                ground_ops.taxi_timer = 0.0;
                toast.send(ToastEvent {
                    message: "Pushback complete — cleared to start engines.".into(),
                    duration_secs: 3.0,
                });
            }
        }
        _ => {}
    }
}
