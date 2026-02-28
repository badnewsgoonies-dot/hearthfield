//! Sprinkler placement & automation — Phase 4 kind-aware sprinkler system.
//!
//! Manages `SprinklerState` (the authoritative registry of all placed sprinklers)
//! and provides three systems:
//!
//! * `handle_place_sprinkler` — validates and processes `PlaceSprinklerEvent`
//! * `auto_water_sprinklers`  — runs on `DayEndEvent`, waters tiles via kind-aware
//!                              range/diagonal logic (replaces legacy 3×3 for Phase 4)
//! * `remove_sprinkler`       — listens for pickaxe/interact tool use on a sprinkler
//!                              tile and returns it to inventory
//!
//! All shared types are imported via `crate::shared::*`.  No imports from any other
//! domain are permitted.

use bevy::prelude::*;
use crate::shared::*;
use super::{FarmEntities, soil::spawn_or_update_soil_entity};

// ─────────────────────────────────────────────────────────────────────────────
// Public helper — compute affected tiles for any sprinkler kind
// ─────────────────────────────────────────────────────────────────────────────

/// Compute all tiles watered by a sprinkler of `kind` placed at (`cx`, `cy`).
///
/// * `Basic`   — range 1, cardinal only  →  4 tiles  (N/S/E/W)
/// * `Quality` — range 1, with diagonals →  8 tiles  (3×3 minus centre)
/// * `Iridium` — range 2, with diagonals → 24 tiles  (5×5 minus centre)
pub fn sprinkler_affected_tiles(kind: SprinklerKind, cx: i32, cy: i32) -> Vec<(i32, i32)> {
    let range = kind.range() as i32;
    let diags = kind.includes_diagonals();
    let mut tiles = Vec::new();
    for dx in -range..=range {
        for dy in -range..=range {
            if dx == 0 && dy == 0 {
                continue; // skip the centre (the sprinkler's own tile)
            }
            if !diags && dx != 0 && dy != 0 {
                continue; // cardinal-only: skip diagonals
            }
            tiles.push((cx + dx, cy + dy));
        }
    }
    tiles
}

// ─────────────────────────────────────────────────────────────────────────────
// Item ID constants for the three sprinkler variants
// ─────────────────────────────────────────────────────────────────────────────

const ITEM_BASIC_SPRINKLER: &str = "sprinkler";
const ITEM_QUALITY_SPRINKLER: &str = "quality_sprinkler";
const ITEM_IRIDIUM_SPRINKLER: &str = "iridium_sprinkler";

fn sprinkler_item_id(kind: SprinklerKind) -> &'static str {
    match kind {
        SprinklerKind::Basic   => ITEM_BASIC_SPRINKLER,
        SprinklerKind::Quality => ITEM_QUALITY_SPRINKLER,
        SprinklerKind::Iridium => ITEM_IRIDIUM_SPRINKLER,
    }
}

