//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Deterministic hash of tile position for consistent decoration placement.
/// Returns a value in 0..1000.
pub fn bed_tile_hash(x: usize, y: usize) -> u32 {
    // Simple but effective hash for visual variety.
    let h = (x as u32)
        .wrapping_mul(2654435761)
        .wrapping_add((y as u32).wrapping_mul(2246822519));
    h % 1000
}


