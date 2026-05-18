//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Season {
    #[default]
    Fall,
    Spring,
    Summer,
    Winter,
}


/// Returns the season tint multiplier colour for terrain tiles.
///
/// The tint is applied as a multiplicative colour: White = no change.
pub fn terrain_tint_helper(season: Season) -> Color {
    match season {
        Season::Spring => Color::srgb(0.86, 1.00, 0.88), // fresher green lift
        Season::Summer => Color::srgb(1.00, 0.94, 0.80), // sun-baked golden warmth
        Season::Fall => Color::srgb(1.00, 0.70, 0.42),   // warmer orange-amber cast
        Season::Winter => Color::srgb(0.82, 0.90, 1.00), // cool blue-white
    }
}


