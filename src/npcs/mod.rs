//! NPC domain plugin for Hearthfield.
//!
//! Manages all 10 townspeople: their schedules, movement, dialogue, and gift system.
//! Communicates exclusively through shared resources and events.

use bevy::prelude::*;
use crate::shared::*;

mod animation;
mod definitions;
mod dialogue;
mod gifts;
mod map_events;
pub mod romance;
pub mod quests;
mod schedule;
pub mod schedules;
mod spawning;

use definitions::build_npc_registry;
use dialogue::{handle_npc_interaction, ActiveNpcInteraction};
use gifts::{handle_gifts, handle_gift_input};
use map_events::{handle_map_transition, handle_day_end, GiftDecayTracker};
use romance::{
    WeddingTimer,
    update_relationship_stages,
    handle_bouquet,
    handle_proposal,
    tick_wedding_timer,
    handle_wedding,
    spouse_daily_action,
    handle_spouse_action,
    update_spouse_happiness,
};
use schedule::{
    update_npc_schedules,
    move_npcs_toward_targets,
    ScheduleUpdateTimer,
};
use schedules::{
    apply_enhanced_schedules,
    refresh_schedules_on_season_change,
    check_farm_visits,
    FarmVisitTracker,
};
use animation::animate_npc_sprites;
use spawning::{spawn_initial_npcs, SpawnedNpcs, NpcSpriteData};
use quests::{
    post_daily_quests,
    handle_quest_accepted,
    track_quest_progress,
    handle_quest_completed,
    expire_quests,
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        // Initialize NPC-domain resources
        app
            .init_resource::<SpawnedNpcs>()
            .init_resource::<NpcSpriteData>()
            .init_resource::<ActiveNpcInteraction>()
            .init_resource::<ScheduleUpdateTimer>()
            .init_resource::<GiftDecayTracker>()
            .init_resource::<WeddingTimer>()
            .init_resource::<FarmVisitTracker>();

        // Populate NPC registry on startup (before Loading completes)
        app.add_systems(Startup, setup_npc_registry);

        // Apply enhanced (seasonally-varied) schedules and spawn NPCs when entering Playing.
        // apply_enhanced_schedules must run before spawn_initial_npcs so spawning uses the
        // correct seasonal positions.
        app.add_systems(
            OnEnter(GameState::Playing),
            (apply_enhanced_schedules, spawn_initial_npcs).chain(),
        );

        // Playing-state systems: core NPC behaviour
        app.add_systems(
            Update,
            (
                update_npc_schedules,
                move_npcs_toward_targets,
                animate_npc_sprites,
                handle_npc_interaction,
                handle_gift_input,
                handle_gifts,
                handle_map_transition,
                handle_day_end,
                refresh_schedules_on_season_change,
                check_farm_visits,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // Playing-state systems: romance + quests
        app.add_systems(
            Update,
            (
                update_relationship_stages,
                handle_bouquet,
                handle_proposal,
                tick_wedding_timer,
                handle_wedding,
                spouse_daily_action,
                handle_spouse_action,
                update_spouse_happiness,
                post_daily_quests,
                handle_quest_accepted,
                track_quest_progress,
                handle_quest_completed,
                expire_quests,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// System: populate the NpcRegistry from our built-in definitions.
fn setup_npc_registry(mut npc_registry: ResMut<NpcRegistry>) {
    let registry = build_npc_registry();
    npc_registry.npcs = registry.npcs;
    npc_registry.schedules = registry.schedules;
}
