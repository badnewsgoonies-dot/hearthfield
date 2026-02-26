use bevy::prelude::*;
use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// Resources
// ─────────────────────────────────────────────────────────────────────────────

/// Which shop the player is currently visiting, and its filtered active listings.
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveShop {
    pub shop_id: Option<ShopId>,
    /// Listings filtered for the current season, ready for the UI to display.
    pub listings: Vec<ActiveListing>,
}

/// A single entry in the current shop, enriched with item info for the UI.
#[derive(Debug, Clone)]
pub struct ActiveListing {
    pub item_id: ItemId,
    pub display_name: String,
    pub price: u32,
    pub sell_price: u32,   // what the player would receive if they sell it back
    pub sprite_index: u32,
    pub can_afford: bool,  // cached against current gold — UI re-reads per frame
}

// ─────────────────────────────────────────────────────────────────────────────
// Events (internal — used to drive transactions from UI input)
// ─────────────────────────────────────────────────────────────────────────────

/// Fired by the UI when the player confirms a purchase.
#[derive(Event, Debug, Clone)]
pub struct BuyRequestEvent {
    pub item_id: ItemId,
    pub quantity: u8,
}

/// Fired by the UI when the player confirms selling an item from inventory.
#[derive(Event, Debug, Clone)]
pub struct SellRequestEvent {
    pub item_id: ItemId,
    pub quantity: u8,
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Detects MapTransitionEvents for shop maps and:
///   1. Transitions GameState to GameState::Shop
///   2. Populates ActiveShop with season-filtered listings
pub fn on_enter_shop(
    mut map_events: EventReader<MapTransitionEvent>,
    shop_data: Res<ShopData>,
    item_registry: Res<ItemRegistry>,
    player_state: Res<PlayerState>,
    calendar: Res<Calendar>,
    mut active_shop: ResMut<ActiveShop>,
    mut next_state: ResMut<NextState<GameState>>,
    current_state: Res<State<GameState>>,
) {
    for ev in map_events.read() {
        let shop_id = match ev.to_map {
            MapId::GeneralStore => ShopId::GeneralStore,
            MapId::AnimalShop => ShopId::AnimalShop,
            MapId::Blacksmith => ShopId::Blacksmith,
            _ => continue,
        };

        // Only transition if we are currently Playing (avoid re-triggering from within Shop).
        if *current_state.get() == GameState::Playing {
            next_state.set(GameState::Shop);
            info!("[Economy] Entering shop: {:?}", shop_id);
        }

        // Build the filtered listing set.
        let listings = build_listings(
            shop_id,
            &shop_data,
            &item_registry,
            player_state.gold,
            calendar.season,
        );

        *active_shop = ActiveShop {
            shop_id: Some(shop_id),
            listings,
        };
    }
}

/// Refreshes the `can_afford` flag each frame while in the shop.
/// This is cheap and keeps the UI honest without event overhead.
pub fn refresh_shop_affordability(
    player_state: Res<PlayerState>,
    mut active_shop: ResMut<ActiveShop>,
) {
    if active_shop.shop_id.is_none() {
        return;
    }
    let gold = player_state.gold;
    for listing in active_shop.listings.iter_mut() {
        listing.can_afford = gold >= listing.price;
    }
}

/// Processes BuyRequestEvents — the core purchase flow.
pub fn handle_buy(
    mut buy_events: EventReader<BuyRequestEvent>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    active_shop: Res<ActiveShop>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut transaction_writer: EventWriter<ShopTransactionEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    let shop_id = match active_shop.shop_id {
        Some(id) => id,
        None => return,
    };

    for ev in buy_events.read() {
        // Validate item exists in registry.
        let item_def = match item_registry.get(&ev.item_id) {
            Some(def) => def,
            None => {
                warn!("[Economy] Buy failed — unknown item '{}'", ev.item_id);
                continue;
            }
        };

        // Find price from active listing (not from item_def.buy_price directly,
        // so shop-specific overrides work).
        let price_per_unit = match active_shop
            .listings
            .iter()
            .find(|l| l.item_id == ev.item_id)
        {
            Some(listing) => listing.price,
            None => {
                warn!(
                    "[Economy] Buy failed — '{}' not in current shop listing",
                    ev.item_id
                );
                continue;
            }
        };

        let quantity = ev.quantity.max(1);
        let total_cost = price_per_unit.saturating_mul(quantity as u32);

        // Check affordability.
        if player_state.gold < total_cost {
            info!(
                "[Economy] Cannot afford {} × '{}' (need {}g, have {}g)",
                quantity, ev.item_id, total_cost, player_state.gold
            );
            sfx_writer.send(PlaySfxEvent {
                sfx_id: "ui_deny".to_string(),
            });
            continue;
        }

        // Check inventory space.
        let leftover = {
            // We do a dry-run using a clone — the real add happens below.
            let mut inv_clone = inventory.clone();
            inv_clone.try_add(&ev.item_id, quantity, item_def.stack_size)
        };
        if leftover > 0 {
            info!(
                "[Economy] Not enough inventory space to buy {} × '{}'",
                quantity, ev.item_id
            );
            sfx_writer.send(PlaySfxEvent {
                sfx_id: "ui_deny".to_string(),
            });
            continue;
        }

        // All checks passed — commit the transaction.
        player_state.gold -= total_cost;
        inventory.try_add(&ev.item_id, quantity, item_def.stack_size);

        gold_writer.send(GoldChangeEvent {
            // We already deducted gold manually above; this event is for tracking & UI only.
            // Send 0 here to avoid double-deduction — the actual deduction is above.
            // NOTE: Gold is already applied directly; the event is purely informational.
            amount: -(total_cost as i32),
            reason: format!("Bought {} × {}", quantity, item_def.name),
        });

        pickup_writer.send(ItemPickupEvent {
            item_id: ev.item_id.clone(),
            quantity,
        });

        transaction_writer.send(ShopTransactionEvent {
            shop_id,
            item_id: ev.item_id.clone(),
            quantity,
            total_cost,
            is_purchase: true,
        });

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "shop_buy".to_string(),
        });

