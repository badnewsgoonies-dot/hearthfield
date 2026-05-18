//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Deterministic hash for bush tint variety.
pub fn tint_hash(x: i32, y: i32) -> u32 {
    let h = (x as u32)
        .wrapping_mul(1103515245)
        .wrapping_add((y as u32).wrapping_mul(12345));
    h % 100
}


