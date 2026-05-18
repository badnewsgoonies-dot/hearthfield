//! Single-fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug, Default, Clone)]
pub struct Triage;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct BedSlot(pub Vec2);

#[derive(Resource, Debug, Default)]
pub struct PatientLog {
    pub entries: Vec<String>,
}
impl PatientLog {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, dose: u8) -> u8 {
        let mut n: u8 = 0;
        while n < dose && self.entries.pop().is_some() { n += 1; }
        n
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreatmentRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl TreatmentRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct TreatmentRegistry {
    pub defs: HashMap<String, TreatmentRegistryDef>,
}
impl TreatmentRegistry {
    pub fn get(&self, id: &str) -> Option<&TreatmentRegistryDef> {
        self.defs.get(id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreatmentPlan {
    pub medications: Vec<(String, u8)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StackSize(pub u8);
impl StackSize { pub fn get(self) -> u8 { self.0 } }

#[derive(Event, Debug, Clone, Default)]
pub struct PatientAdmittedEvent {
    pub med_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct BedsideAlertEvent { pub sfx_id: String }

#[derive(Event, Debug, Clone, Default)]
pub struct StatusToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

pub fn spawn_arrival_mark(_commands: &mut Commands, _pos: Vec2) {}



/// Reads PatientAdmittedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn admit_patients(
    mut commands: Commands,
    mut pickup_events: EventReader<PatientAdmittedEvent>,
    mut inventory: ResMut<PatientLog>,
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
                spawn_arrival_mark(&mut commands, world_pos);
            }
            info!("[Triage] Picked up {} × '{}'", ev.quantity, ev.med_id);
        } else {
            let name = item_registry
                .get(&ev.med_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.med_id);
            toast_events.send(StatusToastEvent {
                message: format!("PatientLog full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Triage] PatientLog full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.med_id, remaining
            );
        }
    }
}


