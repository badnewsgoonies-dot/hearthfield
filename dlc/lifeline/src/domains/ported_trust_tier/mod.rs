//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Get the dialogue tier key for a given heart count.
/// Tiers: 0 (0-2 hearts), 3 (3-5 hearts), 6 (6-8 hearts), 9 (9-10 hearts)
pub fn trust_tier(hearts: u8) -> u8 {
    match hearts {
        0..=2 => 0,
        3..=5 => 3,
        6..=8 => 6,
        _ => 9,
    }
}


