//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GiftPreference {
    #[default]
    Disliked,
    Hated,
    Liked,
    Loved,
    Neutral,
}


/// Convert a GiftPreference to friendship point delta (positive or negative).
pub fn preference_to_points_helper(preference: GiftPreference) -> i32 {
    match preference {
        GiftPreference::Loved => 80,
        GiftPreference::Liked => 45,
        GiftPreference::Neutral => 20,
        GiftPreference::Disliked => -20,
        GiftPreference::Hated => -40,
    }
}


