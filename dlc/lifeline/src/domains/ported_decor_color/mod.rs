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


pub fn decor_color(season: Season) -> Color {
    match season {
        Season::Spring => Color::srgba(0.35, 0.75, 0.25, 0.9), // bright green
        Season::Summer => Color::srgba(0.20, 0.60, 0.20, 0.9), // deep green
        Season::Fall => Color::srgba(0.80, 0.45, 0.10, 0.9),   // orange-brown
        Season::Winter => Color::srgba(0.70, 0.70, 0.70, 0.7), // pale grey (sparse)
    }
}


