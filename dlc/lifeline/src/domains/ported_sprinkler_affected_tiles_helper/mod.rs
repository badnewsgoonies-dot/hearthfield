//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct SprinklerKind;

impl SprinklerKind {
    pub fn includes_diagonals(&self) -> bool { false }
    pub fn range(&self) -> bool { false }
}


/// Compute all tiles watered by a sprinkler of `kind` placed at (`cx`, `cy`).
///
/// * `Basic`   — range 1, cardinal only  →  4 tiles  (N/S/E/W)
/// * `Quality` — range 1, with diagonals →  8 tiles  (3×3 minus centre)
/// * `Iridium` — range 2, with diagonals → 24 tiles  (5×5 minus centre)
pub fn sprinkler_affected_tiles_helper(kind: SprinklerKind, cx: i32, cy: i32) -> Vec<(i32, i32)> {
    let range = kind.range() as i32;
    let diags = kind.includes_diagonals();
    let mut tiles = Vec::new();
    for dx in -range..=range {
        for dy in -range..=range {
            if dx == 0 && dy == 0 {
                continue; // skip the centre (the sprinkler's own tile)
            }
            if !diags && dx != 0 && dy != 0 {
                continue; // cardinal-only: skip diagonals
            }
            tiles.push((cx + dx, cy + dy));
        }
    }
    tiles
}


