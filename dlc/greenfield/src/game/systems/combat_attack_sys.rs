use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Player combat damage based on pickaxe tier (doubles as weapon).
pub fn combat_attack_system(tier: ToolTier) -> f32 {
    match tier {
        ToolTier::Basic => 10.0,
        ToolTier::Copper => 15.0,
        ToolTier::Iron => 20.0,
        ToolTier::Gold => 30.0,
        ToolTier::Iridium => 50.0,
    }
}


