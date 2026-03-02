use bevy::prelude::*;
use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// Constants — material requirements per tier upgrade
// ─────────────────────────────────────────────────────────────────────────────

/// The item ID for the bar that must be consumed to reach each tier.
/// Basic → Copper requires 5 copper bars, etc.
fn required_bars_for_tier(target_tier: ToolTier) -> Option<(&'static str, u8)> {
    match target_tier {
        ToolTier::Basic => None, // Already the starting point; no upgrade into Basic.
        ToolTier::Copper => Some(("copper_bar", 5)),
        ToolTier::Iron => Some(("iron_bar", 5)),
        ToolTier::Gold => Some(("gold_bar", 5)),
        ToolTier::Iridium => Some(("iridium_bar", 5)),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resources
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks tools that are currently being upgraded (they cannot be used during this time).
#[derive(Resource, Debug, Clone, Default)]
pub struct ToolUpgradeQueue {
    pub pending: Vec<PendingUpgrade>,
}

#[derive(Debug, Clone)]
pub struct PendingUpgrade {
    pub tool: ToolKind,
    pub target_tier: ToolTier,
    /// How many full days remain until the upgrade is complete.
    pub days_remaining: u8,
}

impl ToolUpgradeQueue {
    /// Returns true if the given tool is currently being upgraded (unavailable).
    pub fn is_upgrading(&self, tool: ToolKind) -> bool {
        self.pending.iter().any(|p| p.tool == tool)
    }

    /// Returns a human-readable status for a tool (used by UI).
    #[allow(dead_code)]
    pub fn upgrade_status(&self, tool: ToolKind) -> Option<String> {
        self.pending
            .iter()
            .find(|p| p.tool == tool)
            .map(|p| {
                if p.days_remaining == 1 {
                    format!("{:?} upgrade ready tomorrow!", tool)
                } else {
                    format!("{:?} upgrade: {} days remaining", tool, p.days_remaining)
                }
            })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Events (internal)
// ─────────────────────────────────────────────────────────────────────────────

/// Fired by the UI / shop when the player requests a tool upgrade.
#[derive(Event, Debug, Clone)]
pub struct ToolUpgradeRequestEvent {
    pub tool: ToolKind,
}

/// Fired when an upgrade completes (for UI notification).
#[derive(Event, Debug, Clone)]
pub struct ToolUpgradeCompleteEvent {
    pub tool: ToolKind,
    pub new_tier: ToolTier,
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Drains `ToolUpgradeCompleteEvent` to prevent Bevy "event not read" warnings.
/// The sender (`tick_upgrade_queue`) already fires a `ToastEvent` and an SFX
/// event for player feedback; this handler ensures the event queue is cleared.
pub fn drain_upgrade_complete(mut events: EventReader<ToolUpgradeCompleteEvent>) {
    for _event in events.read() {}
}

/// Handles ToolUpgradeRequestEvents from the shop UI.
///
/// Validates:
///   - Player is in the Blacksmith
///   - Tool is not already in the upgrade queue
///   - Tool has a next tier (not already Iridium)
///   - Player has enough gold (upgrade_cost of the *target* tier)
///   - Player has the required bars in inventory
///
/// On success:
///   - Deducts gold and bars
///   - Adds to ToolUpgradeQueue (2-day timer)
pub fn handle_upgrade_request(
    mut upgrade_events: EventReader<ToolUpgradeRequestEvent>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    mut upgrade_queue: ResMut<ToolUpgradeQueue>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut removed_writer: EventWriter<ItemRemovedEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    active_shop: Res<crate::economy::shop::ActiveShop>,
) {
    // Only process upgrades while the player is in the Blacksmith.
    if active_shop.shop_id != Some(ShopId::Blacksmith) {
        // Drain the reader to avoid log spam.
        for _ in upgrade_events.read() {}
        return;
    }

    for ev in upgrade_events.read() {
        let current_tier = match player_state.tools.get(&ev.tool) {
            Some(t) => *t,
            None => {
                warn!("[Economy] Upgrade requested for unrecognised tool {:?}", ev.tool);
                continue;
            }
        };

        // Already upgrading?
        if upgrade_queue.is_upgrading(ev.tool) {
            info!(
                "[Economy] {:?} is already in the upgrade queue.",
                ev.tool
            );
            sfx_writer.send(PlaySfxEvent { sfx_id: "ui_deny".to_string() });
            continue;
        }

        // Determine target tier.
        let target_tier = match current_tier.next() {
            Some(t) => t,
            None => {
                info!("[Economy] {:?} is already at max tier (Iridium).", ev.tool);
                sfx_writer.send(PlaySfxEvent { sfx_id: "ui_deny".to_string() });
                continue;
            }
        };

        // Gold cost is the upgrade_cost of the *target* tier.
        let gold_cost = target_tier.upgrade_cost();
        if player_state.gold < gold_cost {
            info!(
                "[Economy] Cannot upgrade {:?} to {:?}: need {}g, have {}g.",
                ev.tool, target_tier, gold_cost, player_state.gold
            );
            sfx_writer.send(PlaySfxEvent { sfx_id: "ui_deny".to_string() });
            continue;
        }

        // Bar requirements.
        let (bar_id, bar_qty) = match required_bars_for_tier(target_tier) {
            Some(req) => req,
            None => {
                warn!("[Economy] No bar requirement defined for {:?}.", target_tier);
                continue;
            }
        };

        if !inventory.has(bar_id, bar_qty) {
            info!(
                "[Economy] Cannot upgrade {:?} to {:?}: need {} × '{}', have {}.",
                ev.tool,
                target_tier,
                bar_qty,
                bar_id,
                inventory.count(bar_id)
            );
            sfx_writer.send(PlaySfxEvent { sfx_id: "ui_deny".to_string() });
            continue;
        }

        // All checks passed — commit the upgrade cost.
        player_state.gold -= gold_cost;
        gold_writer.send(GoldChangeEvent {
            amount: -(gold_cost as i32),
            reason: format!("Tool upgrade: {:?} → {:?}", ev.tool, target_tier),
        });

        let removed_bars = inventory.try_remove(bar_id, bar_qty);
        removed_writer.send(ItemRemovedEvent {
            item_id: bar_id.to_string(),
            quantity: removed_bars,
        });

        // Enqueue the upgrade — it takes 2 in-game days.
        upgrade_queue.pending.push(PendingUpgrade {
            tool: ev.tool,
            target_tier,
            days_remaining: 2,
        });

        sfx_writer.send(PlaySfxEvent { sfx_id: "blacksmith_forge".to_string() });

        info!(
            "[Economy] Upgrade started: {:?} → {:?}. Cost: {}g + {} × '{}'. Ready in 2 days.",
            ev.tool, target_tier, gold_cost, removed_bars, bar_id
        );
    }
}

/// Fires every DayEndEvent — ticks down upgrade timers and applies completed upgrades.
pub fn tick_upgrade_queue(
    mut day_end_events: EventReader<DayEndEvent>,
    mut upgrade_queue: ResMut<ToolUpgradeQueue>,
    mut player_state: ResMut<PlayerState>,
    mut complete_writer: EventWriter<ToolUpgradeCompleteEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        let mut completed = Vec::new();

        for upgrade in upgrade_queue.pending.iter_mut() {
            upgrade.days_remaining = upgrade.days_remaining.saturating_sub(1);
            if upgrade.days_remaining == 0 {
                completed.push((upgrade.tool, upgrade.target_tier));
            }
        }

        // Apply completed upgrades and fire events.
        for (tool, new_tier) in completed {
            upgrade_queue.pending.retain(|p| p.tool != tool);
            player_state.tools.insert(tool, new_tier);

            complete_writer.send(ToolUpgradeCompleteEvent { tool, new_tier });

            sfx_writer.send(PlaySfxEvent {
                sfx_id: "upgrade_complete".to_string(),
            });

            toast_writer.send(ToastEvent {
                message: format!("Your {:?} upgrade is ready! Pick it up at the Blacksmith.", tool),
                duration_secs: 4.0,
            });

            info!(
                "[Economy] Tool upgrade complete! {:?} is now {:?}.",
                tool, new_tier
            );
        }
    }
}

/// Builds the list of possible upgrades for the UI to display in the Blacksmith.
/// Returns `(tool, current_tier, target_tier, gold_cost, bar_id, bar_qty, can_afford, has_bars, is_upgrading)`.
pub type UpgradeEntry = (
    ToolKind,
    ToolTier,
    ToolTier,
    u32,
    &'static str,
    u8,
    bool,  // can_afford
    bool,  // has_bars
    bool,  // is_upgrading
);

#[allow(dead_code)]
pub fn list_available_upgrades(
    player_state: &PlayerState,
    inventory: &Inventory,
    queue: &ToolUpgradeQueue,
) -> Vec<UpgradeEntry> {
    let upgradeable_tools = [
        ToolKind::Hoe,
        ToolKind::WateringCan,
        ToolKind::Axe,
        ToolKind::Pickaxe,
        ToolKind::FishingRod,
    ];

    upgradeable_tools
        .iter()
        .filter_map(|&tool| {
            let current = *player_state.tools.get(&tool)?;
            let target = current.next()?; // None = already max
            let gold_cost = target.upgrade_cost();
            let (bar_id, bar_qty) = required_bars_for_tier(target)?;
            let can_afford = player_state.gold >= gold_cost;
            let has_bars = inventory.has(bar_id, bar_qty);
            let is_upgrading = queue.is_upgrading(tool);
            Some((
                tool,
                current,
                target,
                gold_cost,
                bar_id,
                bar_qty,
                can_afford,
                has_bars,
                is_upgrading,
            ))
        })
        .collect()
}
