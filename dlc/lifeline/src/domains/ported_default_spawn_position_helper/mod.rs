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


/// Returns a safe default spawn position for each map (e.g., for cutscene
/// teleports or fallback positioning).
pub fn default_spawn_position_helper(map_id: MapId) -> (i32, i32) {
    match map_id {
        MapId::Farm => (16, 12),
        MapId::Town => (12, 8),
        MapId::TownWest => (12, 14),
        MapId::Beach => (10, 6),
        MapId::Forest => (8, 8),
        MapId::DeepForest => (3, 15),
        MapId::MineEntrance => (7, 6),
        MapId::Mine => (12, 12),
        MapId::PlayerHouse => (8, 8),
        MapId::TownHouseWest => (6, 8),
        MapId::TownHouseEast => (6, 8),
        MapId::GeneralStore => (6, 8),
        MapId::AnimalShop => (6, 8),
        MapId::Blacksmith => (6, 8),
        MapId::Library => (7, 10),
        MapId::Tavern => (8, 12),
        MapId::CoralIsland => (15, 1),
        MapId::SnowMountain => (16, 22),
    }
}


