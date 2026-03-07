//! Detailed flight checklists for each phase of flight.
//!
//! Each flight goes through multiple phases, each with a set of items to verify.
//! The player presses F to check the next item; skipping items increases risk.

use crate::shared::*;
use bevy::prelude::*;

// ─── Checklist Phase ─────────────────────────────────────────────────────

/// All distinct checklist phases during a flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChecklistPhase {
    PreStart,
    EngineStart,
    BeforeTaxi,
    BeforeTakeoff,
    AfterTakeoff,
    Cruise,
    Descent,
    BeforeLanding,
    AfterLanding,
    Shutdown,
}

impl ChecklistPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::PreStart => "Pre-Start",
            Self::EngineStart => "Engine Start",
            Self::BeforeTaxi => "Before Taxi",
            Self::BeforeTakeoff => "Before Takeoff",
            Self::AfterTakeoff => "After Takeoff",
            Self::Cruise => "Cruise",
            Self::Descent => "Descent",
            Self::BeforeLanding => "Before Landing",
            Self::AfterLanding => "After Landing",
            Self::Shutdown => "Shutdown",
        }
    }

    pub fn next(&self) -> Option<ChecklistPhase> {
        match self {
            Self::PreStart => Some(Self::EngineStart),
            Self::EngineStart => Some(Self::BeforeTaxi),
            Self::BeforeTaxi => Some(Self::BeforeTakeoff),
            Self::BeforeTakeoff => Some(Self::AfterTakeoff),
            Self::AfterTakeoff => Some(Self::Cruise),
            Self::Cruise => Some(Self::Descent),
            Self::Descent => Some(Self::BeforeLanding),
            Self::BeforeLanding => Some(Self::AfterLanding),
            Self::AfterLanding => Some(Self::Shutdown),
            Self::Shutdown => None,
        }
    }

    pub fn ordered() -> &'static [ChecklistPhase] {
        &[
            Self::PreStart,
            Self::EngineStart,
            Self::BeforeTaxi,
            Self::BeforeTakeoff,
            Self::AfterTakeoff,
            Self::Cruise,
            Self::Descent,
            Self::BeforeLanding,
            Self::AfterLanding,
            Self::Shutdown,
        ]
    }
}

// ─── Item Status ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ChecklistItemStatus {
    #[default]
    Unchecked,
    Checked,
    Failed,
}

// ─── Checklist Item ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct PhaseChecklistItem {
    pub label: &'static str,
    pub description: &'static str,
    pub status: ChecklistItemStatus,
}

impl PhaseChecklistItem {
    pub const fn new(label: &'static str, description: &'static str) -> Self {
        Self {
            label,
            description,
            status: ChecklistItemStatus::Unchecked,
        }
    }
}

// ─── Aircraft Type for checklist selection ────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ChecklistAircraftType {
    Piston,
    Jet,
}

impl ChecklistAircraftType {
    pub fn from_class(class: AircraftClass) -> Self {
        match class {
            AircraftClass::SingleProp | AircraftClass::TwinProp | AircraftClass::Seaplane => {
                Self::Piston
            }
            AircraftClass::Turboprop
            | AircraftClass::LightJet
            | AircraftClass::MediumJet
            | AircraftClass::HeavyJet
            | AircraftClass::Cargo => Self::Jet,
        }
    }
}

// ─── Phase-specific item lists ───────────────────────────────────────────

fn pre_start_items(ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    let mut items = vec![
        PhaseChecklistItem::new("walk_around", "Perform external walk-around inspection"),
        PhaseChecklistItem::new("oil_level", "Check engine oil level"),
        PhaseChecklistItem::new("fuel_quantity", "Verify fuel quantity matches plan"),
        PhaseChecklistItem::new("tire_condition", "Inspect tires for wear and pressure"),
        PhaseChecklistItem::new("control_surfaces", "Verify ailerons, elevator, rudder free"),
        PhaseChecklistItem::new("documents", "Confirm aircraft documents on board"),
    ];
    match ac {
        ChecklistAircraftType::Piston => {
            items.push(PhaseChecklistItem::new(
                "propeller",
                "Check propeller for nicks",
            ));
        }
        ChecklistAircraftType::Jet => {
            items.push(PhaseChecklistItem::new(
                "intake_check",
                "Inspect engine intakes clear of FOD",
            ));
            items.push(PhaseChecklistItem::new(
                "apu_battery",
                "Verify APU battery charge",
            ));
        }
    }
    items
}

