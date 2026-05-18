//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Convert a raw point total (0–21) into a candle count.
pub fn score_to_credits(points: u32) -> u8 {
    match points {
        0..=5 => 1,
        6..=10 => 2,
        11..=15 => 3,
        _ => 4, // 16-21
    }
}


