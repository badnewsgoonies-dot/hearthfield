//! pure host fn substrate port.

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


pub fn shift_display_name(kind: FestivalKind) -> &'static str {
    match kind {
        FestivalKind::EggFestival => "Egg Festival",
        FestivalKind::Luau => "Luau",
        FestivalKind::HarvestFestival => "Harvest Festival",
        FestivalKind::WinterStar => "Winter Star Festival",
    }
}


