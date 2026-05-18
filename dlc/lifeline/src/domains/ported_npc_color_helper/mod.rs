//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Placeholder sprite tint per NPC (used for name tags).
pub fn npc_color_helper(npc_id: &str) -> Color {
    match npc_id {
        "margaret" => Color::srgb(0.9, 0.6, 0.3), // warm orange (baker)
        "marco" => Color::srgb(0.8, 0.3, 0.2),    // warm red (chef)
        "lily" => Color::srgb(1.0, 0.8, 0.2),     // sunny yellow (florist)
        "old_tom" => Color::srgb(0.5, 0.5, 0.3),  // weathered tan (fisherman)
        "elena" => Color::srgb(0.5, 0.4, 0.3),    // forge-brown (blacksmith)
        "mira" => Color::srgb(0.6, 0.4, 0.8),     // exotic violet (merchant)
        "doc" => Color::srgb(0.3, 0.7, 0.7),      // teal (doctor)
        "mayor_rex" => Color::srgb(0.4, 0.3, 0.7), // regal purple (mayor)
        "sam" => Color::srgb(0.4, 0.4, 0.4),      // stone grey (musician)
        "nora" => Color::srgb(0.4, 0.6, 0.3),     // earthy green (farmer)
        "bjorn" => Color::srgb(0.6, 0.65, 0.7),   // frosty blue-grey (hermit)
        _ => Color::srgb(0.8, 0.8, 0.8),          // fallback grey
    }
}


