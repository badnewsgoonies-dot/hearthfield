use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Pickaxe damage per tool tier.
pub fn combat_damage_system(tier: ToolTier) -> u8 {
    match tier {
        ToolTier::Basic => 1,
        ToolTier::Copper => 2,
        ToolTier::Iron => 3,
        ToolTier::Gold => 4,
        ToolTier::Iridium => 5,
    }
}


