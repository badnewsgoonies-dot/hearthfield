//! Lifeline::domains::patients — substrate-mechanical port via parser_v3.
//! No LLM authored this. Bodies pulled from hearthfield, types renamed
//! through a substitution map, cargo-driven stub generation closes gaps.

use bevy::prelude::*;
use std::collections::HashMap;

// ── components ────────────────────────────────────────────────────────

#[derive(Component, Debug, Default)]
pub struct PatientSprite;

#[derive(Component, Debug, Default)]
pub struct Triage;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct BedSlot(pub Vec2);

// ── resources ─────────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct PatientQueue {
    pub queued: Vec<String>,
}

impl PatientQueue {
    pub fn try_remove(&mut self, med_id: &str, dose: u8) -> u8 {
        let _ = (med_id, dose);
        if self.queued.pop().is_some() { 1 } else { 0 }
    }
    pub fn try_add(&mut self, med_id: &str, qty: u8, _max: u8) -> u8 {
        let _ = qty;
        self.queued.push(med_id.to_string());
        0
    }
}

#[derive(Debug, Clone)]
pub struct TreatmentPlan {
    pub medications: Vec<(String, u8)>,
}

#[derive(Debug, Clone, Default)]
pub struct TreatmentDef {
    pub name: String,
    pub stack_size: TreatmentStackSize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TreatmentStackSize(pub u8);
impl TreatmentStackSize {
    pub fn get(self) -> u8 { self.0 }
}

#[derive(Resource, Default)]
pub struct TreatmentRegistry {
    pub plans: HashMap<String, TreatmentPlan>,
    pub defs: HashMap<String, TreatmentDef>,
}

impl TreatmentRegistry {
    pub fn get(&self, id: &str) -> Option<&TreatmentDef> {
        self.defs.get(id)
    }
}

// ── events ────────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone)]
pub struct PatientArrivedEvent {
    pub med_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone)]
pub struct BedsideAlertEvent {
    pub sfx_id: String,
}

#[derive(Event, Debug, Clone)]
pub struct StatusToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

pub fn spawn_admit_indicator(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────────

/// Consume all non-wildcard medications.
pub fn dispense_treatment(inventory: &mut PatientQueue, recipe: &TreatmentPlan) {
    for (med_id, dose) in &recipe.medications {
        if med_id == "placeholder_med" {
            continue;
        }
        let dispensed = inventory.try_remove(med_id, *dose);
        if dispensed < *dose {
            warn!(
                "dispense_treatment: only dispensed {} of {} '{}'",
                dispensed, dose, med_id
            );
        }
    }
}



/// Refund all non-wildcard medications.
pub fn restock_treatment(
    inventory: &mut PatientQueue,
    recipe: &TreatmentPlan,
    registry: &TreatmentRegistry,
) {
    for (med_id, dose) in &recipe.medications {
        if med_id == "placeholder_med" {
            continue;
        }
        let max_stack = registry.get(med_id).map(|d| d.stack_size.get()).unwrap_or(99);
        inventory.try_add(med_id, *dose, max_stack);
    }
}


/// Reads PatientArrivedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn admit_patients(
    mut commands: Commands,
    mut pickup_events: EventReader<PatientArrivedEvent>,
    mut inventory: ResMut<PatientQueue>,
    item_registry: Res<TreatmentRegistry>,
    mut sfx_events: EventWriter<BedsideAlertEvent>,
    mut toast_events: EventWriter<StatusToastEvent>,
    player_query: Query<&BedSlot, With<Triage>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.med_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.med_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(BedsideAlertEvent {
                sfx_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_admit_indicator(&mut commands, world_pos);
            }
            info!("[Triage] Picked up {} × '{}'", ev.quantity, ev.med_id);
        } else {
            let name = item_registry
                .get(&ev.med_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.med_id);
            toast_events.send(StatusToastEvent {
                message: format!("PatientQueue full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Triage] PatientQueue full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.med_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PatientQueue>()
            .init_resource::<TreatmentRegistry>()
            .add_event::<PatientArrivedEvent>()
            .add_event::<BedsideAlertEvent>()
            .add_event::<StatusToastEvent>()
            .add_systems(Update, admit_patients);
    }
}
