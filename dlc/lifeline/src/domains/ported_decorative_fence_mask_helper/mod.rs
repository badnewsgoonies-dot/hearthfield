//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Compute autotile bitmask (0-15) for a fence post at `(x, y)` given the
/// full set of fence grid positions. Uses the same bit convention as
/// `farming::render::fence_autotile_index`:
///   bit 0 = north neighbor  (x, y-1)
///   bit 1 = east neighbor   (x+1, y)
///   bit 2 = south neighbor  (x, y+1)
///   bit 3 = west neighbor   (x-1, y)
pub fn decorative_fence_mask_helper(positions: &[(i32, i32)], x: i32, y: i32) -> usize {
    let has = |tx: i32, ty: i32| positions.iter().any(|&(px, py)| px == tx && py == ty);
    let mut mask: u8 = 0;
    if has(x, y - 1) {
        mask |= 1;
    }
    if has(x + 1, y) {
        mask |= 2;
    }
    if has(x, y + 1) {
        mask |= 4;
    }
    if has(x - 1, y) {
        mask |= 8;
    }
    mask as usize
}


