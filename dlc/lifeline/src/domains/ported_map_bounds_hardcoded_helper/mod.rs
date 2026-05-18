//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    AnimalShop,
    Beach,
    Blacksmith,
    CoralIsland,
    DeepForest,
    Farm,
    Forest,
    GeneralStore,
    Library,
    Mine,
    MineEntrance,
    PlayerHouse,
    SnowMountain,
    Tavern,
    Town,
    TownHouseEast,
    TownHouseWest,
    TownWest,
}


/// Hardcoded fallback for map bounds (kept for safety).
pub fn map_bounds_hardcoded_helper(map: &MapId) -> (i32, i32, i32, i32) {
    match map {
        MapId::Farm => (0, 31, 0, 23),
        MapId::Town => (0, 27, 0, 21),
        MapId::TownWest => (0, 15, 0, 21),
        MapId::Beach => (0, 19, 0, 13),
        MapId::Forest => (0, 21, 0, 17),
        MapId::DeepForest => (0, 29, 0, 27),
        MapId::MineEntrance => (0, 13, 0, 11),
        MapId::Mine => (0, 23, 0, 23),
        MapId::PlayerHouse => (0, 15, 0, 15),
        MapId::TownHouseWest => (0, 11, 0, 11),
        MapId::TownHouseEast => (0, 11, 0, 11),
        MapId::GeneralStore => (0, 11, 0, 11),
        MapId::AnimalShop => (0, 11, 0, 11),
        MapId::Blacksmith => (0, 11, 0, 11),
        MapId::Library => (0, 13, 0, 11),
        MapId::Tavern => (0, 15, 0, 13),
        MapId::CoralIsland => (0, 29, 0, 21),
        MapId::SnowMountain => (0, 31, 0, 23),
    }
}


