//! Aircraft domain — fleet management, maintenance, fuel, upgrades.

use crate::shared::*;
use bevy::prelude::*;

pub mod fleet;
pub mod fuel;
pub mod inspections;
pub mod maintenance;
pub mod upgrades;

pub struct AircraftPlugin;

impl Plugin for AircraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                maintenance::check_maintenance.run_if(in_state(GameState::Playing)),
                fuel::handle_refuel.run_if(in_state(GameState::Playing)),
                fleet::handle_flight_complete_aircraft.run_if(in_state(GameState::Playing)),
            ),
        )
        .add_plugins(upgrades::UpgradePlugin)
        .add_plugins(inspections::InspectionPlugin);
    }
}
