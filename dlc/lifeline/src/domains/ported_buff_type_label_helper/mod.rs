//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuffType {
    #[default]
    Attack,
    Defense,
    Farming,
    Fishing,
    Luck,
    MaxStamina,
    Mining,
    Speed,
}


/// Returns a human-readable label for a BuffType.
pub fn buff_type_label_helper(buff_type: BuffType) -> &'static str {
    match buff_type {
        BuffType::Speed => "Speed",
        BuffType::Mining => "Mining",
        BuffType::Fishing => "Fishing",
        BuffType::Farming => "Farming",
        BuffType::Defense => "Defense",
        BuffType::Attack => "Attack",
        BuffType::Luck => "Luck",
        BuffType::MaxStamina => "Max Stamina",
    }
}


