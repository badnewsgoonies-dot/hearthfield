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


/// Returns the tint colour for tree/bush objects based on season.
/// Each tree gets either the "a" or "b" variant depending on a hash of its
/// grid position, so the same map looks varied across adjacent trees.
pub fn tree_tint_helper(season: Season, variant_b: bool) -> Color {
    match season {
        Season::Spring => {
            if variant_b {
                Color::srgb(1.00, 0.80, 0.88) // brighter cherry blossom pink
            } else {
                Color::srgb(0.58, 1.00, 0.54) // vivid spring green
            }
        }
        Season::Summer => Color::srgb(0.80, 0.92, 0.42), // warm late-summer green
        Season::Fall => {
            if variant_b {
                Color::srgb(1.00, 0.76, 0.18) // harvest gold
            } else {
                Color::srgb(0.94, 0.42, 0.16) // stronger burnt orange
            }
        }
        Season::Winter => Color::srgb(0.74, 0.80, 0.86), // frosted blue-grey
    }
}


