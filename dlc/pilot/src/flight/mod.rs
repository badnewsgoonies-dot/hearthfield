//! Flight domain — preflight, takeoff, cruise, landing mechanics.

use bevy::prelude::*;
use crate::shared::*;

pub mod preflight;
pub mod takeoff;
pub mod cruise;
pub mod landing;
pub mod navigation;
pub mod emergencies;
pub mod autopilot;
pub mod radio;
pub mod instruments;
pub mod atc;
pub mod weather_effects;
pub mod passengers;
pub mod checklists;
pub mod fuel_planning;

pub struct FlightPlugin;

impl Plugin for FlightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<preflight::PreflightProgress>()
            .init_resource::<takeoff::TakeoffState>()
            .init_resource::<atc::AtcState>()
            .init_resource::<weather_effects::WeatherEffects>()
            .init_resource::<passengers::CabinState>()
            .init_resource::<checklists::ActiveChecklist>()
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
                    checklists::reset_checklist_on_flight_start,
                ),
            )
            .add_systems(
                Update,
                (
                    atc::process_takeoff_clearance.run_if(in_state(GameState::Flying)),
                    atc::handle_frequency_handoffs.run_if(in_state(GameState::Flying)),
                    atc::assign_approach.run_if(in_state(GameState::Flying)),
                    atc::evaluate_go_around.run_if(in_state(GameState::Flying)),
                    atc::enforce_altitude_restriction.run_if(in_state(GameState::Flying)),
                    atc::update_holding_pattern.run_if(in_state(GameState::Flying)),
                    atc::request_takeoff_clearance.run_if(in_state(GameState::Flying)),
                    atc::reset_atc_on_arrival.run_if(in_state(GameState::Flying)),
                    weather_effects::apply_turbulence.run_if(in_state(GameState::Flying)),
                    weather_effects::update_icing.run_if(in_state(GameState::Flying)),
                    weather_effects::toggle_de_icing.run_if(in_state(GameState::Flying)),
                    weather_effects::de_icing_fuel_cost.run_if(in_state(GameState::Flying)),
                ),
            )
            .add_systems(
                Update,
                (
                    weather_effects::update_crosswind.run_if(in_state(GameState::Flying)),
                    weather_effects::update_visibility.run_if(in_state(GameState::Flying)),
                    weather_effects::detect_windshear.run_if(in_state(GameState::Flying)),
                    weather_effects::check_mountain_waves.run_if(in_state(GameState::Flying)),
                    passengers::update_passenger_mood.run_if(in_state(GameState::Flying)),
                    passengers::auto_cabin_announcements.run_if(in_state(GameState::Flying)),
                    passengers::service_rounds.run_if(in_state(GameState::Flying)),
                    passengers::random_passenger_events.run_if(in_state(GameState::Flying)),
                    passengers::calculate_final_satisfaction.run_if(in_state(GameState::Flying)),
                ),
            )
            .add_systems(
                OnEnter(GameState::Flying),
                atc::setup_atc_on_flight_start,
            )
            .add_systems(
                Update,
                (
                    checklists::advance_checklist.run_if(in_state(GameState::Flying)),
                    checklists::skip_checklist_item.run_if(in_state(GameState::Flying)),
                    fuel_planning::display_fuel_plan_on_preflight.run_if(in_state(GameState::Flying)),
                ),
            )
            .add_plugins(emergencies::EmergencyPlugin)
            .add_plugins(autopilot::AutopilotPlugin)
            .add_plugins(radio::RadioPlugin)
            .add_plugins(instruments::InstrumentPlugin);
    }
}
