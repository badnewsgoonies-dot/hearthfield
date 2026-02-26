//! Economy domain — shops, transactions, shipping bin, tool upgrades, gold tracking.
//!
//! All cross-domain communication goes through `crate::shared::*` events and resources.
//! No other domain module is imported here.

use bevy::prelude::*;
use crate::shared::*;

pub mod gold;
pub mod shop;
pub mod shipping;
pub mod blacksmith;

use gold::{apply_gold_changes, EconomyStats};
use shop::{
    ActiveShop, BuyRequestEvent, SellRequestEvent,
    handle_buy, handle_sell, on_enter_shop, on_exit_shop, refresh_shop_affordability,
};
use shipping::{
    ShipItemEvent, ShippingBinPreview,
    place_in_shipping_bin, process_shipping_bin_on_day_end, update_shipping_bin_preview,
};
use blacksmith::{
    ToolUpgradeQueue, ToolUpgradeRequestEvent, ToolUpgradeCompleteEvent,
    handle_upgrade_request, tick_upgrade_queue,
};

// ─────────────────────────────────────────────────────────────────────────────
// Plugin
// ─────────────────────────────────────────────────────────────────────────────

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        // ── Resources ──────────────────────────────────────────────────────
        app.init_resource::<EconomyStats>()
            .init_resource::<ActiveShop>()
            .init_resource::<ShippingBinPreview>()
            .init_resource::<ToolUpgradeQueue>();

        // ── Internal Events ────────────────────────────────────────────────
        app.add_event::<BuyRequestEvent>()
            .add_event::<SellRequestEvent>()
            .add_event::<ShipItemEvent>()
            .add_event::<ToolUpgradeRequestEvent>()
            .add_event::<ToolUpgradeCompleteEvent>();

        // ── Systems: Playing state ─────────────────────────────────────────
        app.add_systems(
            Update,
            (
                // Listen for map transitions that enter a shop.
                on_enter_shop,
                // Shipping bin interactions only make sense while exploring.
                place_in_shipping_bin,
                // Update the pending-value preview for the HUD.
                update_shipping_bin_preview,
                // Gold change events can arrive from any domain at any time.
                apply_gold_changes,
                // Day-end: process the shipping bin sell-through.
                process_shipping_bin_on_day_end,
                // Day-end: tick tool upgrade timers.
                tick_upgrade_queue,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ── Systems: Shop state ────────────────────────────────────────────
        app.add_systems(
            Update,
            (
                // Keep affordability flags fresh each frame.
                refresh_shop_affordability,
                // Process buy/sell requests from the UI.
                handle_buy,
                handle_sell,
                // Tool upgrades are only requested from the Blacksmith shop.
                handle_upgrade_request,
                // Allow exiting the shop with Escape.
                on_exit_shop,
                // Gold changes can also arrive while in the shop
                // (e.g., from another concurrent event — keep it consistent).
                apply_gold_changes,
            )
                .run_if(in_state(GameState::Shop)),
        );

        info!("[Economy] EconomyPlugin registered.");
    }
}
