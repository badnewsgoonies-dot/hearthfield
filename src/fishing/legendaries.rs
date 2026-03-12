//! Legendary fish definitions and spawn-rate helpers.
//!
//! Legendary fish are extremely rare catches that require specific location,
//! season, and (for some) weather conditions. Their difficulty ratings are the
//! highest in the game, making the minigame very challenging even when they bite.
//!
//! # Spec: 4 legendary fish
//! - Legend: rainy Forest (River), any season, difficulty 0.95, 2% spawn
//! - Crimsonfish: Summer Beach (Ocean), difficulty 0.90, 2% spawn
//! - Glacierfish: Winter Forest (River), difficulty 0.85, 1.5% spawn
//! - Frostfang: Winter SnowMountain (MountainLake), snowy, difficulty 0.92, 1.5% spawn
//!
//! # Spawn rates
//! Each legendary has a per-cast appearance probability (1.5–2%) that is checked
//! independently **before** the normal weighted selection pool. If no legendary
//! triggers, the normal pool is used as a fallback.

use rand::Rng;

use crate::shared::*;

// ─── Legendary fish table ─────────────────────────────────────────────────────

/// Entry in the legendary fish table.
/// (fish_id, required_map, required_season, minigame_difficulty, spawn_chance, weather_required)
type LegendaryEntry = (
    &'static str,
    MapId,
    Option<Season>,
    f32,
    f64,
    Option<Weather>,
);

/// Static table of all legendary fish and their requirements.
///
/// `spawn_chance` is the probability (0.0–1.0) that **this specific legendary**
/// is offered on a qualifying cast. The value is intentionally very low so that
/// legendary fish remain rare even when all conditions are met.
pub const LEGENDARY_FISH: &[LegendaryEntry] = &[
    // Legend — Forest (River), any season, rainy weather required, very hard (0.95), 2% chance
    (
        "legend_fish",
        MapId::Forest,
        None, // any season
        0.95,
        0.02,
        Some(Weather::Rainy),
    ),
    // Crimsonfish — Beach (Ocean), Summer only, any weather, hard (0.90), 2% chance
    (
        "crimsonfish",
        MapId::Beach,
        Some(Season::Summer),
        0.90,
        0.02,
        None,
    ),
    // Glacierfish — Forest (River), Winter only, any weather, moderately hard (0.85), 1.5% chance
    (
        "glacierfish",
        MapId::Forest,
        Some(Season::Winter),
        0.85,
        0.015,
        None,
    ),
    // Frostfang — SnowMountain (MountainLake), Winter only, snowy weather, very hard (0.92), 1.5% chance
    (
        "frostfang",
        MapId::SnowMountain,
        Some(Season::Winter),
        0.92,
        0.015,
        Some(Weather::Snowy),
    ),
];

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Try to roll for a legendary fish given the current game context.
///
/// Returns `Some((fish_id, difficulty))` if a legendary fish triggers this cast,
/// or `None` if normal fish selection should proceed.
///
/// Each legendary is evaluated independently; the first one whose conditions
/// match **and** whose spawn roll succeeds is returned.
pub fn try_roll_legendary(
    map_id: MapId,
    season: Season,
    weather: Weather,
) -> Option<(&'static str, f32)> {
    let mut rng = rand::thread_rng();

    for &(fish_id, req_map, req_season, difficulty, spawn_chance, req_weather) in LEGENDARY_FISH {
        if map_id != req_map {
            continue;
        }

        // Season check: None means any season is valid
        if let Some(required_season) = req_season {
            if season != required_season {
                continue;
            }
        }

        // Weather check: None means any weather is valid
        if let Some(required_weather) = req_weather {
            if weather != required_weather {
                continue;
            }
        }

        if rng.gen_bool(spawn_chance) {
            return Some((fish_id, difficulty));
        }
    }

    None
}

/// Check whether a given fish ID is a legendary fish.
pub fn is_legendary(fish_id: &str) -> bool {
    LEGENDARY_FISH
        .iter()
        .any(|&(id, _, _, _, _, _)| id == fish_id)
}

