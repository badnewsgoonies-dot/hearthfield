//! Player domain — pilot character movement, interaction, camera.

use crate::shared::*;
use bevy::prelude::*;

pub mod apartment;
pub mod camera;
pub mod collections;
pub mod day_cycle;
pub mod interaction;
pub mod journal;
pub mod logbook;
pub mod movement;
pub mod reputation;
pub mod skills;
pub mod spawn;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<apartment::ApartmentState>()
            .init_resource::<logbook::Logbook>()
            .init_resource::<skills::PilotSkills>()
            .init_resource::<reputation::Reputation>()
            .init_resource::<spawn::PlayerSpriteData>()
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn::spawn_player, spawn::setup_new_game),
            )
            .add_systems(
                Update,
                (
                    spawn::animate_spawn_fade.run_if(in_state(GameState::Playing)),
                    spawn::respawn_after_day_end.run_if(in_state(GameState::Playing)),
                    movement::player_movement.run_if(in_state(GameState::Playing)),
                    interaction::check_interactions.run_if(in_state(GameState::Playing)),
                    camera::follow_camera.run_if(in_state(GameState::Playing)),
                    interaction::handle_day_end.run_if(in_state(GameState::Playing)),
                    apartment::interact_furniture.run_if(in_state(GameState::Playing)),
                    apartment::morning_routine.run_if(in_state(GameState::Playing)),
                    logbook::record_flight.run_if(in_state(GameState::Playing)),
                    logbook::check_logbook_rank_requirement.run_if(in_state(GameState::Playing)),
                    skills::award_flight_skill_xp.run_if(in_state(GameState::Playing)),
                    skills::award_emergency_xp.run_if(in_state(GameState::Playing)),
                    skills::daily_practice_bonus.run_if(in_state(GameState::Playing)),
                    skills::flight_school_training.run_if(in_state(GameState::Playing)),
                    reputation::update_reputation_on_flight.run_if(in_state(GameState::Playing)),
                    reputation::reputation_decay.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}
