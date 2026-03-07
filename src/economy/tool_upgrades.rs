//! Tool upgrade helpers — cost lookups, progress queries, and utility functions.
//!
//! The core upgrade types (`ToolKind`, `ToolTier`) and their methods (`upgrade_cost`,
//! `upgrade_cost_gold`, `upgrade_bars_needed`, `upgrade_bar_item`, `next`) live in
//! `crate::shared`. This module provides economy-specific helpers that combine those
//! primitives with game state to answer higher-level questions.

use crate::shared::*;

/// Summary of what upgrading a specific tool will cost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolUpgradeCost {
    pub tool: ToolKind,
    pub current_tier: ToolTier,
    pub target_tier: ToolTier,
    pub gold_cost: u32,
    pub bar_item: &'static str,
    pub bars_needed: u8,
}

/// Returns the full cost summary for upgrading a tool from its current tier to the
/// next tier, or `None` if the tool is already at max tier (Iridium).
#[allow(dead_code)]
pub fn upgrade_cost_summary(tool: ToolKind, current_tier: ToolTier) -> Option<ToolUpgradeCost> {
    let target_tier = current_tier.next()?;
    let gold_cost = target_tier.upgrade_cost();
    let bar_item = current_tier.upgrade_bar_item()?;
    let bars_needed = current_tier.upgrade_bars_needed();

    Some(ToolUpgradeCost {
        tool,
        current_tier,
        target_tier,
        gold_cost,
        bar_item,
        bars_needed,
    })
}

/// Checks whether the player can afford to upgrade a tool right now.
///
/// Returns `Ok(ToolUpgradeCost)` if affordable, or `Err(&str)` describing why not.
#[allow(dead_code)]
pub fn can_afford_upgrade(
    tool: ToolKind,
    player_state: &PlayerState,
    inventory: &Inventory,
) -> Result<ToolUpgradeCost, &'static str> {
    let current_tier = match player_state.tools.get(&tool) {
        Some(&tier) => tier,
        None => return Err("Tool not found in player state"),
    };

    let cost = match upgrade_cost_summary(tool, current_tier) {
        Some(c) => c,
        None => return Err("Tool is already at maximum tier"),
    };

    if player_state.gold < cost.gold_cost {
        return Err("Not enough gold");
    }

    if !inventory.has(cost.bar_item, cost.bars_needed) {
        return Err("Not enough bars");
    }

    Ok(cost)
}

/// Returns the total gold investment needed to fully upgrade a tool from Basic to Iridium.
/// Useful for displaying "total remaining cost" in the UI.
#[allow(dead_code)]
pub fn total_remaining_upgrade_cost(current_tier: ToolTier) -> u32 {
    let mut tier = current_tier;
    let mut total = 0u32;
    while let Some(next) = tier.next() {
        total = total.saturating_add(next.upgrade_cost());
        tier = next;
    }
    total
}

