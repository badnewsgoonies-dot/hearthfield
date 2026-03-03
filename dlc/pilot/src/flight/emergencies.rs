//! Emergency event system — random in-flight emergencies with checklists.

use bevy::prelude::*;
use crate::shared::*;

pub struct EmergencyPlugin;

impl Plugin for EmergencyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EmergencyState>()
            .add_systems(
                Update,
                (
                    trigger_random_emergency,
                    handle_emergency,
                    resolve_emergency,
                    update_emergency_timers,
                )
                    .run_if(in_state(GameState::Flying)),
            );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmergencyType {
    EngineFailure,
    FuelLeak,
    HydraulicFailure,
    BirdStrike,
    LightningStrike,
    Depressurization,
    InstrumentFailure,
    IceAccumulation,
}

impl EmergencyType {
    pub fn display_name(&self) -> &'static str {
        match self {
            EmergencyType::EngineFailure => "Engine Failure",
            EmergencyType::FuelLeak => "Fuel Leak",
            EmergencyType::HydraulicFailure => "Hydraulic Failure",
            EmergencyType::BirdStrike => "Bird Strike",
            EmergencyType::LightningStrike => "Lightning Strike",
            EmergencyType::Depressurization => "Cabin Depressurization",
            EmergencyType::InstrumentFailure => "Instrument Failure",
            EmergencyType::IceAccumulation => "Ice Accumulation",
        }
    }

    pub fn severity(&self) -> EmergencySeverity {
        match self {
            EmergencyType::EngineFailure => EmergencySeverity::Critical,
            EmergencyType::FuelLeak => EmergencySeverity::Major,
            EmergencyType::HydraulicFailure => EmergencySeverity::Major,
            EmergencyType::BirdStrike => EmergencySeverity::Moderate,
            EmergencyType::LightningStrike => EmergencySeverity::Moderate,
            EmergencyType::Depressurization => EmergencySeverity::Critical,
            EmergencyType::InstrumentFailure => EmergencySeverity::Minor,
            EmergencyType::IceAccumulation => EmergencySeverity::Major,
        }
    }

    pub fn checklist_actions(&self) -> Vec<&'static str> {
        match self {
            EmergencyType::EngineFailure => vec![
                "Reduce throttle to idle",
                "Switch to backup engine",
                "Declare emergency to ATC",
                "Prepare for emergency landing",
            ],
            EmergencyType::FuelLeak => vec![
                "Close fuel crossfeed valve",
                "Monitor fuel gauge",
                "Divert to nearest airport",
            ],
            EmergencyType::HydraulicFailure => vec![
                "Switch to manual controls",
                "Reduce airspeed",
                "Prepare for manual gear extension",
            ],
            EmergencyType::BirdStrike => vec![
                "Check engine readings",
                "Reduce speed",
                "Inspect for damage",
            ],
            EmergencyType::LightningStrike => vec![
                "Check all instruments",
                "Reset circuit breakers",
                "Report to ATC",
            ],
            EmergencyType::Depressurization => vec![
                "Don oxygen masks",
                "Emergency descent to 10000ft",
                "Declare emergency",
                "Divert to nearest airport",
            ],
            EmergencyType::InstrumentFailure => vec![
                "Switch to standby instruments",
                "Cross-check readings",
                "Report partial panel",
            ],
            EmergencyType::IceAccumulation => vec![
                "Activate de-icing system",
                "Increase airspeed slightly",
                "Change altitude if possible",
            ],
        }
    }

    fn from_index(i: usize) -> Self {
        match i % 8 {
            0 => EmergencyType::EngineFailure,
            1 => EmergencyType::FuelLeak,
            2 => EmergencyType::HydraulicFailure,
            3 => EmergencyType::BirdStrike,
            4 => EmergencyType::LightningStrike,
            5 => EmergencyType::Depressurization,
            6 => EmergencyType::InstrumentFailure,
            _ => EmergencyType::IceAccumulation,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EmergencySeverity {
    Minor,
    Moderate,
    Major,
    Critical,
}

impl EmergencySeverity {
    pub fn xp_bonus(&self) -> u32 {
        match self {
            EmergencySeverity::Minor => 15,
            EmergencySeverity::Moderate => 30,
            EmergencySeverity::Major => 50,
            EmergencySeverity::Critical => 100,
        }
    }

    pub fn throttle_penalty(&self) -> f32 {
        match self {
            EmergencySeverity::Minor => 0.0,
            EmergencySeverity::Moderate => 0.1,
            EmergencySeverity::Major => 0.25,
            EmergencySeverity::Critical => 0.5,
        }
    }
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct EmergencyState {
    pub active_emergency: Option<ActiveEmergency>,
    pub cooldown_secs: f32,
    pub emergencies_handled: u32,
    pub emergencies_failed: u32,
    trigger_accumulator: f32,
}

pub struct ActiveEmergency {
    pub emergency_type: EmergencyType,
    pub checklist_progress: usize,
    pub checklist_total: usize,
    pub time_remaining: f32,
    pub resolved: bool,
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn trigger_random_emergency(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    fleet: Res<Fleet>,
    mut emergency_state: ResMut<EmergencyState>,
    mut emergency_events: EventWriter<EmergencyEvent>,
) {
    if emergency_state.active_emergency.is_some() || emergency_state.cooldown_secs > 0.0 {
        return;
    }
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent
    ) {
        return;
    }

    let dt = time.delta_secs();
    let weather_factor = 1.0 + weather.current.flight_difficulty() * 2.0;
    let condition_factor = if let Some(ac) = fleet.active() {
        2.0 - (ac.condition / 100.0).max(0.01)
    } else {
        1.0
    };

    // Base chance ~0.2% per second, scaled by weather and aircraft condition
    let chance_per_sec = 0.002 * weather_factor * condition_factor;
    emergency_state.trigger_accumulator += chance_per_sec * dt;

    if emergency_state.trigger_accumulator >= 1.0 {
        emergency_state.trigger_accumulator = 0.0;

        let tick = (time.elapsed_secs() * 1000.0) as usize;
        let etype = EmergencyType::from_index(tick);

        // Weather-correlated emergencies
        let etype = match weather.current {
            Weather::Storm => EmergencyType::LightningStrike,
            Weather::Snow => EmergencyType::IceAccumulation,
            _ => etype,
        };

        let kind = match etype {
            EmergencyType::EngineFailure => EmergencyKind::EngineFailure,
            EmergencyType::FuelLeak => EmergencyKind::FuelLeak,
            EmergencyType::HydraulicFailure => EmergencyKind::HydraulicFailure,
            EmergencyType::BirdStrike => EmergencyKind::BirdStrike,
            EmergencyType::LightningStrike => EmergencyKind::LightningStrike,
            _ => EmergencyKind::EngineFailure,
        };

        emergency_events.send(EmergencyEvent { kind });

        let actions = etype.checklist_actions();
        let total = actions.len();
        let time_limit = match etype.severity() {
            EmergencySeverity::Minor => 60.0,
            EmergencySeverity::Moderate => 45.0,
            EmergencySeverity::Major => 30.0,
            EmergencySeverity::Critical => 20.0,
        };

        emergency_state.active_emergency = Some(ActiveEmergency {
            emergency_type: etype,
            checklist_progress: 0,
            checklist_total: total,
            time_remaining: time_limit,
            resolved: false,
        });
    }
}

pub fn handle_emergency(
    mut events: EventReader<EmergencyEvent>,
    mut flight_state: ResMut<FlightState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in events.read() {
        toast_events.send(ToastEvent {
            message: format!("⚠ EMERGENCY: {:?}! Complete checklist!", ev.kind),
            duration_secs: 5.0,
        });

        flight_state.phase = FlightPhase::Emergency;

        match ev.kind {
            EmergencyKind::EngineFailure => {
                flight_state.throttle = (flight_state.throttle * 0.3).min(0.3);
            }
            EmergencyKind::FuelLeak => {
                flight_state.fuel_remaining *= 0.9;
            }
            EmergencyKind::HydraulicFailure => {
                flight_state.flaps_deployed = false;
            }
            EmergencyKind::BirdStrike => {
                flight_state.speed_knots *= 0.85;
            }
            EmergencyKind::LightningStrike => {
                flight_state.autopilot = false;
            }
        }

        flight_state.passengers_happy = (flight_state.passengers_happy - 20.0).max(0.0);
    }
}

pub fn resolve_emergency(
    input: Res<PlayerInput>,
    mut emergency_state: ResMut<EmergencyState>,
    mut flight_state: ResMut<FlightState>,
    mut pilot_state: ResMut<PilotState>,
    mut toast_events: EventWriter<ToastEvent>,
    mut xp_events: EventWriter<XpGainEvent>,
) {
    let resolved_info = {
        let Some(ref mut active) = emergency_state.active_emergency else {
            return;
        };
        if active.resolved {
            return;
        }

        // Player presses confirm to advance checklist
        if input.confirm && active.checklist_progress < active.checklist_total {
            active.checklist_progress += 1;
            let actions = active.emergency_type.checklist_actions();
            if active.checklist_progress <= actions.len() {
                let action = actions[active.checklist_progress - 1];
                toast_events.send(ToastEvent {
                    message: format!("✓ {action}"),
                    duration_secs: 2.0,
                });
            }
        }

        if active.checklist_progress >= active.checklist_total {
            active.resolved = true;
            let severity = active.emergency_type.severity();
            let xp = severity.xp_bonus();
            let display = active.emergency_type.display_name().to_string();
            Some((xp, display))
        } else {
            None
        }
    };

    if let Some((xp, display)) = resolved_info {
        flight_state.phase = FlightPhase::Cruise;
        emergency_state.emergencies_handled += 1;
        emergency_state.cooldown_secs = 120.0;
        pilot_state.xp += xp;

        xp_events.send(XpGainEvent {
            amount: xp,
            source: format!("Resolved: {display}"),
        });
        toast_events.send(ToastEvent {
            message: format!("Emergency resolved! +{xp} XP"),
            duration_secs: 4.0,
        });
    }
}

pub fn update_emergency_timers(
    time: Res<Time>,
    mut emergency_state: ResMut<EmergencyState>,
    mut flight_state: ResMut<FlightState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let dt = time.delta_secs();

    if emergency_state.cooldown_secs > 0.0 {
        emergency_state.cooldown_secs = (emergency_state.cooldown_secs - dt).max(0.0);
    }

    // Handle timeout on active emergency
    let timed_out = if let Some(ref mut active) = emergency_state.active_emergency {
        if !active.resolved {
            active.time_remaining -= dt;
            if active.time_remaining <= 0.0 {
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    if timed_out {
        emergency_state.emergencies_failed += 1;
        emergency_state.cooldown_secs = 60.0;
        emergency_state.active_emergency = None;
        flight_state.phase = FlightPhase::Descent;
        toast_events.send(ToastEvent {
            message: "Emergency unresolved! Forced descent.".to_string(),
            duration_secs: 5.0,
        });
        return;
    }

    // Clean up resolved emergencies after a brief delay
    let should_clear = emergency_state
        .active_emergency
        .as_ref()
        .map_or(false, |e| e.resolved);
    if should_clear {
        emergency_state.active_emergency = None;
    }
}
