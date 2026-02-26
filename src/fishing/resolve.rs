//! Catch resolution and escape logic.
//!
//! These are helper functions called from within systems, not systems themselves.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;
use super::{FishingPhase, FishingState, FishEncyclopedia};

// ─── Treasure loot tables ─────────────────────────────────────────────────────

/// Chance that a successful catch also yields a bonus treasure chest item.
const TREASURE_CHANCE: f64 = 0.15;

/// Pick a random bonus treasure item ID based on a weighted loot table.
fn roll_treasure_item(rng: &mut impl Rng) -> &'static str {
    // Tier probabilities:
    //   40% → ore  (copper_ore, iron_ore, gold_ore)
    //   30% → gem  (amethyst, diamond, ruby, emerald)
    //   20% → artifact (ancient_doll, rusty_spoon, dinosaur_egg)
    //   10% → rare (iridium_ore, prismatic_shard)
    let tier_roll: f64 = rng.gen();

    if tier_roll < 0.40 {
        // Ore tier
        let ores = ["copper_ore", "iron_ore", "gold_ore"];
        ores[rng.gen_range(0..ores.len())]
    } else if tier_roll < 0.70 {
        // Gem tier
        let gems = ["amethyst", "diamond", "ruby", "emerald"];
        gems[rng.gen_range(0..gems.len())]
    } else if tier_roll < 0.90 {
        // Artifact tier
        let artifacts = ["ancient_doll", "rusty_spoon", "dinosaur_egg"];
        artifacts[rng.gen_range(0..artifacts.len())]
    } else {
        // Rare tier
        let rares = ["iridium_ore", "prismatic_shard"];
        rares[rng.gen_range(0..rares.len())]
    }
}

// ─── catch_fish ───────────────────────────────────────────────────────────────

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
    encyclopedia: &mut FishEncyclopedia,
    calendar: &Calendar,
    toast_events: &mut EventWriter<ToastEvent>,
) {
    let mut rng = rand::thread_rng();

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

    // ── Treasure Chest ────────────────────────────────────────────────────
    if rng.gen_bool(TREASURE_CHANCE) {
        let bonus_item = roll_treasure_item(&mut rng);
        item_pickup_events.send(ItemPickupEvent {
            item_id: bonus_item.to_string(),
            quantity: 1,
        });
        toast_events.send(ToastEvent {
            message: "You found treasure!".to_string(),
            duration_secs: 3.0,
        });
        sfx_events.send(PlaySfxEvent {
            sfx_id: "treasure_found".to_string(),
        });
    }

    // Sound effect for the catch itself
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

// ─── end_fishing_escape ───────────────────────────────────────────────────────

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
