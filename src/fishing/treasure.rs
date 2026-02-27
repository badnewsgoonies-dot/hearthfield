//! Treasure chest loot logic for the fishing system.
//!
//! A small percentage of successful catches also yield a treasure chest containing
//! a random selection of items and some gold. The base chance is 5%, but certain
//! bait types (magnet_bait, wild_bait) raise that probability significantly.
//!
//! # Loot table
//! | Tier     | Weight | Example items                          |
//! |----------|--------|----------------------------------------|
//! | Ore      | 60%    | copper_ore ×3, iron_ore ×2             |
//! | Gem      | 20%    | amethyst ×1, topaz ×1                  |
//! | Artifact | 15%    | ancient_sword ×1, dinosaur_egg ×1      |
//! | Gold     | always | 50–200 gold                            |

use rand::Rng;

use crate::shared::*;

// ─── Treasure chance constants ────────────────────────────────────────────────

/// Default treasure chance per catch (5%).
pub const BASE_TREASURE_CHANCE: f64 = 0.05;
/// Extra treasure chance when magnet_bait is equipped (+15%).
#[allow(dead_code)]
pub const MAGNET_BAIT_EXTRA_CHANCE: f64 = 0.15;
/// Extra treasure chance when wild_bait is equipped (+5%).
pub const WILD_BAIT_EXTRA_CHANCE: f64 = 0.05;

// ─── Data types ───────────────────────────────────────────────────────────────

/// Contents of a found treasure chest.
#[derive(Debug, Clone)]
pub struct TreasureContents {
    /// Items inside the chest: (item_id, quantity).
    pub items: Vec<(String, u8)>,
    /// Gold coins found alongside the items.
    pub gold: u32,
}

// ─── Loot rolling ─────────────────────────────────────────────────────────────

/// Roll for treasure chest contents using weighted loot tables.
///
/// ```
/// // 60% → ore tier  (copper_ore ×3 or iron_ore ×2)
/// // 20% → gem tier  (amethyst ×1 or topaz ×1)
/// // 15% → artifact  (ancient_sword ×1 or dinosaur_egg ×1)
/// //  5% → rare      (iridium_ore ×1 or prismatic_shard ×1)
/// // Always: 50–200 gold
/// ```
pub fn roll_treasure() -> TreasureContents {
    let mut rng = rand::thread_rng();

    let tier: f64 = rng.gen();
    let (item_id, qty): (&str, u8) = if tier < 0.60 {
        // Ore tier
        if rng.gen_bool(0.5) {
            ("copper_ore", 3)
        } else {
            ("iron_ore", 2)
        }
    } else if tier < 0.80 {
        // Gem tier
        if rng.gen_bool(0.5) {
            ("amethyst", 1)
        } else {
            ("topaz", 1)
        }
    } else if tier < 0.95 {
        // Artifact tier
        if rng.gen_bool(0.5) {
            ("ancient_sword", 1)
        } else {
            ("dinosaur_egg", 1)
        }
    } else {
        // Rare tier
        if rng.gen_bool(0.5) {
            ("iridium_ore", 1)
        } else {
            ("prismatic_shard", 1)
        }
    };

    let gold = rng.gen_range(50u32..=200u32);

    TreasureContents {
        items: vec![(item_id.to_string(), qty)],
        gold,
    }
}

/// Determine whether a treasure chest should spawn this catch and, if so,
/// generate its contents and send the appropriate events.
///
/// `treasure_chance` is the probability in [0.0, 1.0]. Pass
/// `BASE_TREASURE_CHANCE` for a bare-rod cast; bait types may add their own
/// bonus before calling this function.
pub fn check_and_grant_treasure(
    treasure_chance: f64,
    item_pickup_events: &mut bevy::prelude::EventWriter<ItemPickupEvent>,
    gold_change_events: &mut bevy::prelude::EventWriter<GoldChangeEvent>,
    toast_events: &mut bevy::prelude::EventWriter<ToastEvent>,
    sfx_events: &mut bevy::prelude::EventWriter<PlaySfxEvent>,
) {
    let mut rng = rand::thread_rng();
    if !rng.gen_bool(treasure_chance) {
        return;
    }

    let contents = roll_treasure();

    // Grant items
    for (item_id, qty) in &contents.items {
        item_pickup_events.send(ItemPickupEvent {
            item_id: item_id.clone(),
            quantity: *qty,
        });
    }

    // Grant gold
    if contents.gold > 0 {
        gold_change_events.send(GoldChangeEvent {
            amount: contents.gold as i32,
            reason: "Treasure chest".to_string(),
        });
    }

    toast_events.send(ToastEvent {
        message: format!(
            "You found a treasure chest! (+{} gold)",
            contents.gold
        ),
        duration_secs: 3.5,
    });

    sfx_events.send(PlaySfxEvent {
        sfx_id: "treasure_found".to_string(),
    });
}
