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
pub mod stats;
pub mod tool_upgrades;
pub mod evaluation;
pub mod play_stats;
pub mod achievements;
pub mod buildings;

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
    handle_upgrade_request, tick_upgrade_queue, drain_upgrade_complete,
};
use stats::{HarvestStats, AnimalProductStats, track_crop_harvests, track_animal_products};
use evaluation::{check_evaluation_trigger, handle_evaluation};
use achievements::{check_achievements, notify_achievement_unlocked, track_achievement_progress};
use play_stats::{
    track_crops_harvested, track_fish_caught, track_day_end,
    track_gifts_given, track_animals_petted, track_gold_earned, track_recipes_cooked,
};
use buildings::{BuildingLevels, handle_building_upgrade_request, tick_building_upgrade};

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
            .init_resource::<ToolUpgradeQueue>()
            .init_resource::<HarvestStats>()
            .init_resource::<AnimalProductStats>()
            .init_resource::<BuildingLevels>();

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
                // Drain ToolUpgradeCompleteEvent to prevent "event not read" warnings.
                drain_upgrade_complete,
                // Harvest and animal product stat tracking.
                track_crop_harvests,
                track_animal_products,
                // Year-end evaluation: check trigger condition, then score.
                check_evaluation_trigger,
                handle_evaluation,
                // PlayStats counters — passive listeners for global play statistics.
                track_crops_harvested,
                track_fish_caught,
                track_day_end,
                track_gifts_given,
                track_animals_petted,
                track_gold_earned,
                track_recipes_cooked,
                // Achievement progress counters (rocks broken, crops planted, gold-quality crops).
                track_achievement_progress,
                // Achievement condition checks — fires AchievementUnlockedEvent when earned.
                check_achievements,
                // Display toast notifications for newly unlocked achievements.
                notify_achievement_unlocked,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ── Systems: Building upgrades (Playing state) ─────────────────────
        app.add_systems(
            Update,
            (
                handle_building_upgrade_request,
                tick_building_upgrade,
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
