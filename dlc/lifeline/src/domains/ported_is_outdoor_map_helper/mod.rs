//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    Beach,
    CoralIsland,
    DeepForest,
    Farm,
    Forest,
    MineEntrance,
    SnowMountain,
    Town,
    TownWest,
}


/// Returns true if a map is an outdoor area (not an interior building or mine).
pub fn is_outdoor_map_helper(map: MapId) -> bool {
    matches!(
        map,
        MapId::Farm
            | MapId::Town
            | MapId::TownWest
            | MapId::Beach
            | MapId::Forest
            | MapId::DeepForest
            | MapId::CoralIsland
            | MapId::MineEntrance
            | MapId::SnowMountain
    )
}


