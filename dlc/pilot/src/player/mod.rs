//! Player domain — pilot character movement, interaction, camera.

use bevy::prelude::*;
use crate::shared::*;

pub mod movement;
pub mod interaction;
pub mod camera;
pub mod spawn;
pub mod apartment;
pub mod logbook;
pub mod skills;
pub mod reputation;
pub mod day_cycle;
pub mod journal;
pub mod collections;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<apartment::ApartmentState>()
            .init_resource::<logbook::Logbook>()
            .init_resource::<skills::PilotSkills>()
            .init_resource::<reputation::Reputation>()
            .init_resource::<journal::Journal>()
            .init_resource::<collections::Collections>()
            .add_systems(OnEnter(GameState::Playing), spawn::spawn_player)
            .add_systems(
                Update,
                (
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
            )
            .add_systems(
                Update,
                (
                    journal::record_first_airport_visit.run_if(in_state(GameState::Playing)),
                    journal::record_first_emergency.run_if(in_state(GameState::Playing)),
                    journal::record_rank_promotion.run_if(in_state(GameState::Playing)),
                    journal::record_achievement.run_if(in_state(GameState::Playing)),
                    journal::check_journal_milestones.run_if(in_state(GameState::Playing)),
                    journal::write_custom_entry.run_if(in_state(GameState::Playing)),
                    collections::auto_collect_stamp.run_if(in_state(GameState::Playing)),
                    collections::purchase_collectible.run_if(in_state(GameState::Playing)),
                    collections::check_collection_completion.run_if(in_state(GameState::Playing)),
                    collections::generate_trade_offers.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}
