//! Airport domain — zone maps, tile rendering, zone transitions, object spawning.

use bevy::prelude::*;
use crate::shared::*;

pub mod maps;
pub mod render;
pub mod objects;
pub mod transitions;
pub mod facilities;
pub mod city_exploration;
pub mod npcs;
pub mod announcements;
pub mod ground_ops;
pub mod services;

pub struct AirportPlugin;

impl Plugin for AirportPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<announcements::AnnouncementTimer>()
            .init_resource::<ground_ops::GroundOpsState>()
            .init_resource::<services::AirportServiceState>()
            .add_systems(
                Update,
                (
                    transitions::handle_zone_transition.run_if(in_state(GameState::Playing)),
                    transitions::handle_airport_arrival.run_if(in_state(GameState::Playing)),
                    render::sync_collision_map.run_if(in_state(GameState::Playing)),
                    npcs::spawn_ambient_npcs.run_if(in_state(GameState::Playing)),
                    npcs::update_npc_patrol.run_if(in_state(GameState::Playing)),
                    npcs::interact_ambient_npc.run_if(in_state(GameState::Playing)),
                    announcements::play_announcements.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_systems(
                OnEnter(GameState::Playing),
                render::spawn_initial_map,
            )
            .add_systems(
                Update,
                (
                    ground_ops::setup_ground_ops_on_arrival.run_if(in_state(GameState::Playing)),
                    ground_ops::update_turnaround.run_if(in_state(GameState::Playing)),
                    ground_ops::update_taxi_progress.run_if(in_state(GameState::Playing)),
                    ground_ops::request_pushback.run_if(in_state(GameState::Playing)),
                    ground_ops::update_pushback.run_if(in_state(GameState::Playing)),
                    services::use_hotel.run_if(in_state(GameState::Playing)),
                    services::use_car_rental.run_if(in_state(GameState::Playing)),
                    services::use_lounge.run_if(in_state(GameState::Playing)),
                    services::request_weather_briefing.run_if(in_state(GameState::Playing)),
                    services::reset_services_on_arrival.run_if(in_state(GameState::Playing)),
                ),
            )
            .add_plugins(facilities::FacilityPlugin)
            .add_plugins(city_exploration::CityExplorationPlugin);
    }
}
