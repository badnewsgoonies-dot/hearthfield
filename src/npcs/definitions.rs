//! NPC definitions: canonical ID list and placeholder colour mapping.
//!
//! NPC data (gift preferences, dialogue, schedules) is populated by
//! DataPlugin in `src/data/npcs.rs` during `OnEnter(GameState::Loading)`.
//! This file only provides the ID constant used by spawning/schedules
//! and a colour helper for placeholder sprites.

use bevy::prelude::*;

/// The 10 canonical NPC IDs, matching `src/data/npcs.rs` exactly.
pub const ALL_NPC_IDS: &[&str] = &[
    "margaret",
    "marco",
    "lily",
    "old_tom",
    "elena",
    "mira",
    "doc",
    "mayor_rex",
    "sam",
    "nora",
];

/// Placeholder sprite tint per NPC (used when sprite sheets aren't loaded).
pub fn npc_color(npc_id: &str) -> Color {
    match npc_id {
        "margaret"  => Color::srgb(0.9, 0.6, 0.3), // warm orange (baker)
        "marco"     => Color::srgb(0.8, 0.3, 0.2), // warm red (chef)
        "lily"      => Color::srgb(1.0, 0.8, 0.2), // sunny yellow (florist)
        "old_tom"   => Color::srgb(0.5, 0.5, 0.3), // weathered tan (fisherman)
        "elena"     => Color::srgb(0.5, 0.4, 0.3), // forge-brown (blacksmith)
        "mira"      => Color::srgb(0.6, 0.4, 0.8), // exotic violet (merchant)
        "doc"       => Color::srgb(0.3, 0.7, 0.7), // teal (doctor)
        "mayor_rex" => Color::srgb(0.4, 0.3, 0.7), // regal purple (mayor)
        "sam"       => Color::srgb(0.4, 0.4, 0.4), // stone grey (musician)
        "nora"      => Color::srgb(0.4, 0.6, 0.3), // earthy green (farmer)
        _           => Color::srgb(0.8, 0.8, 0.8), // fallback grey
    }
}
