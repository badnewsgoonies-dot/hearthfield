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


pub fn map_id_display_name_helper(map_id: MapId) -> &'static str {
    match map_id {
        MapId::Farm => "Farm",
        MapId::Town => "Town",
        MapId::TownWest => "West Willowbrook",
        MapId::Beach => "Beach",
        MapId::Forest => "Forest",
        MapId::DeepForest => "Deep Forest",
        MapId::MineEntrance => "Mine",
        MapId::Mine => "Mine (Deep)",
        MapId::PlayerHouse => "Player House",
        MapId::TownHouseWest => "Town House West",
        MapId::TownHouseEast => "Town House East",
        MapId::GeneralStore => "General Store",
        MapId::AnimalShop => "Animal Shop",
        MapId::Blacksmith => "Blacksmith",
        MapId::Library => "Library",
        MapId::Tavern => "Tavern",
        MapId::CoralIsland => "Coral Island",
        MapId::SnowMountain => "Snowy Mountain",
    }
}


