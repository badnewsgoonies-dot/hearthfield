//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    DeepForest,
    Forest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WorldObjectKind {
    #[default]
    Pine,
    Tree,
}


/// Sprite size for individual biome tree PNGs (pixels, pre-scale).
pub fn ward_decor_png_size(map_id: MapId, kind: WorldObjectKind) -> Vec2 {
    match kind {
        WorldObjectKind::Pine => Vec2::new(64.0, 96.0),
        WorldObjectKind::Tree => match map_id {
            MapId::Forest | MapId::DeepForest => Vec2::new(48.0, 80.0),
            _ => Vec2::new(80.0, 96.0), // oak green/brown
        },
        _ => Vec2::new(32.0, 48.0),
    }
}


