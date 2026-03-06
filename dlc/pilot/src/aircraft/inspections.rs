//! Pre-flight inspection system — walk-around checks before takeoff.

use crate::shared::*;
use bevy::prelude::*;

pub struct InspectionPlugin;

impl Plugin for InspectionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectionState>().add_systems(
            Update,
            (perform_inspection, evaluate_inspection_result).run_if(in_state(GameState::Playing)),
        );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct InspectionItem {
    pub name: &'static str,
    pub description: &'static str,
    pub checked: bool,
    pub condition_impact: f32,
}

impl InspectionItem {
    pub fn new(name: &'static str, desc: &'static str, impact: f32) -> Self {
        Self {
            name,
            description: desc,
            checked: false,
            condition_impact: impact,
        }
    }
}

#[derive(Resource)]
pub struct InspectionState {
    pub checklist: Vec<InspectionItem>,
    pub active: bool,
    pub current_item: usize,
    pub inspection_completed: bool,
    pub thoroughness: f32,
    pub emergency_chance_modifier: f32,
}

impl Default for InspectionState {
    fn default() -> Self {
        Self {
            checklist: build_default_checklist(),
            active: false,
            current_item: 0,
            inspection_completed: false,
            thoroughness: 0.0,
            emergency_chance_modifier: 1.0,
        }
    }
}

impl InspectionState {
    pub fn reset(&mut self) {
        self.checklist = build_default_checklist();
        self.active = false;
        self.current_item = 0;
        self.inspection_completed = false;
        self.thoroughness = 0.0;
        self.emergency_chance_modifier = 1.0;
    }

    pub fn items_checked(&self) -> usize {
        self.checklist.iter().filter(|i| i.checked).count()
    }

    pub fn total_items(&self) -> usize {
        self.checklist.len()
    }

    pub fn completion_ratio(&self) -> f32 {
        if self.checklist.is_empty() {
            return 0.0;
        }
        self.items_checked() as f32 / self.total_items() as f32
    }
}

fn build_default_checklist() -> Vec<InspectionItem> {
    vec![
        InspectionItem::new("Tires", "Check tire pressure and wear", 5.0),
        InspectionItem::new("Engine Oil", "Verify oil level and quality", 10.0),
        InspectionItem::new("Fuel Level", "Confirm fuel quantity matches plan", 8.0),
        InspectionItem::new("Control Surfaces", "Move ailerons, elevators, rudder", 12.0),
        InspectionItem::new("Lights", "Test nav, strobe, and landing lights", 3.0),
        InspectionItem::new("Instruments", "Power up and verify readings", 7.0),
        InspectionItem::new(
            "Emergency Equipment",
            "Check fire extinguisher and first aid",
            5.0,
        ),
    ]
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn perform_inspection(
    input: Res<PlayerInput>,
    player_location: Res<PlayerLocation>,
    mut inspection: ResMut<InspectionState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if player_location.zone != MapZone::Hangar {
        return;
    }

    // Start inspection with interact key when near aircraft
    if input.interact && !inspection.active && !inspection.inspection_completed {
        inspection.active = true;
        inspection.current_item = 0;
        toast_events.send(ToastEvent {
            message: "Starting pre-flight inspection...".to_string(),
            duration_secs: 3.0,
        });
        return;
    }

    if !inspection.active {
        return;
    }

    // Advance through checklist items
    if input.confirm {
        let idx = inspection.current_item;
        if idx < inspection.checklist.len() {
            inspection.checklist[idx].checked = true;
            let item_name = inspection.checklist[idx].name;
            toast_events.send(ToastEvent {
                message: format!("✓ {} — OK", item_name),
                duration_secs: 2.0,
            });
            inspection.current_item += 1;
        }

        if inspection.current_item >= inspection.checklist.len() {
            inspection.active = false;
            inspection.inspection_completed = true;
            toast_events.send(ToastEvent {
                message: "Pre-flight inspection complete!".to_string(),
                duration_secs: 3.0,
            });
        }
    }

    // Skip remaining with cancel
    if input.cancel && inspection.active {
        inspection.active = false;
        inspection.inspection_completed = true;
        toast_events.send(ToastEvent {
            message: "⚠ Inspection incomplete — increased emergency risk!".to_string(),
            duration_secs: 4.0,
        });
    }
}

pub fn evaluate_inspection_result(
    mut inspection: ResMut<InspectionState>,
    mut flight_events: EventReader<FlightStartEvent>,
) {
    for _ev in flight_events.read() {
        let ratio = inspection.completion_ratio();
        inspection.thoroughness = ratio;

        // Skipping inspection increases emergency chance
        inspection.emergency_chance_modifier = if ratio >= 1.0 {
            0.5 // Thorough inspection halves emergency chance
        } else if ratio >= 0.5 {
            1.0 // Partial inspection — normal chance
        } else {
            2.0 // Skipped — doubled emergency chance
        };

        inspection.reset();
    }
}
