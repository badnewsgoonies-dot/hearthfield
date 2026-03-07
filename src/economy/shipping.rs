use crate::economy::gold::EconomyStats;
use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// Events (internal)
// ─────────────────────────────────────────────────────────────────────────────

/// Fired by the player interaction system when they deposit an item in the shipping bin.
#[derive(Event, Debug, Clone)]
pub struct ShipItemEvent {
    pub item_id: ItemId,
    pub quantity: u8,
    /// Quality of the shipped item. Affects sell price multiplier.
    pub quality: ItemQuality,
}

/// Local resource that tracks item quality for each shipping bin slot.
/// Mirrors the indices of `ShippingBin.items` so quality can be looked up
/// at end-of-day settlement without modifying the frozen shared contract.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShippingBinQuality {
    /// Parallel to `ShippingBin.items` — each entry is (item_id, quality).
    /// When items merge by item_id in ShippingBin, we keep separate quality
    /// entries because different qualities have different multipliers.
    pub entries: Vec<(ItemId, u8, ItemQuality)>,
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
    mut bin_quality: ResMut<ShippingBinQuality>,
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

        // Track quality in the parallel quality resource.
        // Merge with existing entry of the same item_id + quality, or add new.
        let existing_q = bin_quality
            .entries
            .iter_mut()
            .find(|(id, _, q)| *id == ev.item_id && *q == ev.quality);
        if let Some(entry) = existing_q {
            entry.1 = entry.1.saturating_add(removed);
        } else {
            bin_quality
                .entries
                .push((ev.item_id.clone(), removed, ev.quality));
        }

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "shipping_bin".to_string(),
        });

        info!(
            "[Economy] Placed {} × '{}' ({:?}) in shipping bin. Bin now holds {} unique items.",
            removed,
            ev.item_id,
            ev.quality,
            shipping_bin.items.len()
        );

        // Suppress unused variable warning for item_def (we used it for validation above).
        let _ = item_def;
    }
}

/// Fires on DayEndEvent: sells everything in the shipping bin, adds gold, clears bin.
/// This is the primary income source for the player.
/// Quality multipliers (Silver 1.25x, Gold 1.5x, Iridium 2.0x) are applied via
/// the parallel `ShippingBinQuality` resource.
#[allow(clippy::too_many_arguments)]
pub fn process_shipping_bin_on_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut shipping_bin: ResMut<ShippingBin>,
    mut bin_quality: ResMut<ShippingBinQuality>,
    item_registry: Res<ItemRegistry>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut stats: ResMut<EconomyStats>,
    mut shipping_log: ResMut<ShippingLog>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        if shipping_bin.items.is_empty() {
            continue;
        }

        let mut total_value: u32 = 0;
        let mut items_shipped: u64 = 0;
        let mut sale_details: Vec<String> = Vec::new();

        // Use the quality-aware entries for pricing when available.
        // Each entry is (item_id, quantity, quality).
        for &(ref item_id, qty, quality) in bin_quality.entries.iter() {
            let sell_price = item_registry
                .get(item_id)
                .map(|def| def.sell_price)
                .unwrap_or(1); // fallback: 1g for unknown items

            // Apply quality multiplier: Normal 1.0, Silver 1.25, Gold 1.5, Iridium 2.0
            let quality_adjusted_price = (sell_price as f32 * quality.sell_multiplier()) as u32;
            let slot_value = quality_adjusted_price.saturating_mul(qty as u32);
            total_value = total_value.saturating_add(slot_value);
            items_shipped += qty as u64;

            // Record this item_id in the shipping log so evaluation can track unique items shipped.
            *shipping_log
                .shipped_items
                .entry(item_id.clone())
                .or_insert(0) += qty as u32;

            sale_details.push(format!(
                "{} × '{}' ({:?}) = {}g",
                qty, item_id, quality, slot_value
            ));
        }

        // Fall back: if bin_quality had no entries (e.g. from an older code path),
        // iterate the bin directly with Normal quality.
        if bin_quality.entries.is_empty() {
            for slot in shipping_bin.items.iter() {
                let sell_price = item_registry
                    .get(&slot.item_id)
                    .map(|def| def.sell_price)
                    .unwrap_or(1);
                let slot_value = sell_price.saturating_mul(slot.quantity as u32);
                total_value = total_value.saturating_add(slot_value);
                items_shipped += slot.quantity as u64;

                *shipping_log
                    .shipped_items
                    .entry(slot.item_id.clone())
                    .or_insert(0) += slot.quantity as u32;

                sale_details.push(format!(
                    "{} × '{}' = {}g",
                    slot.quantity, slot.item_id, slot_value
                ));
            }
        }

        stats.total_items_shipped = stats.total_items_shipped.saturating_add(items_shipped);

        // Emit GoldChangeEvent — apply_gold_changes will add the gold to PlayerState.
        gold_writer.send(GoldChangeEvent {
            amount: total_value as i32,
            reason: format!("Shipping bin sold ({} items)", shipping_bin.items.len()),
        });

        info!(
            "[Economy] Shipping bin sold! Total earned: {}g. Sales: [{}]",
            total_value,
            sale_details.join(", ")
        );

        // Notify the player of their earnings
        toast_writer.send(ToastEvent {
            message: format!(
                "Shipping: earned {}g from {} items",
                total_value, items_shipped
            ),
            duration_secs: 4.0,
        });

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "sell".to_string(),
        });

        // Clear the bin and quality tracking for the next day.
        shipping_bin.items.clear();
        bin_quality.entries.clear();
    }
}

