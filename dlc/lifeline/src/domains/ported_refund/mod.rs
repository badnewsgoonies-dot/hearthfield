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



/// Refund all non-wildcard medications.
pub fn restock_treatment(
    inventory: &mut PatientLog,
    recipe: &TreatmentPlan,
    registry: &TreatmentRegistry,
) {
    for (med_id, dose) in &recipe.medications {
        if med_id == "placebo" {
            continue;
        }
        let max_stack = registry.get(med_id).map(|d| d.stack_size.get()).unwrap_or(99);
        inventory.try_add(med_id, *dose, max_stack);
    }
}

