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


/// Map SoilState to an atlas index in tilled_dirt.png (11 cols × 7 rows).
///
/// Index 0  — clean plain tilled dirt fill
/// Index 4  — alternate plain fill used for watered soil before tinting
pub fn bed_atlas_index(state: SoilState) -> usize {
    match state {
        SoilState::Untilled => 0, // shouldn't normally be rendered
        SoilState::Tilled => 0,
        SoilState::Watered => 4,
    }
}