/// Returns a human-readable description of the upgrade path for a tool.
#[allow(dead_code)]
pub fn upgrade_path_description(tool: ToolKind, current_tier: ToolTier) -> String {
    let mut descriptions = Vec::new();
    let mut tier = current_tier;

    while let Some(next) = tier.next() {
        let bar_item = tier.upgrade_bar_item().unwrap_or("unknown_bar");
        let bars = tier.upgrade_bars_needed();
        let gold = next.upgrade_cost();
        descriptions.push(format!(
            "{:?} -> {:?}: {}g + {} x {}",
            tier, next, gold, bars, bar_item
        ));
        tier = next;
    }

    if descriptions.is_empty() {
        format!("{:?} is already at maximum tier (Iridium)", tool)
    } else {
        descriptions.join(" | ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upgrade_cost_summary_basic() {
        let cost = upgrade_cost_summary(ToolKind::Hoe, ToolTier::Basic).unwrap();
        assert_eq!(cost.target_tier, ToolTier::Copper);
        assert_eq!(cost.gold_cost, 2_000);
        assert_eq!(cost.bar_item, "copper_bar");
        assert_eq!(cost.bars_needed, 5);
    }

    #[test]
    fn test_upgrade_cost_summary_copper() {
        let cost = upgrade_cost_summary(ToolKind::Pickaxe, ToolTier::Copper).unwrap();
        assert_eq!(cost.target_tier, ToolTier::Iron);
        assert_eq!(cost.gold_cost, 5_000);
        assert_eq!(cost.bar_item, "iron_bar");
        assert_eq!(cost.bars_needed, 5);
    }

    #[test]
    fn test_upgrade_cost_summary_iron() {
        let cost = upgrade_cost_summary(ToolKind::Axe, ToolTier::Iron).unwrap();
        assert_eq!(cost.target_tier, ToolTier::Gold);
        assert_eq!(cost.gold_cost, 10_000);
        assert_eq!(cost.bar_item, "gold_bar");
        assert_eq!(cost.bars_needed, 5);
    }

    #[test]
    fn test_upgrade_cost_summary_gold() {
        let cost = upgrade_cost_summary(ToolKind::WateringCan, ToolTier::Gold).unwrap();
        assert_eq!(cost.target_tier, ToolTier::Iridium);
        assert_eq!(cost.gold_cost, 25_000);
        assert_eq!(cost.bar_item, "iridium_bar");
        assert_eq!(cost.bars_needed, 5);
    }

    #[test]
    fn test_upgrade_cost_summary_iridium_returns_none() {
        let cost = upgrade_cost_summary(ToolKind::Hoe, ToolTier::Iridium);
        assert!(cost.is_none());
    }

    #[test]
    fn test_total_remaining_cost_from_basic() {
        // Basic -> Copper (2000) -> Iron (5000) -> Gold (10000) -> Iridium (25000) = 42000
        assert_eq!(total_remaining_upgrade_cost(ToolTier::Basic), 42_000);
    }

    #[test]
    fn test_total_remaining_cost_from_gold() {
        // Gold -> Iridium (25000) = 25000
        assert_eq!(total_remaining_upgrade_cost(ToolTier::Gold), 25_000);
    }

    #[test]
    fn test_total_remaining_cost_from_iridium() {
        assert_eq!(total_remaining_upgrade_cost(ToolTier::Iridium), 0);
    }

    #[test]
    fn test_can_afford_upgrade_success() {
        let mut player = PlayerState::default();
        player.gold = 10_000;
        player.tools.insert(ToolKind::Hoe, ToolTier::Basic);

        let mut inv = Inventory::default();
        inv.try_add("copper_bar", 10, 99);

        let result = can_afford_upgrade(ToolKind::Hoe, &player, &inv);
        assert!(result.is_ok());
        let cost = result.unwrap();
        assert_eq!(cost.gold_cost, 2_000);
    }

    #[test]
    fn test_can_afford_upgrade_insufficient_gold() {
        let mut player = PlayerState::default();
        player.gold = 100;
        player.tools.insert(ToolKind::Hoe, ToolTier::Basic);

        let mut inv = Inventory::default();
        inv.try_add("copper_bar", 5, 99);

        let result = can_afford_upgrade(ToolKind::Hoe, &player, &inv);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough gold");
    }

    #[test]
    fn test_can_afford_upgrade_insufficient_bars() {
        let mut player = PlayerState::default();
        player.gold = 10_000;
        player.tools.insert(ToolKind::Hoe, ToolTier::Basic);

        let inv = Inventory::default(); // no bars

        let result = can_afford_upgrade(ToolKind::Hoe, &player, &inv);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Not enough bars");
    }

    #[test]
    fn test_can_afford_upgrade_max_tier() {
        let mut player = PlayerState::default();
        player.gold = 999_999;
        player.tools.insert(ToolKind::Hoe, ToolTier::Iridium);

        let inv = Inventory::default();

        let result = can_afford_upgrade(ToolKind::Hoe, &player, &inv);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Tool is already at maximum tier");
    }

    #[test]
    fn test_upgrade_path_description_basic() {
        let desc = upgrade_path_description(ToolKind::Hoe, ToolTier::Basic);
        assert!(desc.contains("Basic -> Copper"));
        assert!(desc.contains("2000g"));
        assert!(desc.contains("copper_bar"));
    }

    #[test]
    fn test_upgrade_path_description_iridium() {
        let desc = upgrade_path_description(ToolKind::Hoe, ToolTier::Iridium);
        assert!(desc.contains("maximum tier"));
    }
}