        info!(
            "[Economy] Bought {} × '{}' for {}g. Remaining gold: {}g",
            quantity, ev.item_id, total_cost, player_state.gold
        );
    }
}

/// Processes SellRequestEvents — selling inventory items for gold.
pub fn handle_sell(
    mut sell_events: EventReader<SellRequestEvent>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    active_shop: Res<ActiveShop>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut removed_writer: EventWriter<ItemRemovedEvent>,
    mut transaction_writer: EventWriter<ShopTransactionEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    let shop_id = match active_shop.shop_id {
        Some(id) => id,
        None => return,
    };

    for ev in sell_events.read() {
        let item_def = match item_registry.get(&ev.item_id) {
            Some(def) => def,
            None => {
                warn!("[Economy] Sell failed — unknown item '{}'", ev.item_id);
                continue;
            }
        };

        let quantity = ev.quantity.max(1);

        // Verify the player actually has enough of the item.
        if !inventory.has(&ev.item_id, quantity) {
            warn!(
                "[Economy] Sell failed — not enough '{}' in inventory (have {}, want {})",
                ev.item_id,
                inventory.count(&ev.item_id),
                quantity
            );
            continue;
        }

        // Shops in Hearthfield buy items at their base sell_price.
        let price_per_unit = item_def.sell_price;
        let total_earned = price_per_unit.saturating_mul(quantity as u32);

        // Commit.
        let removed = inventory.try_remove(&ev.item_id, quantity);
        if removed < quantity {
            warn!(
                "[Economy] Partial sell: only removed {} of {}",
                removed, quantity
            );
        }

        player_state.gold = player_state.gold.saturating_add(total_earned);

        gold_writer.send(GoldChangeEvent {
            amount: total_earned as i32,
            reason: format!("Sold {} × {}", removed, item_def.name),
        });

        removed_writer.send(ItemRemovedEvent {
            item_id: ev.item_id.clone(),
            quantity: removed,
        });

        transaction_writer.send(ShopTransactionEvent {
            shop_id,
            item_id: ev.item_id.clone(),
            quantity: removed,
            total_cost: total_earned,
            is_purchase: false,
        });

        sfx_writer.send(PlaySfxEvent {
            sfx_id: "shop_sell".to_string(),
        });

        info!(
            "[Economy] Sold {} × '{}' for {}g. New balance: {}g",
            removed, ev.item_id, total_earned, player_state.gold
        );
    }
}

/// Called when the player exits the shop (e.g., walks out, presses Escape).
/// Returns game state to Playing and clears the active shop.
pub fn on_exit_shop(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_shop: ResMut<ActiveShop>,
) {
    if *state.get() != GameState::Shop {
        return;
    }
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Playing);
        active_shop.shop_id = None;
        active_shop.listings.clear();
        info!("[Economy] Left shop, returning to Playing state.");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn build_listings(
    shop_id: ShopId,
    shop_data: &ShopData,
    item_registry: &ItemRegistry,
    player_gold: u32,
    current_season: Season,
) -> Vec<ActiveListing> {
    let raw_listings = match shop_data.listings.get(&shop_id) {
        Some(l) => l,
        None => return Vec::new(),
    };

    raw_listings
        .iter()
        .filter(|listing| {
            // Keep items that are always available OR available this season.
            listing
                .season_available
                .map_or(true, |s| s == current_season)
        })
        .filter_map(|listing| {
            let def = item_registry.get(&listing.item_id)?;
            Some(ActiveListing {
                item_id: listing.item_id.clone(),
                display_name: def.name.clone(),
                price: listing.price,
                sell_price: def.sell_price,
                sprite_index: def.sprite_index,
                can_afford: player_gold >= listing.price,
            })
        })
        .collect()
}
