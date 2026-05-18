//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FestivalKind {
    #[default]
    EggFestival,
    HarvestFestival,
    Luau,
    WinterStar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Season {
    #[default]
    Fall,
    Spring,
    Summer,
    Winter,
}


pub fn shift_for_date(season: Season, day: u8) -> Option<FestivalKind> {
    match (season, day) {
        (Season::Spring, 13) => Some(FestivalKind::EggFestival),
        (Season::Summer, 11) => Some(FestivalKind::Luau),
        (Season::Fall, 16) => Some(FestivalKind::HarvestFestival),
        (Season::Winter, 25) => Some(FestivalKind::WinterStar),
        _ => None,
    }
}


