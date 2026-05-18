//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Build a square area of side `2 * radius + 1` centred on `(cx, cy)`.
pub fn square_area_helper(cx: i32, cy: i32, radius: i32) -> Vec<(i32, i32)> {
    let side = 2 * radius + 1;
    let mut tiles = Vec::with_capacity((side * side) as usize);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            tiles.push((cx + dx, cy + dy));
        }
    }
    tiles
}


