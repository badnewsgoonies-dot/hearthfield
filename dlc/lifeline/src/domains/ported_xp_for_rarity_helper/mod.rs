//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Rarity {
    #[default]
    Common,
    Legendary,
    Rare,
    Uncommon,
}


/// XP awarded per catch by rarity.
pub fn xp_for_rarity_helper(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 3,
        Rarity::Uncommon => 8,
        Rarity::Rare => 15,
        Rarity::Legendary => 25,
    }
}


