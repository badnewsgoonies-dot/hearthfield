//! Crew domain — NPC pilots, lounge interactions, relationships, dialogue.

use bevy::prelude::*;
use crate::shared::*;

pub mod spawning;
pub mod dialogue;
pub mod gifts;
pub mod schedules;
pub mod relationships;
pub mod lounge;
pub mod events;
pub mod abilities;

pub struct CrewPlugin;

impl Plugin for CrewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<events::CrewEventState>()
            .init_resource::<abilities::CrewBonuses>()
            .init_resource::<spawning::CrewSpriteData>()
            .init_resource::<spawning::CrewPortraitData>()
            .add_systems(
            Update,
            (
                spawning::spawn_crew_for_zone.run_if(in_state(GameState::Playing)),
                dialogue::handle_dialogue_start.run_if(in_state(GameState::Playing)),
                dialogue::advance_dialogue.run_if(in_state(GameState::Dialogue)),
                gifts::handle_gift_given.run_if(in_state(GameState::Playing)),
                gifts::reset_daily_gifts.run_if(in_state(GameState::Playing)),
                events::check_birthdays.run_if(in_state(GameState::Playing)),
                events::birthday_gift_bonus.run_if(in_state(GameState::Playing)),
                events::check_crew_conflicts.run_if(in_state(GameState::Playing)),
                events::check_crew_achievements.run_if(in_state(GameState::Playing)),
                events::check_crew_departure.run_if(in_state(GameState::Playing)),
                events::check_holiday_events.run_if(in_state(GameState::Playing)),
                events::check_mentorship.run_if(in_state(GameState::Playing)),
                events::cleanup_resolved_events.run_if(in_state(GameState::Playing)),
                abilities::evaluate_crew_abilities.run_if(in_state(GameState::Playing)),
                abilities::apply_xp_bonus.run_if(in_state(GameState::Playing)),
                abilities::apply_fuel_bonus.run_if(in_state(GameState::Flying)),
            ),
        )
        .add_plugins(relationships::RelationshipPlugin)
        .add_plugins(lounge::LoungePlugin);
    }
}