fn engine_start_items(ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    let mut items = vec![
        PhaseChecklistItem::new("battery_on", "Master switch / battery ON"),
        PhaseChecklistItem::new("fuel_selector", "Fuel selector set to correct tank"),
        PhaseChecklistItem::new("beacon_on", "Anti-collision beacon ON"),
        PhaseChecklistItem::new("area_clear", "Ensure propeller / intake area clear"),
    ];
    match ac {
        ChecklistAircraftType::Piston => {
            items.push(PhaseChecklistItem::new(
                "primer",
                "Prime engine 2–4 strokes",
            ));
            items.push(PhaseChecklistItem::new("magnetos", "Magnetos set to BOTH"));
            items.push(PhaseChecklistItem::new(
                "starter_engage",
                "Engage starter, watch RPM",
            ));
        }
        ChecklistAircraftType::Jet => {
            items.push(PhaseChecklistItem::new(
                "apu_start",
                "Start APU, wait for stable",
            ));
            items.push(PhaseChecklistItem::new("fuel_pumps", "Fuel boost pumps ON"));
            items.push(PhaseChecklistItem::new(
                "n1_rotation",
                "Advance throttle, verify N1 rotation",
            ));
            items.push(PhaseChecklistItem::new(
                "egt_check",
                "Monitor EGT within limits",
            ));
        }
    }
    items
}

fn before_taxi_items(_ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    vec![
        PhaseChecklistItem::new("avionics_on", "Avionics master ON"),
        PhaseChecklistItem::new("radios_set", "Set tower and ground frequencies"),
        PhaseChecklistItem::new("altimeter_set", "Set altimeter to field elevation"),
        PhaseChecklistItem::new("heading_indicator", "Align heading indicator with compass"),
        PhaseChecklistItem::new("taxi_clearance", "Request taxi clearance from ground"),
        PhaseChecklistItem::new("brakes_check", "Test brakes during initial roll"),
    ]
}

fn before_takeoff_items(ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    let mut items = vec![
        PhaseChecklistItem::new("flaps_set", "Set flaps for takeoff"),
        PhaseChecklistItem::new("trim_set", "Set elevator trim for takeoff"),
        PhaseChecklistItem::new("instruments_checked", "Verify engine instruments green"),
        PhaseChecklistItem::new("doors_secured", "All doors and hatches secure"),
        PhaseChecklistItem::new("transponder_on", "Set transponder to ALT mode"),
        PhaseChecklistItem::new("lights_on", "Landing and strobe lights ON"),
        PhaseChecklistItem::new("briefing_complete", "Takeoff briefing complete"),
    ];
    match ac {
        ChecklistAircraftType::Piston => {
            items.push(PhaseChecklistItem::new(
                "mixture_rich",
                "Mixture set to RICH",
            ));
            items.push(PhaseChecklistItem::new(
                "carb_heat_off",
                "Carburetor heat OFF",
            ));
        }
        ChecklistAircraftType::Jet => {
            items.push(PhaseChecklistItem::new(
                "takeoff_config",
                "Takeoff config warning test",
            ));
            items.push(PhaseChecklistItem::new("anti_ice", "Anti-ice as required"));
            items.push(PhaseChecklistItem::new(
                "speed_bugs",
                "Set V-speed bugs on airspeed indicator",
            ));
        }
    }
    items
}

fn after_takeoff_items(_ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    vec![
        PhaseChecklistItem::new("gear_up", "Retract landing gear (if retractable)"),
        PhaseChecklistItem::new("flaps_retract", "Retract flaps on schedule"),
        PhaseChecklistItem::new("power_set", "Set climb power"),
        PhaseChecklistItem::new("heading_set", "Turn to assigned departure heading"),
        PhaseChecklistItem::new("altitude_climbing", "Verify positive rate of climb"),
    ]
}

fn cruise_items(_ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    vec![
        PhaseChecklistItem::new("level_off", "Level off at assigned altitude"),
        PhaseChecklistItem::new("power_cruise", "Set cruise power / thrust"),
        PhaseChecklistItem::new("fuel_check", "Check fuel quantity and balance"),
        PhaseChecklistItem::new("nav_check", "Verify navigation track"),
        PhaseChecklistItem::new("cabin_check", "Cabin altitude and pressurisation OK"),
    ]
}

