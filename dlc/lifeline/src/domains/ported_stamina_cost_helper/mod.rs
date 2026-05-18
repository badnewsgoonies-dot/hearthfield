//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolKind {
    #[default]
    Axe,
    FishingRod,
    Hoe,
    Pickaxe,
    Scythe,
    WateringCan,
}


/// Stamina cost for each tool kind.
pub fn stamina_cost_helper(tool: &ToolKind) -> f32 {
    match tool {
        ToolKind::Hoe => 4.0,
        ToolKind::WateringCan => 3.0,
        ToolKind::Axe => 6.0,
        ToolKind::Pickaxe => 6.0,
        ToolKind::FishingRod => 4.0,
        ToolKind::Scythe => 2.0,
    }
}