/// Returns the current estimated value of everything in the shipping bin.
/// Called by the UI to display a "pending earnings" preview.
/// Uses the quality-aware entries from `ShippingBinQuality` when available.
pub fn calculate_bin_value(
    bin_quality: &ShippingBinQuality,
    shipping_bin: &ShippingBin,
    item_registry: &ItemRegistry,
) -> u32 {
    if !bin_quality.entries.is_empty() {
        bin_quality
            .entries
            .iter()
            .map(|(item_id, qty, quality)| {
                let sell_price = item_registry
                    .get(item_id)
                    .map(|def| def.sell_price)
                    .unwrap_or(1);
                let quality_adjusted = (sell_price as f32 * quality.sell_multiplier()) as u32;
                quality_adjusted.saturating_mul(*qty as u32)
            })
            .fold(0u32, |acc, v| acc.saturating_add(v))
    } else {
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
    bin_quality: Res<ShippingBinQuality>,
    item_registry: Res<ItemRegistry>,
    mut preview: ResMut<ShippingBinPreview>,
) {
    if shipping_bin.is_changed() || bin_quality.is_changed() || item_registry.is_changed() {
        preview.pending_value = calculate_bin_value(&bin_quality, &shipping_bin, &item_registry);
        preview.item_count = shipping_bin.items.iter().map(|s| s.quantity as u32).sum();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry_with(items: Vec<(&str, u32)>) -> ItemRegistry {
        let mut registry = ItemRegistry::default();
        for (id, price) in items {
            registry.items.insert(
                id.to_string(),
                ItemDef {
                    id: id.to_string(),
                    name: id.to_string(),
                    description: String::new(),
                    category: ItemCategory::Crop,
                    sell_price: price,
                    buy_price: None,
                    stack_size: 99,
                    edible: false,
                    energy_restore: 0.0,
                    sprite_index: 0,
                },
            );
        }
        registry
    }

    #[test]
    fn test_calculate_bin_value_empty() {
        let quality = ShippingBinQuality::default();
        let bin = ShippingBin::default();
        let registry = ItemRegistry::default();
        assert_eq!(calculate_bin_value(&quality, &bin, &registry), 0);
    }

    #[test]
    fn test_calculate_bin_value_single_item() {
        let registry = make_registry_with(vec![("turnip", 60)]);
        let bin = ShippingBin {
            items: vec![InventorySlot {
                item_id: "turnip".to_string(),
                quantity: 5,
            }],
        };
        let quality = ShippingBinQuality {
            entries: vec![("turnip".to_string(), 5, ItemQuality::Normal)],
        };
        assert_eq!(calculate_bin_value(&quality, &bin, &registry), 300); // 60 * 5
    }

    #[test]
    fn test_calculate_bin_value_multiple_items() {
        let registry = make_registry_with(vec![("turnip", 60), ("potato", 80)]);
        let bin = ShippingBin {
            items: vec![
                InventorySlot {
                    item_id: "turnip".to_string(),
                    quantity: 3,
                },
                InventorySlot {
                    item_id: "potato".to_string(),
                    quantity: 2,
                },
            ],
        };
        let quality = ShippingBinQuality {
            entries: vec![
                ("turnip".to_string(), 3, ItemQuality::Normal),
                ("potato".to_string(), 2, ItemQuality::Normal),
            ],
        };
        // turnip: 60*3=180, potato: 80*2=160 => 340
        assert_eq!(calculate_bin_value(&quality, &bin, &registry), 340);
    }

    #[test]
    fn test_calculate_bin_value_unknown_item_defaults_to_1g() {
        let registry = ItemRegistry::default(); // empty
        let bin = ShippingBin {
            items: vec![InventorySlot {
                item_id: "mystery_item".to_string(),
                quantity: 10,
            }],
        };
        // No quality entries — falls back to bin items with Normal quality
        let quality = ShippingBinQuality::default();
        assert_eq!(calculate_bin_value(&quality, &bin, &registry), 10);
    }

    #[test]
    fn test_calculate_bin_value_quality_multipliers() {
        let registry = make_registry_with(vec![("turnip", 100)]);
        let bin = ShippingBin {
            items: vec![InventorySlot {
                item_id: "turnip".to_string(),
                quantity: 4,
            }],
        };
        // 1 Normal (100), 1 Silver (125), 1 Gold (150), 1 Iridium (200)
        let quality = ShippingBinQuality {
            entries: vec![
                ("turnip".to_string(), 1, ItemQuality::Normal),
                ("turnip".to_string(), 1, ItemQuality::Silver),
                ("turnip".to_string(), 1, ItemQuality::Gold),
                ("turnip".to_string(), 1, ItemQuality::Iridium),
            ],
        };
        // 100 + 125 + 150 + 200 = 575
        assert_eq!(calculate_bin_value(&quality, &bin, &registry), 575);
    }

    #[test]
    fn test_shipping_bin_preview_default() {
        let preview = ShippingBinPreview::default();
        assert_eq!(preview.pending_value, 0);
        assert_eq!(preview.item_count, 0);
    }
}
