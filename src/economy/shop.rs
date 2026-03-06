use crate::shared::*;
use bevy::prelude::*;

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
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ActiveListing {
    pub item_id: ItemId,
    pub display_name: String,
    pub price: u32,
    pub sell_price: u32, // what the player would receive if they sell it back
    pub sprite_index: u32,
    pub can_afford: bool, // cached against current gold — UI re-reads per frame
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Detects MapTransitionEvents for shop maps and:
///   1. Transitions GameState to GameState::Shop
///   2. Populates ActiveShop with season-filtered listings
#[allow(clippy::too_many_arguments)]
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

/// Called when the player exits the shop (e.g., walks out, presses Escape).
/// Returns game state to Playing and clears the active shop.
pub fn on_exit_shop(
    player_input: Res<PlayerInput>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_shop: ResMut<ActiveShop>,
) {
    if *state.get() != GameState::Shop {
        return;
    }
    if player_input.ui_cancel {
        next_state.set(GameState::Playing);
        active_shop.shop_id = None;
        active_shop.listings.clear();
        info!("[Economy] Left shop, returning to Playing state.");
    }
}

/// Updates EconomyStats directly for shop transactions without sending
/// GoldChangeEvent. The shop UI already mutates player.gold directly for
/// immediate feedback; sending GoldChangeEvent would cause apply_gold_changes
/// to modify gold a second time (double deduction bug).
pub fn handle_shop_transaction_gold(
    mut tx_events: EventReader<ShopTransactionEvent>,
    mut stats: ResMut<super::gold::EconomyStats>,
) {
    for ev in tx_events.read() {
        if ev.is_purchase {
            stats.total_gold_spent = stats.total_gold_spent.saturating_add(ev.total_cost as u64);
            info!(
                "[Economy] Shop buy stats: {} for {}g. Total spent: {}g",
                ev.item_id, ev.total_cost, stats.total_gold_spent
            );
        } else {
            stats.total_gold_earned = stats.total_gold_earned.saturating_add(ev.total_cost as u64);
            info!(
                "[Economy] Shop sell stats: {} for {}g. Total earned: {}g",
                ev.item_id, ev.total_cost, stats.total_gold_earned
            );
        }
        stats.total_transactions += 1;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Buy / Sell Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Result of a shop transaction attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionResult {
    /// Transaction succeeded. Contains the total cost (positive) or revenue (positive).
    Success { total: u32 },
    /// Player cannot afford the purchase.
    InsufficientGold { need: u32, have: u32 },
    /// Inventory is full — cannot add the purchased item.
    InventoryFull,
    /// Player does not have enough of the item to sell.
    InsufficientItems { need: u8, have: u8 },
    /// Item not found in the item registry.
    UnknownItem,
}

/// Attempts to buy `quantity` of `item_id` at `price_per_unit` from a shop.
///
/// On success:
///   - Deducts gold from `PlayerState`
///   - Adds the item to `Inventory`
///   - Fires a `ShopTransactionEvent` (is_purchase = true)
///   - Fires a `GoldChangeEvent` (negative)
///
/// Returns a `TransactionResult` describing success or failure reason.
///
/// Note: This function mutates state directly for immediate UI feedback.
/// The `GoldChangeEvent` is NOT sent (to avoid double-deduction — see
/// `handle_shop_transaction_gold` which tracks stats separately).
#[allow(dead_code)]
pub fn try_buy(
    item_id: &str,
    quantity: u8,
    price_per_unit: u32,
    player_state: &mut PlayerState,
    inventory: &mut Inventory,
    item_registry: &ItemRegistry,
) -> TransactionResult {
    // Validate item exists.
    let item_def = match item_registry.get(item_id) {
        Some(def) => def,
        None => return TransactionResult::UnknownItem,
    };

    let total_cost = price_per_unit.saturating_mul(quantity as u32);

    // Check gold.
    if player_state.gold < total_cost {
        return TransactionResult::InsufficientGold {
            need: total_cost,
            have: player_state.gold,
        };
    }

    // Check inventory capacity.
    let added = inventory.try_add(item_id, quantity, item_def.stack_size);
    if added == 0 {
        return TransactionResult::InventoryFull;
    }

    // Deduct gold.
    player_state.gold = player_state.gold.saturating_sub(total_cost);

    TransactionResult::Success { total: total_cost }
}

/// Attempts to sell `quantity` of `item_id` from the player's inventory.
///
/// On success:
///   - Removes the item from `Inventory`
///   - Adds gold to `PlayerState`
///
/// The sell price comes from the item's `sell_price` in the registry, modified
/// by the optional quality multiplier.
///
/// Returns a `TransactionResult` describing success or failure reason.
#[allow(dead_code)]
pub fn try_sell(
    item_id: &str,
    quantity: u8,
    quality: crate::shared::ItemQuality,
    player_state: &mut PlayerState,
    inventory: &mut Inventory,
    item_registry: &ItemRegistry,
) -> TransactionResult {
    // Validate item exists.
    let item_def = match item_registry.get(item_id) {
        Some(def) => def,
        None => return TransactionResult::UnknownItem,
    };

    // Check inventory.
    let held = inventory.count(item_id);
    if held < quantity {
        return TransactionResult::InsufficientItems {
            need: quantity,
            have: held,
        };
    }

    let quality_adjusted = (item_def.sell_price as f32 * quality.sell_multiplier()) as u32;
    let total_revenue = quality_adjusted.saturating_mul(quantity as u32);

    // Remove from inventory.
    inventory.try_remove(item_id, quantity);

    // Add gold.
    player_state.gold = player_state.gold.saturating_add(total_revenue);

    TransactionResult::Success {
        total: total_revenue,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Listing Helpers
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
            listing.season_available.is_none_or(|s| s == current_season)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry(items: &[(&str, u32, u32)]) -> ItemRegistry {
        let mut registry = ItemRegistry::default();
        for &(id, sell_price, stack) in items {
            registry.items.insert(
                id.to_string(),
                ItemDef {
                    id: id.to_string(),
                    name: id.to_string(),
                    description: String::new(),
                    category: ItemCategory::Crop,
                    sell_price,
                    buy_price: None,
                    stack_size: stack as u8,
                    edible: false,
                    energy_restore: 0.0,
                    sprite_index: 0,
                },
            );
        }
        registry
    }

    fn default_player(gold: u32) -> PlayerState {
        let mut ps = PlayerState::default();
        ps.gold = gold;
        ps
    }

    #[test]
    fn test_try_buy_success() {
        let registry = make_registry(&[("seeds", 20, 99)]);
        let mut player = default_player(500);
        let mut inv = Inventory::default();

        let result = try_buy("seeds", 5, 20, &mut player, &mut inv, &registry);
        assert_eq!(result, TransactionResult::Success { total: 100 });
        assert_eq!(player.gold, 400);
        assert_eq!(inv.count("seeds"), 5);
    }

    #[test]
    fn test_try_buy_insufficient_gold() {
        let registry = make_registry(&[("seeds", 20, 99)]);
        let mut player = default_player(50);
        let mut inv = Inventory::default();

        let result = try_buy("seeds", 5, 20, &mut player, &mut inv, &registry);
        assert_eq!(
            result,
            TransactionResult::InsufficientGold {
                need: 100,
                have: 50
            }
        );
        assert_eq!(player.gold, 50); // unchanged
    }

    #[test]
    fn test_try_buy_unknown_item() {
        let registry = ItemRegistry::default();
        let mut player = default_player(500);
        let mut inv = Inventory::default();

        let result = try_buy("nonexistent", 1, 10, &mut player, &mut inv, &registry);
        assert_eq!(result, TransactionResult::UnknownItem);
    }

    #[test]
    fn test_try_sell_success() {
        let registry = make_registry(&[("turnip", 60, 99)]);
        let mut player = default_player(100);
        let mut inv = Inventory::default();
        inv.try_add("turnip", 10, 99);

        let result = try_sell(
            "turnip",
            3,
            ItemQuality::Normal,
            &mut player,
            &mut inv,
            &registry,
        );
        assert_eq!(result, TransactionResult::Success { total: 180 });
        assert_eq!(player.gold, 280);
        assert_eq!(inv.count("turnip"), 7);
    }

    #[test]
    fn test_try_sell_quality_multiplier() {
        let registry = make_registry(&[("turnip", 100, 99)]);
        let mut player = default_player(0);
        let mut inv = Inventory::default();
        inv.try_add("turnip", 2, 99);

        // Gold quality = 1.5x multiplier
        let result = try_sell(
            "turnip",
            1,
            ItemQuality::Gold,
            &mut player,
            &mut inv,
            &registry,
        );
        assert_eq!(result, TransactionResult::Success { total: 150 });
        assert_eq!(player.gold, 150);
    }

    #[test]
    fn test_try_sell_iridium_quality() {
        let registry = make_registry(&[("turnip", 100, 99)]);
        let mut player = default_player(0);
        let mut inv = Inventory::default();
        inv.try_add("turnip", 1, 99);

        // Iridium quality = 2.0x
        let result = try_sell(
            "turnip",
            1,
            ItemQuality::Iridium,
            &mut player,
            &mut inv,
            &registry,
        );
        assert_eq!(result, TransactionResult::Success { total: 200 });
    }

    #[test]
    fn test_try_sell_insufficient_items() {
        let registry = make_registry(&[("turnip", 60, 99)]);
        let mut player = default_player(100);
        let mut inv = Inventory::default();
        inv.try_add("turnip", 2, 99);

        let result = try_sell(
            "turnip",
            5,
            ItemQuality::Normal,
            &mut player,
            &mut inv,
            &registry,
        );
        assert_eq!(
            result,
            TransactionResult::InsufficientItems { need: 5, have: 2 }
        );
        assert_eq!(player.gold, 100); // unchanged
    }

    #[test]
    fn test_try_sell_unknown_item() {
        let registry = ItemRegistry::default();
        let mut player = default_player(100);
        let mut inv = Inventory::default();

        let result = try_sell(
            "ghost_item",
            1,
            ItemQuality::Normal,
            &mut player,
            &mut inv,
            &registry,
        );
        assert_eq!(result, TransactionResult::UnknownItem);
    }

    #[test]
    fn test_active_listing_can_afford() {
        let listing = ActiveListing {
            item_id: "seeds".to_string(),
            display_name: "Seeds".to_string(),
            price: 100,
            sell_price: 50,
            sprite_index: 0,
            can_afford: true,
        };
        assert!(listing.can_afford);
    }

    #[test]
    fn test_active_shop_default() {
        let shop = ActiveShop::default();
        assert!(shop.shop_id.is_none());
        assert!(shop.listings.is_empty());
    }
}
