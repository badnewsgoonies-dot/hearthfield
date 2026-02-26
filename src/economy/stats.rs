use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// HARVEST STATS — tracks crop harvest counts and revenue
// ═══════════════════════════════════════════════════════════════════════

/// Accumulated statistics about crop harvests.
/// Key = crop_id, Value = (total_harvested_count, total_revenue_gold).
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct HarvestStats {
    pub crops: HashMap<String, (u32, u32)>,
}

// ═══════════════════════════════════════════════════════════════════════
// ANIMAL PRODUCT STATS — tracks animal product totals
// ═══════════════════════════════════════════════════════════════════════

/// Accumulated statistics about animal products collected.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimalProductStats {
    pub total_eggs: u32,
    pub total_milk: u32,
    pub total_wool: u32,
    pub total_other: u32,
    pub total_revenue: u32,
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Reads `CropHarvestedEvent` and updates `HarvestStats`.
///
/// Each event increments the count for that crop_id by the harvested quantity.
/// If the event carries a quality level, it is logged for debugging.
pub fn track_crop_harvests(
    mut events: EventReader<CropHarvestedEvent>,
    mut stats: ResMut<HarvestStats>,
    item_registry: Res<ItemRegistry>,
) {
    for ev in events.read() {
        let entry = stats.crops.entry(ev.crop_id.clone()).or_insert((0, 0));

        // Increment harvested count by the quantity from the event
        entry.0 = entry.0.saturating_add(ev.quantity as u32);

        // Look up the sell price from the item registry, applying quality multiplier
        let base_price = item_registry
            .get(&ev.harvest_id)
            .map(|def| def.sell_price)
            .unwrap_or(0);

        let quality_multiplier = ev
            .quality
            .unwrap_or(ItemQuality::Normal)
            .sell_multiplier();

        let revenue = (base_price as f32 * quality_multiplier * ev.quantity as f32) as u32;
        entry.1 = entry.1.saturating_add(revenue);

        // Log quality if present
        if let Some(quality) = &ev.quality {
            info!(
                "[Economy/Stats] Harvested {}x {} (quality: {:?}) at ({}, {}). Revenue estimate: {}g",
                ev.quantity, ev.crop_id, quality, ev.x, ev.y, revenue
            );
        } else {
            info!(
                "[Economy/Stats] Harvested {}x {} at ({}, {}). Revenue estimate: {}g",
                ev.quantity, ev.crop_id, ev.x, ev.y, revenue
            );
        }
    }
}

/// Reads `AnimalProductEvent` and updates `AnimalProductStats`.
///
/// Products are categorized by checking if the product_id contains
/// "egg", "milk", "wool", or falls through to "other".
pub fn track_animal_products(
    mut events: EventReader<AnimalProductEvent>,
    mut stats: ResMut<AnimalProductStats>,
    item_registry: Res<ItemRegistry>,
) {
    for ev in events.read() {
        let product_lower = ev.product_id.to_lowercase();

        if product_lower.contains("egg") {
            stats.total_eggs = stats.total_eggs.saturating_add(1);
        } else if product_lower.contains("milk") {
            stats.total_milk = stats.total_milk.saturating_add(1);
        } else if product_lower.contains("wool") {
            stats.total_wool = stats.total_wool.saturating_add(1);
        } else {
            stats.total_other = stats.total_other.saturating_add(1);
        }

        // Track estimated revenue from sell price
        let sell_price = item_registry
            .get(&ev.product_id)
            .map(|def| def.sell_price)
            .unwrap_or(0);
        stats.total_revenue = stats.total_revenue.saturating_add(sell_price);

        info!(
            "[Economy/Stats] Animal product: {} from {:?}. Estimated value: {}g",
            ev.product_id, ev.animal_kind, sell_price
        );
    }
}
