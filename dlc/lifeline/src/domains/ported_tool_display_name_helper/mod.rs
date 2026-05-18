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


/// Human-readable tool name for toast messages.
pub fn tool_display_name_helper(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Axe => "axe",
        ToolKind::Pickaxe => "pickaxe",
        ToolKind::Hoe => "hoe",
        ToolKind::WateringCan => "watering can",
        ToolKind::FishingRod => "fishing rod",
        ToolKind::Scythe => "scythe",
    }
}


