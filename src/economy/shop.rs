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

/// Converts ShopTransactionEvents into GoldChangeEvents so that
/// EconomyStats and PlayStats correctly track gold spent/earned in shops.
/// The shop UI already mutates player.gold directly; this fires the event
/// alongside that mutation purely for stats tracking.
pub fn handle_shop_transaction_gold(
    mut tx_events: EventReader<ShopTransactionEvent>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
) {
    for ev in tx_events.read() {
        if ev.is_purchase {
            gold_writer.send(GoldChangeEvent {
                amount: -(ev.total_cost as i32),
                reason: format!("shop buy: {}", ev.item_id),
            });
        } else {
            gold_writer.send(GoldChangeEvent {
                amount: ev.total_cost as i32,
                reason: format!("shop sell: {}", ev.item_id),
            });
        }
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
