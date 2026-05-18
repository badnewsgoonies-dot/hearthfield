//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolTier {
    #[default]
    Basic,
    Copper,
    Gold,
    Iridium,
    Iron,
}


pub fn tool_tier_label_helper(tier: ToolTier) -> &'static str {
    match tier {
        ToolTier::Basic => "Basic",
        ToolTier::Copper => "Copper",
        ToolTier::Iron => "Iron",
        ToolTier::Gold => "Gold",
        ToolTier::Iridium => "Iridium",
    }
}


