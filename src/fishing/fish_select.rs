//! Fish selection logic based on location, season, time, weather, and rarity.

use rand::Rng;

use crate::shared::*;

// ─── Rarity weights ──────────────────────────────────────────────────────────

fn rarity_weight(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 60,
        Rarity::Uncommon => 25,
        Rarity::Rare => 12,
        Rarity::Legendary => 3,
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
/// Returns `None` if no fish qualify (very unlikely with a full registry).
pub fn select_fish(
    fish_registry: &FishRegistry,
    player_state: &PlayerState,
    calendar: &Calendar,
) -> Option<ItemId> {
    let location = map_to_fish_location(player_state.current_map);
    let season = calendar.season;
    let time = calendar.time_float();
    let weather = calendar.weather;

    // Collect eligible fish
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
