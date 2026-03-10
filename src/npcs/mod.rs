//! NPC domain plugin for Hearthfield.
//!
//! Manages all 10 townspeople: their schedules, movement, dialogue, and gift system.
//! Communicates exclusively through shared resources and events.

use crate::shared::*;
use bevy::prelude::*;

mod animation;
pub mod definitions;
pub mod dialogue;
pub mod emotes;
mod gifts;
pub mod idle_behavior;
pub mod map_events;
pub mod quests;
pub mod romance;
mod schedule;
pub mod schedules;
pub mod spawning;

use animation::animate_npc_sprites;
use dialogue::{handle_npc_interaction, reset_daily_talks, ActiveNpcInteraction, DailyTalkTracker};
use emotes::{animate_emote_bubbles, spawn_emote_bubbles, EmoteSprites, NpcEmoteEvent};
use gifts::{handle_gift_input, handle_gifts};
use idle_behavior::{attach_npc_shadows, npc_idle_behavior_system, ShadowSpriteCache};
use map_events::{handle_day_end, handle_map_transition, GiftDecayTracker};
use quests::{
    check_story_quests, expire_quests, handle_quest_accepted, handle_quest_completed,
    log_quest_posted, post_daily_quests, post_seasonal_quests, track_monster_slain,
    track_quest_progress,
};
use romance::{
    handle_bouquet, handle_proposal, handle_spouse_action, handle_wedding, spouse_daily_action,
    tick_wedding_timer, update_relationship_stages, update_spouse_happiness, WeddingTimer,
};
use schedule::{move_npcs_toward_targets, update_npc_schedules, ScheduleUpdateTimer};
use schedules::{
    apply_enhanced_schedules, check_farm_visits, refresh_schedules_on_season_change,
    FarmVisitTracker,
};
use spawning::{preload_npc_sprites, spawn_initial_npcs, NpcSpriteData, SpawnedNpcs};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        // Initialize NPC-domain resources
        app.init_resource::<SpawnedNpcs>()
            .init_resource::<NpcSpriteData>()
            .init_resource::<ActiveNpcInteraction>()
            .init_resource::<DailyTalkTracker>()
            .init_resource::<ScheduleUpdateTimer>()
            .init_resource::<GiftDecayTracker>()
            .init_resource::<WeddingTimer>()
            .init_resource::<FarmVisitTracker>()
            .init_resource::<EmoteSprites>()
            .init_resource::<ShadowSpriteCache>()
            .add_event::<NpcEmoteEvent>();

        // NPC data is populated by DataPlugin during OnEnter(Loading).

        // Apply enhanced (seasonally-varied) schedules and spawn NPCs when entering Playing.
        // apply_enhanced_schedules must run before spawn_initial_npcs so spawning uses the
        // correct seasonal positions.
        app.add_systems(
            OnEnter(GameState::Playing),
            (
                preload_npc_sprites,
                apply_enhanced_schedules,
                spawn_initial_npcs,
            )
                .chain(),
        );

        // NPC interaction runs before the world interaction dispatcher so NPCs
        // take priority over world objects when both are within range.
        app.add_systems(
            Update,
            handle_npc_interaction
                .in_set(UpdatePhase::Intent)
                .run_if(in_state(GameState::Playing))
                .before(crate::player::interact_dispatch::dispatch_world_interaction),
        );

        // Deterministic schedule resolution cadence.
        app.add_systems(
            FixedUpdate,
            update_npc_schedules
                .in_set(UpdatePhase::Simulation)
                .run_if(in_state(GameState::Playing)),
        );

        // Playing-state systems: core NPC behaviour
        app.add_systems(
            Update,
            (
                move_npcs_toward_targets,
                animate_npc_sprites,
                handle_gift_input,
                handle_gifts,
                handle_map_transition,
                handle_day_end,
                refresh_schedules_on_season_change,
                check_farm_visits,
                spawn_emote_bubbles,
                animate_emote_bubbles,
                npc_idle_behavior_system,
                attach_npc_shadows,
                reset_daily_talks,
            )
                .in_set(UpdatePhase::Simulation)
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
                post_seasonal_quests,
                check_story_quests,
                log_quest_posted,
                handle_quest_accepted,
                track_quest_progress,
                track_monster_slain,
                handle_quest_completed,
                expire_quests,
            )
                .in_set(UpdatePhase::Reactions)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
