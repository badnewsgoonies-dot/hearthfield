//! Catch resolution and escape logic.
//!
//! These are helper functions called from within systems, not systems themselves.

use bevy::prelude::*;

use crate::shared::*;
use super::{FishingPhase, FishingState};

/// Called when the player successfully catches a fish.
pub fn catch_fish(
    fishing_state: &mut FishingState,
    next_state: &mut NextState<GameState>,
    stamina_events: &mut EventWriter<StaminaDrainEvent>,
    item_pickup_events: &mut EventWriter<ItemPickupEvent>,
    sfx_events: &mut EventWriter<PlaySfxEvent>,
    fish_registry: &FishRegistry,
    commands: &mut Commands,
    bobber_entities: Vec<Entity>,
) {
    // Determine what was caught
    let fish_id = fishing_state
        .selected_fish_id
        .clone()
        .unwrap_or_else(|| "carp".to_string());

    // Validate the fish exists in registry; if not, use any available fish
    let valid_id = if fish_registry.fish.contains_key(&fish_id) {
        fish_id
    } else {
        fish_registry
            .fish
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "carp".to_string())
    };

    // Add fish to inventory
    item_pickup_events.send(ItemPickupEvent {
        item_id: valid_id.clone(),
        quantity: 1,
    });

    // Sound effects
    sfx_events.send(PlaySfxEvent {
        sfx_id: "fish_caught".to_string(),
    });

    // Drain stamina (full cast costs 4)
    stamina_events.send(StaminaDrainEvent { amount: 4.0 });

    // Despawn bobber
    for entity in bobber_entities {
        commands.entity(entity).despawn_recursive();
    }

    // Reset fishing state
    fishing_state.reset();
    fishing_state.phase = FishingPhase::Idle;

    // Return to Playing state (OnExit(Fishing) will clean up minigame UI)
    next_state.set(GameState::Playing);
}

/// Called when a fish escapes or the player cancels fishing.
///
/// `from_fishing_state` is true when called from within `GameState::Fishing`
/// (so we need to transition back to Playing). When called from `GameState::Playing`
/// during the waiting phase, no state transition is needed.
pub fn end_fishing_escape(
    fishing_state: &mut FishingState,
    next_state: &mut NextState<GameState>,
    stamina_events: &mut EventWriter<StaminaDrainEvent>,
    commands: &mut Commands,
    bobber_entities: Vec<Entity>,
    from_fishing_state: bool,
) {
    // Partial stamina drain for a failed or cancelled cast
    stamina_events.send(StaminaDrainEvent { amount: 2.0 });

    // Despawn bobber
    for entity in bobber_entities {
        commands.entity(entity).despawn_recursive();
    }

    // Reset fishing state
    fishing_state.reset();
    fishing_state.phase = FishingPhase::Idle;

    // Only transition if we were in the Fishing game state
    if from_fishing_state {
        next_state.set(GameState::Playing);
    }
}
