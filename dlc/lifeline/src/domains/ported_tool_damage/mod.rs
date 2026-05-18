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


/// Pickaxe damage per tool tier.
pub fn tool_damage(tier: ToolTier) -> u8 {
    match tier {
        ToolTier::Basic => 1,
        ToolTier::Copper => 2,
        ToolTier::Iron => 3,
        ToolTier::Gold => 4,
        ToolTier::Iridium => 5,
    }
}


