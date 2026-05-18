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


/// Player combat damage based on pickaxe tier (doubles as weapon).
pub fn staff_action_damage(tier: ToolTier) -> f32 {
    match tier {
        ToolTier::Basic => 10.0,
        ToolTier::Copper => 15.0,
        ToolTier::Iron => 20.0,
        ToolTier::Gold => 30.0,
        ToolTier::Iridium => 50.0,
    }
}


