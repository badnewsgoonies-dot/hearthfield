//! Building upgrade system — handles construction requests and timed completion
//! for House, Coop, Barn, and Silo upgrades.

use bevy::prelude::*;
use crate::shared::*;

// ─────────────────────────────────────────────────────────────────────────────
// Cost definitions
// ─────────────────────────────────────────────────────────────────────────────

/// Returns `(gold_cost, Vec<(material_item_id, quantity)>)` for upgrading a
/// building *to* the given tier. Returns `(0, vec![])` for invalid combinations.
fn upgrade_cost(building: BuildingKind, to_tier: BuildingTier) -> (u32, Vec<(&'static str, u8)>) {
    match (building, to_tier) {
        // House upgrades (starts at Basic by default, upgrades to Big then Deluxe)
        (BuildingKind::House, BuildingTier::Big) => (10_000, vec![("wood", 200)]),
        (BuildingKind::House, BuildingTier::Deluxe) => (50_000, vec![("hardwood", 100)]),

        // Coop upgrades (None → Basic → Big → Deluxe)
        (BuildingKind::Coop, BuildingTier::Basic) => (4_000, vec![("wood", 150), ("stone", 50)]),
        (BuildingKind::Coop, BuildingTier::Big) => (10_000, vec![("wood", 200), ("stone", 100)]),
        (BuildingKind::Coop, BuildingTier::Deluxe) => (20_000, vec![("wood", 250), ("stone", 150)]),

        // Barn upgrades (None → Basic → Big → Deluxe)
        (BuildingKind::Barn, BuildingTier::Basic) => (6_000, vec![("wood", 200), ("stone", 75)]),
        (BuildingKind::Barn, BuildingTier::Big) => (12_000, vec![("wood", 250), ("stone", 125)]),
        (BuildingKind::Barn, BuildingTier::Deluxe) => (25_000, vec![("wood", 250), ("stone", 200)]),

        // Silo (only one tier: None → Basic)
        (BuildingKind::Silo, BuildingTier::Basic) => (100, vec![("stone", 50), ("copper_bar", 5)]),

        _ => (0, vec![]),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resource
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks the current tier of each farm building and any upgrade in progress.
#[derive(Resource, Debug, Clone, Default)]
pub struct BuildingLevels {
    pub coop_tier: BuildingTier,
    pub barn_tier: BuildingTier,
    pub silo_built: bool,
    /// Timer: `(building, target_tier, days_left)`. `None` = no upgrade in progress.
    pub upgrade_in_progress: Option<(BuildingKind, BuildingTier, u8)>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Systems
// ─────────────────────────────────────────────────────────────────────────────

/// Listens to [`BuildingUpgradeEvent`] and validates the request.
///
/// On success: deducts gold and materials, starts a 2-day construction timer,
/// and notifies the player with a toast.
///
/// On failure: sends a toast with the reason for denial.
pub fn handle_building_upgrade_request(
    mut events: EventReader<BuildingUpgradeEvent>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    mut building_levels: ResMut<BuildingLevels>,
    mut gold_writer: EventWriter<GoldChangeEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for ev in events.read() {
        // 1. Check no upgrade already in progress.
        if building_levels.upgrade_in_progress.is_some() {
            toast_writer.send(ToastEvent {
                message: "A building upgrade is already in progress!".to_string(),
                duration_secs: 3.0,
            });
            continue;
        }

        // Look up the canonical cost for this upgrade.
        let (gold_cost, material_costs) = upgrade_cost(ev.building, ev.to_tier);

        // Invalid upgrade combination.
        if gold_cost == 0 && material_costs.is_empty() {
            toast_writer.send(ToastEvent {
                message: format!("Cannot upgrade {:?} to {:?}.", ev.building, ev.to_tier),
                duration_secs: 3.0,
            });
            continue;
        }

        // 2. Check if player has enough gold.
        if player_state.gold < gold_cost {
            toast_writer.send(ToastEvent {
                message: format!(
                    "Not enough gold! Need {}g, have {}g.",
                    gold_cost, player_state.gold
                ),
                duration_secs: 3.0,
            });
            continue;
        }

        // 3. Check if player has enough materials.
        let mut missing_materials = Vec::new();
        for &(mat_id, qty) in &material_costs {
            if !inventory.has(mat_id, qty) {
                missing_materials.push(format!(
                    "{} (need {}, have {})",
                    mat_id,
                    qty,
                    inventory.count(mat_id)
                ));
            }
        }

        if !missing_materials.is_empty() {
            toast_writer.send(ToastEvent {
                message: format!("Missing materials: {}", missing_materials.join(", ")),
                duration_secs: 4.0,
            });
            continue;
        }

        // ── All checks passed ──────────────────────────────────────────────

        // Deduct gold.
        player_state.gold -= gold_cost;
        gold_writer.send(GoldChangeEvent {
            amount: -(gold_cost as i32),
            reason: format!("{:?} upgrade to {:?}", ev.building, ev.to_tier),
        });

        // Remove materials from inventory.
        for &(mat_id, qty) in &material_costs {
            inventory.try_remove(mat_id, qty);
        }

        // Start the 2-day construction timer.
        building_levels.upgrade_in_progress = Some((ev.building, ev.to_tier, 2));

        toast_writer.send(ToastEvent {
            message: "Upgrade started! Come back in 2 days.".to_string(),
            duration_secs: 3.5,
        });

        info!(
            "[Economy/Buildings] Upgrade started: {:?} → {:?}. Cost: {}g + {:?}. Ready in 2 days.",
            ev.building, ev.to_tier, gold_cost, material_costs
        );
    }
}

/// Ticks on [`DayEndEvent`]. Decrements the construction timer and applies the
/// upgrade when it reaches zero.
pub fn tick_building_upgrade(
    mut day_events: EventReader<DayEndEvent>,
    mut building_levels: ResMut<BuildingLevels>,
    mut house_state: ResMut<HouseState>,
    mut animal_state: ResMut<AnimalState>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for _ev in day_events.read() {
        let finished = if let Some((building, target_tier, ref mut days_left)) =
            building_levels.upgrade_in_progress
        {
            *days_left = days_left.saturating_sub(1);
            if *days_left == 0 {
                Some((building, target_tier))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((building, target_tier)) = finished {
            // Apply the upgrade based on building kind.
            match building {
                BuildingKind::House => {
                    match target_tier {
                        BuildingTier::Big => {
                            house_state.tier = HouseTier::Big;
                            house_state.has_kitchen = true;
                        }
                        BuildingTier::Deluxe => {
                            house_state.tier = HouseTier::Deluxe;
                            house_state.has_kitchen = true;
                            house_state.has_nursery = true;
                        }
                        _ => {}
                    }
                }
                BuildingKind::Coop => {
                    animal_state.has_coop = true;
                    animal_state.coop_level = match target_tier {
                        BuildingTier::Basic => 1,
                        BuildingTier::Big => 2,
                        BuildingTier::Deluxe => 3,
                        _ => animal_state.coop_level,
                    };
                    building_levels.coop_tier = target_tier;
                }
                BuildingKind::Barn => {
                    animal_state.has_barn = true;
                    animal_state.barn_level = match target_tier {
                        BuildingTier::Basic => 1,
                        BuildingTier::Big => 2,
                        BuildingTier::Deluxe => 3,
                        _ => animal_state.barn_level,
                    };
                    building_levels.barn_tier = target_tier;
                }
                BuildingKind::Silo => {
                    building_levels.silo_built = true;
                }
            }

            // Clear the in-progress slot.
            building_levels.upgrade_in_progress = None;

            sfx_writer.send(PlaySfxEvent {
                sfx_id: "upgrade_complete".to_string(),
            });

            toast_writer.send(ToastEvent {
                message: format!("{:?} upgraded to {:?}!", building, target_tier),
                duration_secs: 4.0,
            });

            info!(
                "[Economy/Buildings] Upgrade complete: {:?} is now {:?}.",
                building, target_tier
            );
        }
    }
}
