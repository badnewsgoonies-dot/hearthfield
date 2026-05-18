//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Map a hash value to a bush color variant for visual variety.
pub fn bush_variant_color_helper(hash: u32) -> Color {
    if hash < 30 {
        // Slightly yellower (berry bush feel)
        Color::srgb(1.0, 0.95, 0.8)
    } else if hash < 60 {
        // Darker green
        Color::srgb(0.8, 0.95, 0.8)
    } else {
        // Default green
        Color::WHITE
    }
}