fn sprinkler_display_name(kind: SprinklerKind) -> &'static str {
    match kind {
        SprinklerKind::Basic   => "Sprinkler",
        SprinklerKind::Quality => "Quality Sprinkler",
        SprinklerKind::Iridium => "Iridium Sprinkler",
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 1 — handle_place_sprinkler
// ─────────────────────────────────────────────────────────────────────────────

/// Reads `PlaceSprinklerEvent`, validates placement, removes the item from
/// inventory, registers the sprinkler in `SprinklerState`, adds a
/// `FarmObject::Sprinkler` entry in `FarmState.objects`, and sends feedback.
pub fn handle_place_sprinkler(
    mut events: EventReader<PlaceSprinklerEvent>,
    mut sprinkler_state: ResMut<SprinklerState>,
    mut farm_state: ResMut<FarmState>,
    mut inventory: ResMut<Inventory>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for ev in events.read() {
        let pos = (ev.tile_x, ev.tile_y);

        // ── Validation 1: tile must not already be occupied by any farm object ──
        if farm_state.objects.contains_key(&pos) {
            toast_events.send(ToastEvent {
                message: "That tile is already occupied.".to_string(),
                duration_secs: 2.0,
            });
            continue;
        }

        // ── Validation 2: must not already have a Phase-4 sprinkler here ──
        let already_placed = sprinkler_state
            .sprinklers
            .iter()
            .any(|s| s.tile_x == ev.tile_x && s.tile_y == ev.tile_y);
        if already_placed {
            toast_events.send(ToastEvent {
                message: "A sprinkler is already placed there.".to_string(),
                duration_secs: 2.0,
            });
            continue;
        }

        // ── Validation 3: player must have the item ──
        let item_id = sprinkler_item_id(ev.kind);
        if !inventory.has(item_id, 1) {
            toast_events.send(ToastEvent {
                message: format!("You don't have a {}.", sprinkler_display_name(ev.kind)),
                duration_secs: 2.0,
            });
            continue;
        }

        // ── Consume item ──
        inventory.try_remove(item_id, 1);

        // ── Register in SprinklerState ──
        sprinkler_state.sprinklers.push(PlacedSprinkler {
            kind: ev.kind,
            tile_x: ev.tile_x,
            tile_y: ev.tile_y,
        });

        // ── Add visual representation in FarmState ──
        farm_state.objects.insert(pos, FarmObject::Sprinkler);

        // ── Feedback ──
        toast_events.send(ToastEvent {
            message: format!("{} placed.", sprinkler_display_name(ev.kind)),
            duration_secs: 2.0,
        });
        sfx_events.send(PlaySfxEvent {
            sfx_id: "place_object".to_string(),
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 2 — auto_water_sprinklers
// ─────────────────────────────────────────────────────────────────────────────

/// Runs on `DayEndEvent`.  For every sprinkler in `SprinklerState`, computes
/// the set of affected tiles using the kind-aware range/diagonal rules and
/// waters any `Tilled` soil in that area (sets to `Watered`, marks crops
/// `watered_today`).
///
/// This system handles Phase 4 (kind-aware) sprinklers.  The legacy
/// `sprinkler::apply_sprinklers` still handles old-style `FarmObject::Sprinkler`
/// entries that were placed without a `PlacedSprinkler` record.
pub fn auto_water_sprinklers(
    mut day_end_events: EventReader<DayEndEvent>,
    sprinkler_state: Res<SprinklerState>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
) {
    // Only process when a day has actually ended.
    if day_end_events.read().next().is_none() {
        return;
    }

    // Iterate sprinkler list by reference (separate resource from farm_state).
    for sp in &sprinkler_state.sprinklers {
        let affected = sprinkler_affected_tiles(sp.kind, sp.tile_x, sp.tile_y);
        for tile_pos in affected {
            let current_soil = farm_state.soil.get(&tile_pos).copied();
            if current_soil == Some(SoilState::Tilled) {
                // Water the soil.
                farm_state.soil.insert(tile_pos, SoilState::Watered);

                // Mark any crop on this tile.
                if let Some(crop) = farm_state.crops.get_mut(&tile_pos) {
                    crop.watered_today = true;
                    crop.days_without_water = 0;
                }

                // Ensure the ECS entity is up to date.
                spawn_or_update_soil_entity(
                    &mut commands,
                    &mut farm_entities,
                    tile_pos,
                    SoilState::Watered,
                );
            }
            // Already-watered tiles don't need action; untilled tiles are skipped.
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System 3 — remove_sprinkler
// ─────────────────────────────────────────────────────────────────────────────

/// Listens for `ToolUseEvent` targeting a tile that holds a Phase-4 sprinkler
/// (pickaxe interaction picks it up; any non-damaging tool also works for
/// convenience).  On match:
///
/// 1. Removes the `PlacedSprinkler` from `SprinklerState`.
/// 2. Removes the `FarmObject::Sprinkler` from `FarmState.objects`.
/// 3. Returns the item to the player's inventory via `ItemPickupEvent`.
/// 4. Sends toast + sfx feedback.
///
/// If the tile holds a legacy `FarmObject::Sprinkler` that has NO matching
/// `PlacedSprinkler` entry we still clean it up so the farm stays consistent,
/// but we return a basic sprinkler to inventory as the best-effort fallback.
pub fn remove_sprinkler(
    mut tool_events: EventReader<ToolUseEvent>,
    mut sprinkler_state: ResMut<SprinklerState>,
    mut farm_state: ResMut<FarmState>,
    mut inventory: ResMut<Inventory>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for ev in tool_events.read() {
        // Only pickaxe or interact (no tool) can pick up a sprinkler.
        // We treat Pickaxe as the canonical removal tool.
        if ev.tool != ToolKind::Pickaxe {
            continue;
        }

        let pos = (ev.target_x, ev.target_y);

        // Check whether there is a farm object that is a Sprinkler here.
        let has_object_sprinkler = matches!(
            farm_state.objects.get(&pos),
            Some(FarmObject::Sprinkler)
        );
        if !has_object_sprinkler {
            continue;
        }

        // Find a matching Phase-4 sprinkler entry.
        let maybe_idx = sprinkler_state
            .sprinklers
            .iter()
            .position(|s| s.tile_x == ev.target_x && s.tile_y == ev.target_y);

        let returned_item_id;
        let display_name;

        if let Some(idx) = maybe_idx {
            // Phase-4 sprinkler: return the exact kind.
            let placed = sprinkler_state.sprinklers.remove(idx);
            returned_item_id = sprinkler_item_id(placed.kind).to_string();
            display_name = sprinkler_display_name(placed.kind).to_string();
        } else {
            // Legacy sprinkler (FarmObject::Sprinkler without a SprinklerState entry).
            // Return a basic sprinkler as best-effort.
            returned_item_id = ITEM_BASIC_SPRINKLER.to_string();
            display_name = sprinkler_display_name(SprinklerKind::Basic).to_string();
        }

        // Remove the visual entry from FarmState.
        farm_state.objects.remove(&pos);

        // Return item to inventory (stack size 1 per slot; sprinklers don't stack).
        inventory.try_add(&returned_item_id, 1, 1);

        // Feedback.
        toast_events.send(ToastEvent {
            message: format!("{} picked up.", display_name),
            duration_secs: 2.0,
        });
        sfx_events.send(PlaySfxEvent {
            sfx_id: "pickup".to_string(),
        });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_sprinkler_four_cardinal_tiles() {
        let tiles = sprinkler_affected_tiles(SprinklerKind::Basic, 0, 0);
        assert_eq!(tiles.len(), 4, "Basic sprinkler waters exactly 4 tiles");
        assert!(tiles.contains(&(0, 1)),  "North");
        assert!(tiles.contains(&(0, -1)), "South");
        assert!(tiles.contains(&(1, 0)),  "East");
        assert!(tiles.contains(&(-1, 0)), "West");
        // No diagonals.
        assert!(!tiles.contains(&(1, 1)));
        assert!(!tiles.contains(&(-1, -1)));
    }

    #[test]
    fn quality_sprinkler_eight_tiles() {
        let tiles = sprinkler_affected_tiles(SprinklerKind::Quality, 5, 5);
        assert_eq!(tiles.len(), 8, "Quality sprinkler waters exactly 8 tiles (3×3 minus centre)");
        // Centre must not be included.
        assert!(!tiles.contains(&(5, 5)));
        // All 8 surrounding tiles.
        for dx in -1i32..=1 {
            for dy in -1i32..=1 {
                if dx == 0 && dy == 0 { continue; }
                assert!(tiles.contains(&(5 + dx, 5 + dy)), "Missing ({}, {})", 5+dx, 5+dy);
            }
        }
    }

    #[test]
    fn iridium_sprinkler_twenty_four_tiles() {
        let tiles = sprinkler_affected_tiles(SprinklerKind::Iridium, 0, 0);
        assert_eq!(tiles.len(), 24, "Iridium sprinkler waters exactly 24 tiles (5×5 minus centre)");
        // Centre must not be included.
        assert!(!tiles.contains(&(0, 0)));
        // All tiles in the 5×5 box (range 2).
        for dx in -2i32..=2 {
            for dy in -2i32..=2 {
                if dx == 0 && dy == 0 { continue; }
                assert!(tiles.contains(&(dx, dy)), "Missing ({}, {})", dx, dy);
            }
        }
    }

    #[test]
    fn sprinkler_item_ids_are_correct() {
        assert_eq!(sprinkler_item_id(SprinklerKind::Basic),   "sprinkler");
        assert_eq!(sprinkler_item_id(SprinklerKind::Quality), "quality_sprinkler");
        assert_eq!(sprinkler_item_id(SprinklerKind::Iridium), "iridium_sprinkler");
    }
}
