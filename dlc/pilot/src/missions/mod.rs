//! Mission domain — mission board, assignments, completion tracking.

use bevy::prelude::*;
use crate::shared::*;

pub mod board;
pub mod tracking;
pub mod contracts;
pub mod cargo;
pub mod passengers;
pub mod special;
pub mod story;

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<contracts::ContractBoard>()
            .init_resource::<cargo::CargoManifest>()
            .init_resource::<passengers::PassengerManifest>()
            .init_resource::<special::SpecialMissionState>()
            .init_resource::<story::StoryProgress>()
            .add_systems(
                Update,
                (
                    board::refresh_mission_board.run_if(in_state(GameState::Playing)),
                    tracking::track_active_mission.run_if(in_state(GameState::Playing)),
                    tracking::handle_mission_complete.run_if(in_state(GameState::Playing)),
                    board::handle_mission_accepted.run_if(in_state(GameState::Playing)),
                    contracts::refresh_contract_board.run_if(in_state(GameState::Playing)),
                    contracts::evaluate_contracts.run_if(in_state(GameState::Playing)),
                    contracts::track_contract_flights.run_if(in_state(GameState::Playing)),
                    cargo::load_cargo.run_if(in_state(GameState::Playing)),
                    cargo::cargo_condition.run_if(in_state(GameState::Playing)),
                    cargo::rate_cargo_delivery.run_if(in_state(GameState::Playing)),
                    cargo::generate_cargo_for_mission.run_if(in_state(GameState::Playing)),
                    passengers::board_passengers.run_if(in_state(GameState::Playing)),
                    passengers::update_satisfaction.run_if(in_state(GameState::Playing)),
                    passengers::deplane_passengers.run_if(in_state(GameState::Playing)),
                    passengers::generate_passengers_for_mission.run_if(in_state(GameState::Playing)),
                    special::init_special_mission.run_if(in_state(GameState::Playing)),
                    special::update_special_mission.run_if(in_state(GameState::Flying)),
                    special::complete_special_mission.run_if(in_state(GameState::Playing)),
                    story::track_story_progress.run_if(in_state(GameState::Playing)),
                    story::show_story_mission_on_board.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}