/// Return `FishDef` data for legendary fish that should be auto-inserted into
/// the `FishRegistry` if not already present. This makes legendaries available
/// even before the full data layer is loaded.
///
/// Each legendary uses `Rarity::Legendary` and the difficulty from the table.
/// Location and season are derived from the table requirements.
#[allow(dead_code)]
pub fn legendary_fish_defs() -> Vec<FishDef> {
    // Map MapId → FishLocation for legendaries
    fn map_to_location(m: MapId) -> FishLocation {
        match m {
            MapId::Farm | MapId::Forest => FishLocation::River,
            MapId::Beach => FishLocation::Ocean,
            MapId::Mine | MapId::MineEntrance => FishLocation::MinePool,
            MapId::SnowMountain => FishLocation::MountainLake,
            _ => FishLocation::Pond,
        }
    }

    LEGENDARY_FISH
        .iter()
        .map(
            |&(fish_id, req_map, req_season, difficulty, _, req_weather)| FishDef {
                id: fish_id.to_string(),
                name: legendary_display_name(fish_id).to_string(),
                location: map_to_location(req_map),
                seasons: match req_season {
                    Some(s) => vec![s],
                    None => vec![Season::Spring, Season::Summer, Season::Fall, Season::Winter],
                },
                // Legendary fish are active all day
                time_range: (6.0, 26.0),
                weather_required: req_weather,
                rarity: Rarity::Legendary,
                difficulty,
                sell_price: legendary_sell_price(fish_id),
                // Placeholder sprite index — data layer should override
                sprite_index: 0,
            },
        )
        .collect()
}

#[allow(dead_code)]
fn legendary_display_name(fish_id: &str) -> &'static str {
    match fish_id {
        "legend_fish" => "Legend",
        "crimsonfish" => "Crimsonfish",
        "glacierfish" => "Glacierfish",
        "frostfang" => "Frostfang",
        _ => "Legendary Fish",
    }
}

#[allow(dead_code)]
fn legendary_sell_price(fish_id: &str) -> u32 {
    match fish_id {
        "legend_fish" => 5_000,
        "crimsonfish" => 1_500,
        "glacierfish" => 1_200,
        "frostfang" => 2_000,
        _ => 500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legendary_fish_table_has_four_entries() {
        assert_eq!(LEGENDARY_FISH.len(), 4);
    }

    #[test]
    fn test_is_legendary_known_fish() {
        assert!(is_legendary("legend_fish"));
        assert!(is_legendary("crimsonfish"));
        assert!(is_legendary("glacierfish"));
        assert!(is_legendary("frostfang"));
    }

    #[test]
    fn test_is_legendary_unknown_fish() {
        assert!(!is_legendary("bass"));
        assert!(!is_legendary("trout"));
        assert!(!is_legendary(""));
        assert!(!is_legendary("legendary_fish"));
    }

    #[test]
    fn test_legendary_difficulty_values_are_high() {
        for &(_id, _map, _season, difficulty, _chance, _weather) in LEGENDARY_FISH {
            assert!(
                difficulty >= 0.80,
                "Legendary fish should have high difficulty, got {}",
                difficulty
            );
        }
    }

    #[test]
    fn test_legendary_spawn_chances_are_low() {
        for &(id, _map, _season, _difficulty, spawn_chance, _weather) in LEGENDARY_FISH {
            assert!(
                spawn_chance <= 0.05,
                "Legendary {} should have low spawn chance, got {}",
                id,
                spawn_chance
            );
            assert!(
                spawn_chance > 0.0,
                "Legendary {} should have positive spawn chance",
                id
            );
        }
    }

    #[test]
    fn test_legendary_fish_defs_count() {
        let defs = legendary_fish_defs();
        assert_eq!(defs.len(), 4);
    }

    #[test]
    fn test_legendary_fish_defs_all_legendary_rarity() {
        let defs = legendary_fish_defs();
        for def in &defs {
            assert_eq!(def.rarity, Rarity::Legendary);
        }
    }

    #[test]
    fn test_legendary_sell_prices_are_positive() {
        for &(id, _, _, _, _, _) in LEGENDARY_FISH {
            let price = legendary_sell_price(id);
            assert!(
                price > 0,
                "Legendary {} should have a positive sell price",
                id
            );
        }
    }

    #[test]
    fn test_try_roll_legendary_wrong_conditions_returns_none() {
        // Crimsonfish requires Beach + Summer. Try with Farm + Winter.
        for _ in 0..1000 {
            let result = try_roll_legendary(MapId::Farm, Season::Winter, Weather::Sunny);
            if let Some((id, _)) = result {
                assert_ne!(
                    id, "crimsonfish",
                    "Should not roll crimsonfish on Farm/Winter"
                );
            }
        }
    }

    #[test]
    fn test_legend_requires_rain() {
        // Legend requires rainy weather in Forest. Sunny should never trigger.
        for _ in 0..1000 {
            let result = try_roll_legendary(MapId::Forest, Season::Spring, Weather::Sunny);
            if let Some((id, _)) = result {
                assert_ne!(
                    id, "legend_fish",
                    "Should not roll legend_fish without rain"
                );
            }
        }
    }
}
