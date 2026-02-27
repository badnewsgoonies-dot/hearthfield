//! PlayStats tracker — passive event listeners that increment global play counters.
//!
//! Each system reads one or more shared events and bumps the corresponding field
//! in the `PlayStats` resource. No game logic is changed here; this module is
//! purely observational.

use bevy::prelude::*;
use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// System: crops_harvested
// ─────────────────────────────────────────────────────────────────────────────

/// Increments `PlayStats::crops_harvested` for every `CropHarvestedEvent`.
pub fn track_crops_harvested(
    mut events: EventReader<CropHarvestedEvent>,
    mut stats: ResMut<PlayStats>,
) {
    for _ev in events.read() {
        stats.crops_harvested = stats.crops_harvested.saturating_add(1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: fish_caught
// ─────────────────────────────────────────────────────────────────────────────

/// Increments `PlayStats::fish_caught` when an `ItemPickupEvent` is received
/// for an item that exists in the `FishRegistry`.
pub fn track_fish_caught(
    mut events: EventReader<ItemPickupEvent>,
    fish_registry: Res<FishRegistry>,
    mut stats: ResMut<PlayStats>,
) {
    for ev in events.read() {
        if fish_registry.fish.contains_key(&ev.item_id) {
            stats.fish_caught = stats.fish_caught.saturating_add(1);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: days_played + items_shipped
// ─────────────────────────────────────────────────────────────────────────────

/// On `DayEndEvent`:
///   - Increments `PlayStats::days_played` by 1.
///   - Increments `PlayStats::items_shipped` by the total quantity in the
///     `ShippingBin` *before* it is cleared by the shipping system.
///
/// Ordering note: this system runs in the same `Update` set as
/// `process_shipping_bin_on_day_end`. Both consume the same `DayEndEvent`
/// reader independently (Bevy fans events to all readers), so the bin contents
/// are still present when this system fires regardless of ordering.
pub fn track_day_end(
    mut events: EventReader<DayEndEvent>,
    shipping_bin: Res<ShippingBin>,
    mut stats: ResMut<PlayStats>,
) {
    for _ev in events.read() {
        stats.days_played = stats.days_played.saturating_add(1);

        // Count total items (by quantity) currently staged for shipping.
        let shipped_this_day: u64 = shipping_bin
            .items
            .iter()
            .map(|slot| slot.quantity as u64)
            .sum();
        stats.items_shipped = stats.items_shipped.saturating_add(shipped_this_day);

        info!(
            "[PlayStats] Day {} ended. items_shipped today: {}, total days: {}",
            _ev.day, shipped_this_day, stats.days_played
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: gifts_given
// ─────────────────────────────────────────────────────────────────────────────

/// Increments `PlayStats::gifts_given` for every `GiftGivenEvent`.
pub fn track_gifts_given(
    mut events: EventReader<GiftGivenEvent>,
    mut stats: ResMut<PlayStats>,
) {
    for _ev in events.read() {
        stats.gifts_given = stats.gifts_given.saturating_add(1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: animals_petted (proxy via AnimalProductEvent)
// ─────────────────────────────────────────────────────────────────────────────

/// Uses `AnimalProductEvent` as a proxy for animal interaction.
/// Each product collected is treated as one animal interaction, incrementing
/// `PlayStats::animals_petted`.
pub fn track_animals_petted(
    mut events: EventReader<AnimalProductEvent>,
    mut stats: ResMut<PlayStats>,
) {
    for _ev in events.read() {
        stats.animals_petted = stats.animals_petted.saturating_add(1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: total_gold_earned
// ─────────────────────────────────────────────────────────────────────────────

/// Increments `PlayStats::total_gold_earned` for every positive `GoldChangeEvent`.
pub fn track_gold_earned(
    mut events: EventReader<GoldChangeEvent>,
    mut stats: ResMut<PlayStats>,
) {
    for ev in events.read() {
        if ev.amount > 0 {
            stats.total_gold_earned =
                stats.total_gold_earned.saturating_add(ev.amount as u64);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: recipes_cooked (proxy via EatFoodEvent)
// ─────────────────────────────────────────────────────────────────────────────

/// Uses `EatFoodEvent` as a proxy for cooking activity.
/// Each food item consumed increments `PlayStats::recipes_cooked`.
pub fn track_recipes_cooked(
    mut events: EventReader<EatFoodEvent>,
    mut stats: ResMut<PlayStats>,
) {
    for _ev in events.read() {
        stats.recipes_cooked = stats.recipes_cooked.saturating_add(1);
    }
}
