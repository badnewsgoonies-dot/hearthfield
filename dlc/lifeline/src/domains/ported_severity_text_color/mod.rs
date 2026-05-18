//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ItemQuality {
    #[default]
    Gold,
    Iridium,
    Normal,
    Silver,
}


/// Returns a quality-appropriate text color: white for normal, silver shimmer,
/// gold, or purple/prismatic for iridium.
pub fn severity_text_color(quality: ItemQuality) -> Color {
    match quality {
        ItemQuality::Normal => Color::WHITE,
        ItemQuality::Silver => Color::srgb(0.78, 0.82, 0.88), // subtle silver
        ItemQuality::Gold => Color::srgb(1.0, 0.84, 0.0),     // gold
        ItemQuality::Iridium => Color::srgb(0.7, 0.4, 1.0),   // purple/prismatic
    }
}


