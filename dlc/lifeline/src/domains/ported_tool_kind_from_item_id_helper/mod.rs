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


/// Maps a tool item ID to its corresponding ToolKind.
pub fn tool_kind_from_item_id_helper(item_id: &str) -> Option<ToolKind> {
    match item_id {
        "hoe" => Some(ToolKind::Hoe),
        "watering_can" => Some(ToolKind::WateringCan),
        "axe" => Some(ToolKind::Axe),
        "pickaxe" => Some(ToolKind::Pickaxe),
        "fishing_rod" => Some(ToolKind::FishingRod),
        "scythe" => Some(ToolKind::Scythe),
        _ => None,
    }
}


