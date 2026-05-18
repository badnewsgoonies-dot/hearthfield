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


/// Atlas index in furniture.png for each machine type.
pub fn machine_atlas_index_helper(machine_type: MachineType) -> usize {
    match machine_type {
        MachineType::Furnace => 22,
        MachineType::PreservesJar => 23,
        MachineType::Keg => 24,
        MachineType::CheesePress => 25,
        MachineType::Loom => 26,
        MachineType::OilMaker => 19,
        MachineType::MayonnaiseMachine => 20,
        MachineType::Tapper => 21,
        MachineType::BeeHouse => 27,
        MachineType::RecyclingMachine => 28,
        MachineType::CrabPot => 29,
    }
}


