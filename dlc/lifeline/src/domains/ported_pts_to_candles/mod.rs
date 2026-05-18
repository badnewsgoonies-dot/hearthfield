//! Single-fn substrate port — pure/near-pure.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;



/// Convert a raw point total (0–21) into a candle count.
pub fn points_to_credits(points: u32) -> u8 {
    match points {
        0..=5 => 1,
        6..=10 => 2,
        11..=15 => 3,
        _ => 4, // 16-21
    }
}


