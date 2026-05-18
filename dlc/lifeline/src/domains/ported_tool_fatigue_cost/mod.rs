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


/// Stamina cost for a pickaxe swing.
pub fn tool_fatigue_cost(tier: ToolTier) -> f32 {
    match tier {
        ToolTier::Basic => 3.5,
        ToolTier::Copper => 3.0,
        ToolTier::Iron => 2.6,
        ToolTier::Gold => 2.2,
        ToolTier::Iridium => 1.8,
    }
}


