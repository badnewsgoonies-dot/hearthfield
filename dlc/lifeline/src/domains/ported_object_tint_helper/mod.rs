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


/// Returns the tint for non-tree world objects (rocks, stumps, bushes, logs).
/// Rocks and stone objects should stay mostly neutral; only get a slight
/// season shift for consistency.
pub fn object_tint_helper(season: Season) -> Color {
    match season {
        Season::Spring => Color::srgb(0.96, 1.00, 0.97),
        Season::Summer => Color::srgb(1.00, 0.96, 0.88),
        Season::Fall => Color::srgb(0.98, 0.84, 0.68),
        Season::Winter => Color::srgb(0.88, 0.94, 1.00),
    }
}


