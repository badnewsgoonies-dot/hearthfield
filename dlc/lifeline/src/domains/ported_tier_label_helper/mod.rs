//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildingTier {
    #[default]
    Basic,
    Big,
    Deluxe,
    None,
}


pub fn tier_label_helper(tier: BuildingTier) -> &'static str {
    match tier {
        BuildingTier::None => "None",
        BuildingTier::Basic => "Basic",
        BuildingTier::Big => "Big",
        BuildingTier::Deluxe => "Deluxe",
    }
}


