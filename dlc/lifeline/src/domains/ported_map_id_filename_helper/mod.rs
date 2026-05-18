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


/// Map a `MapId` to its lowercase filename (without extension).
pub fn map_id_filename_helper(map_id: MapId) -> &'static str {
    match map_id {
        MapId::Farm => "farm",
        MapId::Town => "town",
        MapId::TownWest => "town_west",
        MapId::Beach => "beach",
        MapId::Forest => "forest",
        MapId::DeepForest => "deep_forest",
        MapId::MineEntrance => "mine_entrance",
        MapId::Mine => "mine",
        MapId::PlayerHouse => "player_house",
        MapId::TownHouseWest => "town_house_west",
        MapId::TownHouseEast => "town_house_east",
        MapId::GeneralStore => "general_store",
        MapId::AnimalShop => "animal_shop",
        MapId::Blacksmith => "blacksmith",
        MapId::Library => "library",
        MapId::Tavern => "tavern",
        MapId::CoralIsland => "coral_island",
        MapId::SnowMountain => "snow_mountain",
    }
}


