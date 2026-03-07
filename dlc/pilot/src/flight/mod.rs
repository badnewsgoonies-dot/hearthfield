//! Flight domain — preflight, takeoff, cruise, landing mechanics.

use crate::shared::*;
use bevy::prelude::*;

pub mod atc;
pub mod autopilot;
pub mod checklists;
pub mod cruise;
pub mod emergencies;
pub mod fuel_planning;
pub mod instruments;
pub mod landing;
pub mod navigation;
pub mod passengers;
pub mod preflight;
pub mod radio;
pub mod takeoff;
pub mod weather_effects;

pub struct FlightPlugin;

impl Plugin for FlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<preflight::PreflightProgress>()
            .init_resource::<takeoff::TakeoffState>()
            .add_systems(
                Update,
                (
                    preflight::run_preflight_checklist.run_if(in_state(GameState::Flying)),
                    takeoff::transition_taxi_to_takeoff.run_if(in_state(GameState::Flying)),
                    takeoff::handle_takeoff.run_if(in_state(GameState::Flying)),
                    cruise::update_flight.run_if(in_state(GameState::Flying)),
                    landing::evaluate_landing.run_if(in_state(GameState::Flying)),
                    navigation::update_navigation.run_if(in_state(GameState::Flying)),
                ),
            )
            .add_systems(
                OnEnter(GameState::Flying),
                (
                    preflight::reset_preflight_on_enter,
                    takeoff::reset_takeoff_state,
                ),
            )
            .add_plugins(emergencies::EmergencyPlugin)
            .add_plugins(autopilot::AutopilotPlugin)
            .add_plugins(radio::RadioPlugin)
            .add_plugins(instruments::InstrumentPlugin);
    }
}
