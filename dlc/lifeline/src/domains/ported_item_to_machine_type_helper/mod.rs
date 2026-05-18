//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MachineType {
    #[default]
    BeeHouse,
    CheesePress,
    CrabPot,
    Furnace,
    Keg,
    Loom,
    MayonnaiseMachine,
    OilMaker,
    PreservesJar,
    RecyclingMachine,
    Tapper,
}


/// Returns the MachineType that corresponds to a placeable item id, or None if
/// the item is not a placeable machine.
pub fn item_to_machine_type_helper(item_id: &str) -> Option<MachineType> {
    match item_id {
        "furnace" => Some(MachineType::Furnace),
        "preserves_jar" => Some(MachineType::PreservesJar),
        "cheese_press" => Some(MachineType::CheesePress),
        "loom" => Some(MachineType::Loom),
        "keg" => Some(MachineType::Keg),
        "oil_maker" => Some(MachineType::OilMaker),
        "mayonnaise_machine" => Some(MachineType::MayonnaiseMachine),
        "tapper" => Some(MachineType::Tapper),
        "bee_house" => Some(MachineType::BeeHouse),
        "recycling_machine" => Some(MachineType::RecyclingMachine),
        "crab_pot" => Some(MachineType::CrabPot),
        _ => None,
    }
}


