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


pub fn severity_weight(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 60,
        Rarity::Uncommon => 25,
        Rarity::Rare => 12,
        // Legendary fish in the normal pool (registered via data) have very low
        // weight; they are primarily obtained through try_roll_legendary().
        Rarity::Legendary => 1,
    }
}


