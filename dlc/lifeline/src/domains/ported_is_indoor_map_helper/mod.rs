//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MapId {
    #[default]
    AnimalShop,
    Blacksmith,
    GeneralStore,
    Library,
    PlayerHouse,
    Tavern,
    TownHouseEast,
    TownHouseWest,
}


/// Returns true if the given map is indoors (no weather particles).
pub fn is_indoor_map_helper(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PlayerHouse
            | MapId::TownHouseWest
            | MapId::TownHouseEast
            | MapId::GeneralStore
            | MapId::AnimalShop
            | MapId::Blacksmith
            | MapId::Library
            | MapId::Tavern
    )
}


