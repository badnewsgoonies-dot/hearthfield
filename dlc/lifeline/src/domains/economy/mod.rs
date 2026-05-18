//! Lifeline::domains::economy — substrate-mechanical port.
//! No LLM. parser_v3 + substitution map + cargo gate.

use bevy::prelude::*;
use std::collections::HashMap;

// ── components ────────────────────────────────────────────

// ── resources ─────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct HospitalLedger { pub entries: Vec<String> }

impl HospitalLedger {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
    pub fn get(&self, _id: &str) -> Option<&HospitalLedgerDef> { None }
}

#[derive(Debug, Clone, Default)]
pub struct HospitalLedgerDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl HospitalLedgerDef { pub fn name(&self) -> &str { &self.name } }

#[derive(Debug, Clone, Copy, Default)]
pub struct StackSize(pub u8);
impl StackSize { pub fn get(self) -> u8 { self.0 } }

// ── events ────────────────────────────────────────

// ── helpers used by ported bodies ─────────────────






// ── ported systems ────────────────────────────────



// ── plugin ────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HospitalLedger>();
        let _ = app;
    }
}
