//! Catch resolution and escape logic.

use bevy::prelude::*;

use crate::shared::*;
use super::{Bobber, FishingPhase, FishingState, FishingMinigameState};

/// Called when the player successfully catches a fish.
pub fn catch_fish(
    mut fishing_state: ResMut<FishingState>,
    mut _minigame_state: ResMut<FishingMinigameState>,
    next_state: &mut NextState<GameState>,
    stamina_events: &mut EventWriter<StaminaDrainEvent>,
    item_pickup_events: &mut EventWriter<ItemPickupEvent>,
    sfx_events: &mut EventWriter<PlaySfxEvent>,
    fish_registry: &FishRegistry,
    commands: &mut Commands,
    bobber_entities: Vec<Entity>,
) {
    // Determine what was caught
    let fish_id = fishing_state.selected_fish_id.clone().unwrap_or_else(|| "carp".to_string());

    // Validate the fish exists in registry; if not, default to carp
    let valid_id = if fish_registry.fish.contains_key(&fish_id) {
        fish_id
    } else {
        // Pick any fish as fallback
        fish_registry
            .fish
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "carp".to_string())
    };

    // Send fish to inventory
    item_pickup_events.write(ItemPickupEvent {
        item_id: valid_id.clone(),
        quantity: 1,
    });

    // Sound effects
    sfx_events.write(PlaySfxEvent {
        sfx_id: "fish_caught".to_string(),
    });

    // Drain stamina (full cast = 4)
    stamina_events.write(StaminaDrainEvent { amount: 4.0 });

    // Despawn bobber
    for entity in bobber_entities {
        commands.entity(entity).despawn_recursive();
    }

    // Reset fishing state
    fishing_state.reset();
    fishing_state.phase = FishingPhase::Idle;

    // Return to playing state (OnExit(Fishing) will clean up minigame UI)
    next_state.set(GameState::Playing);
}

/// Called when a fish escapes or the player cancels.
///
/// `from_fishing_state` is true when called from within `GameState::Fishing`
/// (so we need to set state back to Playing). When called from `GameState::Playing`
/// (waiting phase), the state is already Playing.
pub fn end_fishing_escape(
    fishing_state: &mut FishingState,
    next_state: &mut NextState<GameState>,
    stamina_events: &mut EventWriter<StaminaDrainEvent>,
    commands: &mut Commands,
    bobber_entities: Vec<Entity>,
    from_fishing_state: bool,
) {
    // Drain a smaller amount of stamina for a failed/cancelled cast
    stamina_events.write(StaminaDrainEvent { amount: 2.0 });

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
