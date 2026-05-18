//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

pub const TILE_SIZE: f32 = 0.0;


/// Convert world-space position to grid coordinates.
/// Uses floor() — a point at (15.9, 31.1) with TILE_SIZE=16 is tile (0, 1).
/// This is the ONLY sanctioned world→grid conversion in the codebase.
pub fn world_to_grid_helper(wx: f32, wy: f32) -> IVec2 {
    IVec2::new(
        (wx / TILE_SIZE).floor() as i32,
        (wy / TILE_SIZE).floor() as i32,
    )
}


