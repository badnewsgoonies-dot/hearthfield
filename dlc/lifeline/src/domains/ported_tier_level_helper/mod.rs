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


/// Returns a numeric level for a ToolTier, used for tier comparison.
pub fn tier_level_helper(tier: ToolTier) -> u8 {
    match tier {
        ToolTier::Basic => 0,
        ToolTier::Copper => 1,
        ToolTier::Iron => 2,
        ToolTier::Gold => 3,
        ToolTier::Iridium => 4,
    }
}


