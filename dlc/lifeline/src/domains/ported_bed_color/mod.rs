//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SoilState {
    #[default]
    Tilled,
    Untilled,
    Watered,
}


/// Return the placeholder colour for a soil state.
pub fn bed_color(state: SoilState) -> Color {
    match state {
        SoilState::Untilled => Color::srgb(0.55, 0.42, 0.28), // light dirt (shouldn't be rendered)
        SoilState::Tilled => Color::srgb(0.45, 0.32, 0.20),   // medium brown
        SoilState::Watered => Color::srgb(0.30, 0.22, 0.15),  // dark wet soil
    }
}


