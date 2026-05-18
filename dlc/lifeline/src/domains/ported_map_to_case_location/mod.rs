//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FishLocation {
    #[default]
    MinePool,
    MountainLake,
    Ocean,
    Pond,
    River,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    Beach,
    Farm,
    Forest,
    Mine,
    MineEntrance,
    SnowMountain,
    Town,
}


pub fn map_to_case_location(map_id: MapId) -> FishLocation {
    match map_id {
        MapId::Farm | MapId::Forest => FishLocation::River,
        MapId::Beach => FishLocation::Ocean,
        MapId::Town => FishLocation::Pond,
        MapId::Mine | MapId::MineEntrance => FishLocation::MinePool,
        MapId::SnowMountain => FishLocation::MountainLake,
        // Indoor maps default to pond
        _ => FishLocation::Pond,
    }
}


