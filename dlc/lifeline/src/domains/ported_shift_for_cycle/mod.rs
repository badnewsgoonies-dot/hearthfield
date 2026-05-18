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


pub fn shift_for_cycle(season: Season) -> (u8, &'static str) {
    match season {
        Season::Spring => (13, "Egg Fest"),
        Season::Summer => (11, "Luau"),
        Season::Fall => (16, "Harvest"),
        Season::Winter => (25, "W.Star"),
    }
}


