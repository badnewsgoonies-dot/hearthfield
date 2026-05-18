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


/// Candle positions for each indoor map. Returns (grid_x, grid_y) pairs.
pub fn candle_positions_helper(map_id: MapId) -> Vec<(i32, i32)> {
    match map_id {
        MapId::PlayerHouse => vec![(2, 2), (8, 2), (3, 12), (12, 13)],
        MapId::GeneralStore => vec![(1, 1), (10, 1), (1, 9)],
        MapId::Blacksmith => vec![(1, 3), (10, 1), (5, 7)],
        MapId::AnimalShop => vec![(1, 3), (10, 1), (10, 9)],
        MapId::TownHouseWest => vec![(2, 1), (8, 2), (3, 8)],
        MapId::TownHouseEast => vec![(2, 1), (9, 2), (9, 8)],
        MapId::Tavern => vec![(2, 2), (8, 2), (12, 5), (4, 10)],
        MapId::Library => vec![(2, 2), (11, 2), (2, 7), (11, 8)],
        _ => vec![],
    }
}