fn descent_items(_ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    vec![
        PhaseChecklistItem::new("atis_get", "Obtain destination ATIS"),
        PhaseChecklistItem::new("altimeter_reset", "Set destination altimeter setting"),
        PhaseChecklistItem::new("approach_brief", "Brief the approach procedure"),
        PhaseChecklistItem::new("speed_reduce", "Reduce speed for descent"),
        PhaseChecklistItem::new("seatbelt_sign", "Seatbelt sign ON"),
        PhaseChecklistItem::new("descent_clearance", "Obtain descent clearance"),
    ]
}

fn before_landing_items(ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    let mut items = vec![
        PhaseChecklistItem::new("gear_down", "Landing gear DOWN and locked"),
        PhaseChecklistItem::new("flaps_landing", "Set landing flaps"),
        PhaseChecklistItem::new("speed_check", "Verify approach speed"),
        PhaseChecklistItem::new("runway_insight", "Runway in sight or approach stable"),
        PhaseChecklistItem::new("landing_clearance", "Receive landing clearance"),
        PhaseChecklistItem::new("go_around_brief", "Brief go-around procedure"),
    ];
    if ac == ChecklistAircraftType::Piston {
        items.push(PhaseChecklistItem::new(
            "mixture_rich_land",
            "Mixture RICH for landing",
        ));
    }
    items
}

fn after_landing_items(_ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    vec![
        PhaseChecklistItem::new("flaps_retract_land", "Retract flaps"),
        PhaseChecklistItem::new("transponder_stby", "Transponder to STANDBY"),
        PhaseChecklistItem::new("landing_lights_off", "Landing lights OFF"),
        PhaseChecklistItem::new("taxi_to_gate", "Taxi to assigned gate/ramp"),
        PhaseChecklistItem::new("ground_contact", "Contact ground for taxi instructions"),
    ]
}

fn shutdown_items(ac: ChecklistAircraftType) -> Vec<PhaseChecklistItem> {
    let mut items = vec![
        PhaseChecklistItem::new("parking_brake", "Set parking brake"),
        PhaseChecklistItem::new("avionics_off", "Avionics master OFF"),
        PhaseChecklistItem::new("lights_off", "All exterior lights OFF"),
        PhaseChecklistItem::new("engine_shutdown", "Engine(s) shut down"),
        PhaseChecklistItem::new("master_off", "Master switch / battery OFF"),
        PhaseChecklistItem::new("logbook_entry", "Complete flight logbook entry"),
    ];
    if ac == ChecklistAircraftType::Jet {
        items.push(PhaseChecklistItem::new(
            "apu_shutdown",
            "APU shut down after ground power connected",
        ));
    }
    items
}

// ─── Full checklist builder ──────────────────────────────────────────────

pub fn build_checklist(
    ac: ChecklistAircraftType,
) -> Vec<(ChecklistPhase, Vec<PhaseChecklistItem>)> {
    vec![
        (ChecklistPhase::PreStart, pre_start_items(ac)),
        (ChecklistPhase::EngineStart, engine_start_items(ac)),
        (ChecklistPhase::BeforeTaxi, before_taxi_items(ac)),
        (ChecklistPhase::BeforeTakeoff, before_takeoff_items(ac)),
        (ChecklistPhase::AfterTakeoff, after_takeoff_items(ac)),
        (ChecklistPhase::Cruise, cruise_items(ac)),
        (ChecklistPhase::Descent, descent_items(ac)),
        (ChecklistPhase::BeforeLanding, before_landing_items(ac)),
        (ChecklistPhase::AfterLanding, after_landing_items(ac)),
        (ChecklistPhase::Shutdown, shutdown_items(ac)),
    ]
}

// ─── Runtime checklist resource ──────────────────────────────────────────

#[derive(Resource)]
pub struct ActiveChecklist {
    pub aircraft_type: ChecklistAircraftType,
    pub phases: Vec<(ChecklistPhase, Vec<PhaseChecklistItem>)>,
    pub current_phase_index: usize,
    pub current_item_index: usize,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub completed: bool,
}

impl Default for ActiveChecklist {
    fn default() -> Self {
        let phases = build_checklist(ChecklistAircraftType::Piston);
        Self {
            aircraft_type: ChecklistAircraftType::Piston,
            phases,
            current_phase_index: 0,
            current_item_index: 0,
            skipped_count: 0,
            failed_count: 0,
            completed: false,
        }
    }
}

impl ActiveChecklist {
    pub fn new(ac: ChecklistAircraftType) -> Self {
        Self {
            aircraft_type: ac,
            phases: build_checklist(ac),
            current_phase_index: 0,
            current_item_index: 0,
            skipped_count: 0,
            failed_count: 0,
            completed: false,
        }
    }

