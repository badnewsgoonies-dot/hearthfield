use bevy::prelude::*;
use crate::shared::*;
use crate::economy::gold::EconomyStats;

// ─────────────────────────────────────────────────────────────────────────────
// Events (internal)
// ─────────────────────────────────────────────────────────────────────────────

/// Fired by the player interaction system when they deposit an item in the shipping bin.
#[derive(Event, Debug, Clone)]
pub struct ShipItemEvent {
    pub item_id: ItemId,
    pub quantity: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Listens for ShipItemEvents and moves items from inventory into the shipping bin.
/// The actual sale happens at end of day.
pub fn place_in_shipping_bin(
    mut ship_events: EventReader<ShipItemEvent>,
    mut inventory: ResMut<Inventory>,
    mut shipping_bin: ResMut<ShippingBin>,
    item_registry: Res<ItemRegistry>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for ev in ship_events.read() {
        // Verify item exists.
        let item_def = match item_registry.get(&ev.item_id) {
            Some(def) => def,
            None => {
                warn!("[Economy] Cannot ship unknown item '{}'", ev.item_id);
                continue;
            }
        };

        let quantity = ev.quantity.max(1);

        // Verify inventory holds enough.
        if !inventory.has(&ev.item_id, quantity) {
            info!(
                "[Economy] Cannot ship {} × '{}': only {} in inventory.",
                quantity,
                ev.item_id,
                inventory.count(&ev.item_id)
            );
            continue;
        }

        // Remove from inventory.
        let removed = inventory.try_remove(&ev.item_id, quantity);
        if removed == 0 {
            continue;
        }

        // Add to bin (merge stacks if already present).
        let existing = shipping_bin
            .items
            .iter_mut()
            .find(|slot| slot.item_id == ev.item_id);

        if let Some(slot) = existing {
            slot.quantity = slot.quantity.saturating_add(removed);
        } else {
            shipping_bin.items.push(InventorySlot {
                item_id: ev.item_id.clone(),
                quantity: removed,
            });
        }

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "shipping_bin".to_string(),
        });

        info!(
            "[Economy] Placed {} × '{}' in shipping bin. Bin now holds {} unique items.",
            removed,
            ev.item_id,
            shipping_bin.items.len()
        );

        // Suppress unused variable warning for item_def (we used it for validation above).
        let _ = item_def;
    }
}

/// Fires on DayEndEvent: sells everything in the shipping bin, adds gold, clears bin.
/// This is the primary income source for the player.
pub fn process_shipping_bin_on_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut shipping_bin: ResMut<ShippingBin>,
    mut player_state: ResMut<PlayerState>,
    item_registry: Res<ItemRegistry>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut stats: ResMut<EconomyStats>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for _ev in day_end_events.read() {
        if shipping_bin.items.is_empty() {
            continue;
        }

        let mut total_value: u32 = 0;
        let mut items_shipped: u64 = 0;
        let mut sale_details: Vec<String> = Vec::new();

        for slot in shipping_bin.items.iter() {
            let sell_price = item_registry
                .get(&slot.item_id)
                .map(|def| def.sell_price)
                .unwrap_or(1); // fallback: 1g for unknown items

            let slot_value = sell_price.saturating_mul(slot.quantity as u32);
            total_value = total_value.saturating_add(slot_value);
            items_shipped += slot.quantity as u64;

            sale_details.push(format!(
                "{} × '{}' = {}g",
                slot.quantity, slot.item_id, slot_value
            ));
        }

        // Apply the gold directly.
        player_state.gold = player_state.gold.saturating_add(total_value);
        stats.total_gold_earned = stats.total_gold_earned.saturating_add(total_value as u64);
        stats.total_items_shipped = stats.total_items_shipped.saturating_add(items_shipped);

        // Emit a single GoldChangeEvent for tracking/UI (gold already applied above).
        gold_writer.send(GoldChangeEvent {
            amount: total_value as i32,
            reason: format!("Shipping bin sold ({} items)", shipping_bin.items.len()),
        });

        info!(
            "[Economy] Shipping bin sold! Total earned: {}g. Sales: [{}]",
            total_value,
            sale_details.join(", ")
        );

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "day_end_coins".to_string(),
        });

        // Clear the bin for the next day.
        shipping_bin.items.clear();
    }
}

/// Returns the current estimated value of everything in the shipping bin.
/// Called by the UI to display a "pending earnings" preview.
pub fn calculate_bin_value(shipping_bin: &ShippingBin, item_registry: &ItemRegistry) -> u32 {
    shipping_bin
        .items
        .iter()
        .map(|slot| {
            let sell_price = item_registry
                .get(&slot.item_id)
                .map(|def| def.sell_price)
                .unwrap_or(1);
            sell_price.saturating_mul(slot.quantity as u32)
        })
        .fold(0u32, |acc, v| acc.saturating_add(v))
}

/// Resource that the UI can read to show the current bin value without having to call a function.
#[derive(Resource, Debug, Clone, Default)]
pub struct ShippingBinPreview {
    pub pending_value: u32,
    pub item_count: u32,
}

/// Updates ShippingBinPreview each frame so the UI always has fresh data.
pub fn update_shipping_bin_preview(
    shipping_bin: Res<ShippingBin>,
    item_registry: Res<ItemRegistry>,
    mut preview: ResMut<ShippingBinPreview>,
) {
    if shipping_bin.is_changed() || item_registry.is_changed() {
        preview.pending_value = calculate_bin_value(&shipping_bin, &item_registry);
        preview.item_count = shipping_bin
            .items
            .iter()
            .map(|s| s.quantity as u32)
            .sum();
    }
}
