//! Lifeline::domains::ui — substrate-mechanical port.
//! No LLM. parser_v3 + substitution map + cargo gate.

use bevy::prelude::*;
use std::collections::HashMap;

// ── components ────────────────────────────────────────────

// ── resources ─────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default)]
pub struct StackSize(pub u8);
impl StackSize { pub fn get(self) -> u8 { self.0 } }

// ── events ────────────────────────────────────────

#[derive(Event, Debug, Clone, Default)]
pub struct StatusToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers used by ported bodies ─────────────────






// ── ported systems ────────────────────────────────



// ── plugin ────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StatusToastEvent>();
        let _ = app;
    }
}
