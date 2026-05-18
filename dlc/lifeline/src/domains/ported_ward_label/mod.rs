//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildingKind {
    #[default]
    Barn,
    Coop,
    House,
    Silo,
}


pub fn ward_label(kind: BuildingKind) -> &'static str {
    match kind {
        BuildingKind::House => "House",
        BuildingKind::Coop => "Coop",
        BuildingKind::Barn => "Barn",
        BuildingKind::Silo => "Silo",
    }
}


