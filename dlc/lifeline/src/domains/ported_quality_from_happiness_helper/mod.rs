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


/// Returns the `ItemQuality` corresponding to an animal's happiness value.
///
/// Thresholds:
/// - 220-255 → Iridium (2.0x)
/// - 180-219 → Gold    (1.5x)
/// - 110-179 → Silver  (1.25x)
/// -   0-109 → Normal  (1.0x)
pub fn quality_from_happiness_helper(happiness: u8) -> ItemQuality {
    if happiness >= 220 {
        ItemQuality::Iridium
    } else if happiness >= 180 {
        ItemQuality::Gold
    } else if happiness >= 110 {
        ItemQuality::Silver
    } else {
        ItemQuality::Normal
    }
}


