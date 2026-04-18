//! Lifeline::domains::patients — skeleton plugin.
//! Briefcase `hearthfield_*` transforms splice systems/events/resources
//! into this module via structural anchors (see CONVENTIONS below).
//!
//! CONVENTIONS (for briefcase):
//!   - New events are added to `crate::shared` then registered in `build`.
//!   - New components go directly below the `// ── components ──` anchor.
//!   - New system fns go directly below the `// ── systems ──` anchor
//!     and are registered inside `build` with `.add_systems(Update, …)`.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────────

// ── resources ─────────────────────────────────────────────────────────

// ── systems ───────────────────────────────────────────────────────────

// ── plugin ────────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, _app: &mut App) {
        // Briefcase wires registrations here via `hearthfield_register_system`
        // and `hearthfield_wire_event`.
    }
}
