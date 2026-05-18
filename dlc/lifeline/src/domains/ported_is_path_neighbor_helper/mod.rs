//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileKind {
    #[default]
    Bridge,
    Path,
}


pub fn is_path_neighbor_helper(
    tiles: &[TileKind],
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    dx: i32,
    dy: i32,
) -> bool {
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
        return false;
    }
    matches!(
        tiles[ny as usize * width + nx as usize],
        TileKind::Path | TileKind::Bridge
    )
}