    pub fn current_phase(&self) -> Option<&ChecklistPhase> {
        self.phases.get(self.current_phase_index).map(|(p, _)| p)
    }

    pub fn current_item(&self) -> Option<&PhaseChecklistItem> {
        self.phases
            .get(self.current_phase_index)
            .and_then(|(_, items)| items.get(self.current_item_index))
    }

    pub fn current_item_mut(&mut self) -> Option<&mut PhaseChecklistItem> {
        self.phases
            .get_mut(self.current_phase_index)
            .and_then(|(_, items)| items.get_mut(self.current_item_index))
    }

    pub fn total_items(&self) -> usize {
        self.phases.iter().map(|(_, items)| items.len()).sum()
    }

    pub fn checked_count(&self) -> usize {
        self.phases
            .iter()
            .flat_map(|(_, items)| items)
            .filter(|i| i.status == ChecklistItemStatus::Checked)
            .count()
    }

    /// Risk factor based on skipped/failed items (0.0 = safe, 1.0 = very risky).
    pub fn risk_factor(&self) -> f32 {
        let total = self.total_items().max(1) as f32;
        ((self.skipped_count as f32 * 0.1) + (self.failed_count as f32 * 0.2)) / total
    }

    /// Advance to next item. Returns `true` if checklist fully complete.
    fn advance(&mut self) -> bool {
        let phase_len = self
            .phases
            .get(self.current_phase_index)
            .map(|(_, items)| items.len())
            .unwrap_or(0);

        self.current_item_index += 1;
        if self.current_item_index >= phase_len {
            self.current_item_index = 0;
            self.current_phase_index += 1;
            if self.current_phase_index >= self.phases.len() {
                self.completed = true;
                return true;
            }
        }
        false
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Player presses F to check the next item in the active checklist.
pub fn advance_checklist(
    input: Res<PlayerInput>,
    mut checklist: ResMut<ActiveChecklist>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !input.interact || checklist.completed {
        return;
    }

    let phase_name = checklist
        .current_phase()
        .map(|p| p.display_name())
        .unwrap_or("Unknown");
    let label = checklist.current_item().map(|i| i.label).unwrap_or("?");

    if let Some(item) = checklist.current_item_mut() {
        item.status = ChecklistItemStatus::Checked;
    }

    toast.send(ToastEvent {
        message: format!("[{}] ✓ {}", phase_name, label),
        duration_secs: 2.0,
    });

    let done = checklist.advance();
    if done {
        toast.send(ToastEvent {
            message: "All checklists complete — ready for flight!".into(),
            duration_secs: 3.0,
        });
    } else if let Some(new_phase) = checklist.current_phase() {
        // Announce when we move to a new phase
        let prev_idx = if checklist.current_item_index == 0 && checklist.current_phase_index > 0 {
            checklist.current_phase_index - 1
        } else {
            checklist.current_phase_index
        };
        if checklist.current_item_index == 0 && checklist.current_phase_index != prev_idx + 1 {
            // same phase, no announcement
        } else if checklist.current_item_index == 0 {
            toast.send(ToastEvent {
                message: format!("Starting {} checklist", new_phase.display_name()),
                duration_secs: 2.5,
            });
        }
    }
}

/// Skip the current item — increases risk factor.
pub fn skip_checklist_item(
    input: Res<PlayerInput>,
    mut checklist: ResMut<ActiveChecklist>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !input.cancel || checklist.completed {
        return;
    }

    let label = checklist.current_item().map(|i| i.label).unwrap_or("?");

    if let Some(item) = checklist.current_item_mut() {
        item.status = ChecklistItemStatus::Failed;
    }
    checklist.skipped_count += 1;

    toast.send(ToastEvent {
        message: format!("⚠ Skipped: {} (risk increased)", label),
        duration_secs: 2.5,
    });

    checklist.advance();
}

/// Reset checklist when entering a new flight.
pub fn reset_checklist_on_flight_start(
    fleet: Res<Fleet>,
    aircraft_registry: Res<AircraftRegistry>,
    mut checklist: ResMut<ActiveChecklist>,
) {
    let ac_type = fleet
        .active()
        .and_then(|owned| aircraft_registry.get(&owned.aircraft_id))
        .map(|def| ChecklistAircraftType::from_class(def.class))
        .unwrap_or(ChecklistAircraftType::Piston);

    *checklist = ActiveChecklist::new(ac_type);
}
