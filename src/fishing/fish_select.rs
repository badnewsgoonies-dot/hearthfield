//! Fish selection logic based on location, season, time, weather, and rarity.
//!
//! Legendary fish are checked first via a per-cast probability roll. If no
//! legendary triggers, the normal weighted pool is used.

use rand::Rng;

use crate::shared::*;
use super::legendaries::try_roll_legendary;

// ─── Rarity weights ──────────────────────────────────────────────────────────

fn rarity_weight(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 60,
        Rarity::Uncommon => 25,
        Rarity::Rare => 12,
        // Legendary fish in the normal pool (registered via data) have very low
        // weight; they are primarily obtained through try_roll_legendary().
        Rarity::Legendary => 1,
    }
}

// ─── Map → FishLocation mapping ──────────────────────────────────────────────

fn map_to_fish_location(map_id: MapId) -> FishLocation {
    match map_id {
        MapId::Farm | MapId::Forest => FishLocation::River,
        MapId::Beach => FishLocation::Ocean,
        MapId::Town => FishLocation::Pond,
        MapId::Mine | MapId::MineEntrance => FishLocation::MinePool,
        // Indoor maps default to pond
        _ => FishLocation::Pond,
    }
}

// ─── Selection ───────────────────────────────────────────────────────────────

/// Select a fish from the registry appropriate for current game state.
///
/// Legendary fish are given a first-priority independent roll. If no legendary
/// triggers, the normal weighted pool of eligible fish is used.
///
/// Returns `None` if no fish qualify (very unlikely with a full registry).
pub fn select_fish(
    fish_registry: &FishRegistry,
    player_state: &PlayerState,
    calendar: &Calendar,
) -> Option<ItemId> {
    let map_id = player_state.current_map;
    let location = map_to_fish_location(map_id);
    let season = calendar.season;
    let time = calendar.time_float();
    let weather = calendar.weather;

    // ── Step 1: Legendary check ───────────────────────────────────────────
    // Each legendary has a small independent spawn-chance per cast.
    if let Some((legendary_id, _difficulty)) = try_roll_legendary(map_id, season) {
        // Legendary triggered — verify it exists in registry (or return it
        // anyway; catch_fish will fall back to a default if it's missing).
        return Some(legendary_id.to_string());
    }

    // ── Step 2: Normal weighted pool ─────────────────────────────────────
    let eligible: Vec<(&FishDef, u32)> = fish_registry
        .fish
        .values()
        .filter(|f| {
            // Location must match
            if f.location != location {
                return false;
            }
            // Must be catchable in current season
            if !f.seasons.contains(&season) {
                return false;
            }
            // Must be within time range
            let (t_min, t_max) = f.time_range;
            if time < t_min || time > t_max {
                return false;
            }
            // Weather requirement (if any)
            if let Some(required_weather) = f.weather_required {
                if required_weather != weather {
                    return false;
                }
            }
            true
        })
        .map(|f| {
            let w = rarity_weight(f.rarity);
            (f, w)
        })
        .collect();

    if eligible.is_empty() {
        // Fallback: pick any fish from the location ignoring time/weather constraints
        let fallback: Vec<(&FishDef, u32)> = fish_registry
            .fish
            .values()
            .filter(|f| f.location == location && f.seasons.contains(&season))
            .map(|f| (f, rarity_weight(f.rarity)))
            .collect();

        if fallback.is_empty() {
            // Last resort: any fish at all
            let all: Vec<(&FishDef, u32)> = fish_registry
                .fish
                .values()
                .map(|f| (f, rarity_weight(f.rarity)))
                .collect();
            return weighted_pick(&all);
        }
        return weighted_pick(&fallback);
    }

    weighted_pick(&eligible)
}

/// Weighted random pick from a slice of (item, weight) pairs.
fn weighted_pick(items: &[(&FishDef, u32)]) -> Option<ItemId> {
    if items.is_empty() {
        return None;
    }

    let total: u32 = items.iter().map(|(_, w)| w).sum();
    if total == 0 {
        return None;
    }

    let mut rng = rand::thread_rng();
    let mut roll = rng.gen_range(0..total);

    for (fish, weight) in items {
        if roll < *weight {
            return Some(fish.id.clone());
        }
        roll -= weight;
    }

    // Fallback: last item
    items.last().map(|(f, _)| f.id.clone())
}

// ─── Fish data constants ──────────────────────────────────────────────────────
// NOTE: The data domain populates FishRegistry. This module only performs selection.
// The 20 fish referenced in the spec are:
//   River: sardine(common), herring(common), bass(uncommon), trout(uncommon),
//          salmon(uncommon), catfish(uncommon), carp(common), eel(uncommon),
//          pike(uncommon), legend_fish(legendary)
//   Ocean: herring(common), sardine(common), tuna(uncommon), swordfish(rare),
//          pufferfish(uncommon), squid(rare), octopus(rare), glacier_fish(legendary)
//   Pond:  carp(common), perch(common), catfish(uncommon), sturgeon(rare), crimson_fish(legendary)
//   Mine:  anglerfish(rare)
//
// Legendary fish added in the fishing skill expansion:
//   Ocean/Summer:  crimson_king      (difficulty 0.95, 2% spawn)
//   River/Winter:  glacier_pike      (difficulty 0.90, 1.5% spawn)
//   Mine/Fall:     phantom_eel       (difficulty 0.85, 1.5% spawn)
//   River/Spring:  golden_walleye    (difficulty 0.80, 2% spawn)
//   Ocean/Winter:  ancient_coelacanth (difficulty 0.99, 1% spawn)
