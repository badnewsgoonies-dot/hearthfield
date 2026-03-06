//! Economy domain — shops, transactions, shipping bin, tool upgrades, gold tracking.
//!
//! All cross-domain communication goes through `crate::shared::*` events and resources.
//! No other domain module is imported here.

use crate::shared::*;
use bevy::prelude::*;

pub mod achievements;
pub mod blacksmith;
pub mod buildings;
pub mod evaluation;
pub mod gold;
pub mod play_stats;
pub mod shipping;
pub mod shop;
pub mod stats;
pub mod tool_upgrades;

use achievements::{check_achievements, notify_achievement_unlocked, track_achievement_progress};
use blacksmith::{
    drain_upgrade_complete, handle_upgrade_request, tick_upgrade_queue, ToolUpgradeCompleteEvent,
    ToolUpgradeQueue, ToolUpgradeRequestEvent,
};
use buildings::{handle_building_upgrade_request, tick_building_upgrade, BuildingLevels};
use evaluation::{check_evaluation_trigger, handle_evaluation};
use gold::{apply_gold_changes, EconomyStats};
use play_stats::{
    track_animal_products_collected, track_crops_harvested, track_day_end, track_fish_caught,
    track_food_eaten, track_gifts_given, track_gold_earned,
};
use shipping::{
    place_in_shipping_bin, process_shipping_bin_on_day_end, update_shipping_bin_preview,
    ShipItemEvent, ShippingBinPreview, ShippingBinQuality,
};
use shop::{
    handle_shop_transaction_gold, on_enter_shop, on_exit_shop, refresh_shop_affordability,
    ActiveShop,
};
use stats::{track_animal_products, track_crop_harvests, AnimalProductStats, HarvestStats};

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
            .init_resource::<BuildingLevels>()
            .init_resource::<ShippingBinQuality>();

        // ── Internal Events ────────────────────────────────────────────────
        app.add_event::<ShipItemEvent>()
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
                track_animal_products_collected,
                track_gold_earned,
                track_food_eaten,
                // Achievement progress counters (rocks broken, crops planted, gold-quality crops).
                track_achievement_progress,
            )
                .run_if(in_state(GameState::Playing)),
        );

        // ── Achievement systems (separate group to stay under Bevy tuple limit) ──
        app.add_systems(
            Update,
            (
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
            (handle_building_upgrade_request, tick_building_upgrade)
                .run_if(in_state(GameState::Playing)),
        );

        // ── Systems: Shop state ────────────────────────────────────────────
        app.add_systems(
            Update,
            (
                // Keep affordability flags fresh each frame.
                refresh_shop_affordability,
                // Tool upgrades are only requested from the Blacksmith shop.
                handle_upgrade_request,
                // Allow exiting the shop with Escape.
                on_exit_shop,
                // Gold changes can also arrive while in the shop
                // (e.g., from another concurrent event — keep it consistent).
                apply_gold_changes,
                // Track buy/sell transactions in EconomyStats and PlayStats via GoldChangeEvent.
                handle_shop_transaction_gold,
            )
                .run_if(in_state(GameState::Shop)),
        );

        info!("[Economy] EconomyPlugin registered.");
    }
}
