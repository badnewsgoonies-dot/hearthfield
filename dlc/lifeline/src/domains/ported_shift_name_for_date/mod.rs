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


pub fn shift_name_for_date(season: Season, day: u8) -> Option<&'static str> {
    match (season, day) {
        (Season::Spring, 13) => Some("Egg Festival"),
        (Season::Summer, 11) => Some("Luau"),
        (Season::Fall, 16) => Some("Harvest Festival"),
        (Season::Winter, 25) => Some("Winter Star Festival"),
        _ => None,
    }
}


