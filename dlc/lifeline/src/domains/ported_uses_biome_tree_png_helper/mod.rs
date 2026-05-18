//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    DeepForest,
    Farm,
    Forest,
    SnowMountain,
    Town,
    TownWest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WorldObjectKind {
    #[default]
    Pine,
    Tree,
}


/// Returns true if this map+kind combination should use a biome-specific
/// individual tree PNG instead of the shared tree_sprites.png atlas.
pub fn uses_biome_tree_png_helper(map_id: MapId, kind: WorldObjectKind) -> bool {
    match kind {
        WorldObjectKind::Tree => matches!(
            map_id,
            MapId::Farm | MapId::Town | MapId::TownWest | MapId::Forest | MapId::DeepForest
        ),
        WorldObjectKind::Pine => matches!(map_id, MapId::SnowMountain),
        _ => false,
    }
}


