//! NPC domain plugin for Hearthfield.
//!
//! Manages all 10 townspeople: their schedules, movement, dialogue, and gift system.
//! Communicates exclusively through shared resources and events.

use bevy::prelude::*;
use crate::shared::*;

mod definitions;
mod dialogue;
mod gifts;
mod map_events;
mod schedule;
mod spawning;

use definitions::build_npc_registry;
use dialogue::{handle_npc_interaction, ActiveNpcInteraction};
use gifts::{handle_gifts, handle_gift_input};
use map_events::{handle_map_transition, handle_day_end};
use schedule::{
    update_npc_schedules,
    move_npcs_toward_targets,
    ScheduleUpdateTimer,
};
use spawning::{spawn_initial_npcs, SpawnedNpcs, NpcSpriteData};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        // Initialize NPC-domain resources
        app
            .init_resource::<SpawnedNpcs>()
            .init_resource::<NpcSpriteData>()
            .init_resource::<ActiveNpcInteraction>()
            .init_resource::<ScheduleUpdateTimer>();

        // Populate NPC registry on startup (before Loading completes)
        app.add_systems(Startup, setup_npc_registry);

        // Spawn initial NPCs when entering Playing
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_initial_npcs,
        );

        // Playing-state systems
        app.add_systems(
            Update,
            (
                // Schedule: update targets periodically
                update_npc_schedules,
                // Movement: smooth walk toward target every frame
                move_npcs_toward_targets,
                // Interaction: Space key triggers dialogue
                handle_npc_interaction,
                // Gift input: G key gives selected item
                handle_gift_input,
                // Gift resolution: process GiftGivenEvent
                handle_gifts,
                // Map transition: despawn/spawn NPCs for new map
                handle_map_transition,
                // Day end: reset gifted_today
                handle_day_end,
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
