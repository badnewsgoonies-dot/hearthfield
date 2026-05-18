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


pub fn map_display_name_helper(map: MapId) -> &'static str {
    match map {
        MapId::Farm => "Hearthfield Farm",
        MapId::Town => "Willowbrook",
        MapId::TownWest => "West Willowbrook",
        MapId::Beach => "Tide Pool Beach",
        MapId::Forest => "Briarwood Forest",
        MapId::DeepForest => "Deep Briarwood",
        MapId::MineEntrance => "The Mines",
        MapId::Mine => "Mine Floor",
        MapId::PlayerHouse => "Home",
        MapId::TownHouseWest => "Town House West",
        MapId::TownHouseEast => "Town House East",
        MapId::GeneralStore => "General Store",
        MapId::AnimalShop => "Animal Shop",
        MapId::Blacksmith => "Elena's Forge",
        MapId::Library => "Willowbrook Library",
        MapId::Tavern => "The Copper Cup",
        MapId::CoralIsland => "Coral Island",
        MapId::SnowMountain => "Snowy Mountain",
    }
}


