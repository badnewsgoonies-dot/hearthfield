//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Returns the shimmer color for a valuable ore, or None for plain stone/copper/iron.
pub fn shimmer_color_for_ore_helper(drop_item: &str) -> Option<Color> {
    match drop_item {
        "gold_ore" => Some(Color::srgb(1.0, 0.9, 0.3)),
        "iridium_ore" => Some(Color::srgb(0.7, 0.5, 1.0)),
        "diamond" => Some(Color::srgb(0.9, 0.95, 1.0)),
        "ruby" => Some(Color::srgb(1.0, 0.3, 0.3)),
        "emerald" => Some(Color::srgb(0.3, 1.0, 0.4)),
        "quartz" => Some(Color::srgb(0.95, 0.92, 0.85)),
        "amethyst" => Some(Color::srgb(0.75, 0.4, 0.95)),
        _ => None,
    }
}


