//! Catch resolution and escape logic.
//!
//! These are helper functions called from within systems, not systems themselves.
//!
//! # Treasure integration
//! The new `check_and_grant_treasure` function from `treasure.rs` is used here
//! instead of the old inline loot logic. Bait type affects the treasure chance:
//!
//! | Bait / Condition | Treasure chance |
//! |------------------|-----------------|
//! | No bait          | 10%             |
//! | Generic bait     | 10%             |
//! | wild_bait        | 15%             |
//! | magnet_bait      | 25%             |

use bevy::prelude::*;

use super::treasure::{
    check_and_grant_treasure, BASE_TREASURE_CHANCE, MAGNET_BAIT_EXTRA_CHANCE,
    WILD_BAIT_EXTRA_CHANCE,
};
use super::{FishEncyclopedia, FishingPhase, FishingState};
use crate::shared::*;

// ─── catch_fish ───────────────────────────────────────────────────────────────

/// Called when the player successfully catches a fish.
///
/// `bait_id` is the specific bait item ID used for this cast (e.g. "wild_bait"),
/// or `None` if no bait was equipped. This determines the treasure bonus.
#[allow(clippy::too_many_arguments)]
pub fn catch_fish(
    fishing_state: &mut FishingState,
    next_state: &mut NextState<GameState>,
    _stamina_events: &mut EventWriter<StaminaDrainEvent>,
    item_pickup_events: &mut EventWriter<ItemPickupEvent>,
    sfx_events: &mut EventWriter<PlaySfxEvent>,
    fish_registry: &FishRegistry,
    commands: &mut Commands,
    bobber_entities: Vec<Entity>,
    encyclopedia: &mut FishEncyclopedia,
    calendar: &Calendar,
    toast_events: &mut EventWriter<ToastEvent>,
    gold_events: &mut EventWriter<GoldChangeEvent>,
    bait_id: Option<&str>,
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

    // Toast for the catch
    let catch_name = fish_registry
        .fish
        .get(&valid_id)
        .map(|f| f.name.clone())
        .unwrap_or_else(|| valid_id.clone());
    toast_events.send(ToastEvent {
        message: format!("Caught a {}!", catch_name),
        duration_secs: 2.5,
    });

    // ── Fish Encyclopedia ──────────────────────────────────────────────────
    let total_days = calendar.total_days_elapsed();
    let is_new = encyclopedia.record_catch(&valid_id, total_days, calendar.season);

    if is_new {
        // Look up the fish name from registry for a friendly toast message.
        let fish_name = fish_registry
            .fish
            .get(&valid_id)
            .map(|f| f.name.as_str())
            .unwrap_or(&valid_id);
        toast_events.send(ToastEvent {
            message: format!("New fish: {}!", fish_name),
            duration_secs: 3.0,
        });
    }

    // ── Legendary catch toast ──────────────────────────────────────────────
    if super::legendaries::is_legendary(&valid_id) {
        let fish_name = fish_registry
            .fish
            .get(&valid_id)
            .map(|f| f.name.as_str())
            .unwrap_or(&valid_id);
        toast_events.send(ToastEvent {
            message: format!("LEGENDARY CATCH: {}! Incredible!", fish_name),
            duration_secs: 5.0,
        });
        sfx_events.send(PlaySfxEvent {
            sfx_id: "legendary_catch".to_string(),
        });
    }

    // ── Treasure Chest ────────────────────────────────────────────────────
    // Bait affects treasure probability:
    //   - magnet_bait adds +15% (MAGNET_BAIT_EXTRA_CHANCE)  → 25% total
    //   - wild_bait adds +5% (WILD_BAIT_EXTRA_CHANCE)       → 15% total
    //   - any other bait / no bait: base rate only           → 10% total
    let treasure_chance = match bait_id {
        Some("magnet_bait") => BASE_TREASURE_CHANCE + MAGNET_BAIT_EXTRA_CHANCE,
        Some("wild_bait") => BASE_TREASURE_CHANCE + WILD_BAIT_EXTRA_CHANCE,
        _ => BASE_TREASURE_CHANCE,
    };

    check_and_grant_treasure(
        treasure_chance,
        item_pickup_events,
        gold_events,
        toast_events,
        sfx_events,
    );

    // Sound effect for the catch itself
    sfx_events.send(PlaySfxEvent {
        sfx_id: "fish_caught".to_string(),
    });

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

// ─── end_fishing_escape ───────────────────────────────────────────────────────

/// Called when a fish escapes or the player cancels fishing.
///
/// `from_fishing_state` is true when called from within `GameState::Fishing`
/// (so we need to transition back to Playing). When called from `GameState::Playing`
/// during the waiting phase, no state transition is needed.
pub fn end_fishing_escape(
    fishing_state: &mut FishingState,
    next_state: &mut NextState<GameState>,
    _stamina_events: &mut EventWriter<StaminaDrainEvent>,
    commands: &mut Commands,
    bobber_entities: Vec<Entity>,
    from_fishing_state: bool,
) {
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
