//! Legendary fish definitions and spawn-rate helpers.
//!
//! Legendary fish are extremely rare catches that require specific location,
//! season, and (for some) weather conditions. Their difficulty ratings are the
//! highest in the game, making the minigame very challenging even when they bite.
//!
//! # Spawn rates
//! Each legendary has a per-cast appearance probability (1–2%) that is checked
//! independently **before** the normal weighted selection pool. If no legendary
//! triggers, the normal pool is used as a fallback.

use rand::Rng;

use crate::shared::*;

// ─── Legendary fish table ─────────────────────────────────────────────────────

/// Static table of all legendary fish and their requirements.
///
/// Columns: (fish_id, required_map, required_season, minigame_difficulty, spawn_chance)
///
/// `spawn_chance` is the probability (0.0–1.0) that **this specific legendary**
/// is offered on a qualifying cast. The value is intentionally very low so that
/// legendary fish remain rare even when all conditions are met.
pub const LEGENDARY_FISH: &[(&str, MapId, Season, f32, f64)] = &[
    // Crimson King — Beach in summer, very hard (0.95), 2% chance
    ("crimson_king", MapId::Beach, Season::Summer, 0.95, 0.02),
    // Glacier Pike — Forest in winter, hard (0.90), 1.5% chance
    ("glacier_pike", MapId::Forest, Season::Winter, 0.90, 0.015),
    // Phantom Eel — Mine area in fall, moderately hard (0.85), 1.5% chance
    ("phantom_eel", MapId::Mine, Season::Fall, 0.85, 0.015),
    // Golden Walleye — Farm pond in spring, somewhat hard (0.80), 2% chance
    ("golden_walleye", MapId::Farm, Season::Spring, 0.80, 0.02),
    // Ancient Coelacanth — Beach in winter, the rarest fish in the game (0.99), 1% chance
    ("ancient_coelacanth", MapId::Beach, Season::Winter, 0.99, 0.01),
];

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Try to roll for a legendary fish given the current game context.
///
/// Returns `Some((fish_id, difficulty))` if a legendary fish triggers this cast,
/// or `None` if normal fish selection should proceed.
///
/// Each legendary is evaluated independently; the first one whose conditions
/// match **and** whose spawn roll succeeds is returned. If multiple legendaries
/// match conditions (unlikely given the table design), the first one to win its
/// roll is used.
pub fn try_roll_legendary(
    map_id: MapId,
    season: Season,
) -> Option<(&'static str, f32)> {
    let mut rng = rand::thread_rng();

    for &(fish_id, req_map, req_season, difficulty, spawn_chance) in LEGENDARY_FISH {
        if map_id != req_map || season != req_season {
            continue;
        }

        if rng.gen_bool(spawn_chance) {
            return Some((fish_id, difficulty));
        }
    }

    None
}

/// Check whether a given fish ID is a legendary fish.
pub fn is_legendary(fish_id: &str) -> bool {
    LEGENDARY_FISH.iter().any(|&(id, _, _, _, _)| id == fish_id)
}

/// Return `FishDef` data for legendary fish that should be auto-inserted into
/// the `FishRegistry` if not already present. This makes legendaries available
/// even before the full data layer is loaded.
#[allow(dead_code)]
///
/// Each legendary uses `Rarity::Legendary` and the difficulty from the table.
/// Location and season are derived from the table requirements.
pub fn legendary_fish_defs() -> Vec<FishDef> {
    // Map MapId → FishLocation for legendaries
    fn map_to_location(m: MapId) -> FishLocation {
        match m {
            MapId::Farm | MapId::Forest => FishLocation::River,
            MapId::Beach => FishLocation::Ocean,
            MapId::Mine | MapId::MineEntrance => FishLocation::MinePool,
            _ => FishLocation::Pond,
        }
    }

    LEGENDARY_FISH
        .iter()
        .map(|&(fish_id, req_map, req_season, difficulty, _)| FishDef {
            id: fish_id.to_string(),
            name: legendary_display_name(fish_id).to_string(),
            location: map_to_location(req_map),
            seasons: vec![req_season],
            // Legendary fish are active all day
            time_range: (6.0, 26.0),
            weather_required: None,
            rarity: Rarity::Legendary,
            difficulty,
            sell_price: legendary_sell_price(fish_id),
            // Placeholder sprite index — data layer should override
            sprite_index: 0,
        })
        .collect()
}

#[allow(dead_code)]
fn legendary_display_name(fish_id: &str) -> &'static str {
    match fish_id {
        "crimson_king" => "Crimson King",
        "glacier_pike" => "Glacier Pike",
        "phantom_eel" => "Phantom Eel",
        "golden_walleye" => "Golden Walleye",
        "ancient_coelacanth" => "Ancient Coelacanth",
        _ => "Legendary Fish",
    }
}

#[allow(dead_code)]
fn legendary_sell_price(fish_id: &str) -> u32 {
    match fish_id {
        "crimson_king" => 1_500,
        "glacier_pike" => 1_200,
        "phantom_eel" => 1_000,
        "golden_walleye" => 900,
        "ancient_coelacanth" => 5_000,
        _ => 500,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legendary_fish_table_has_five_entries() {
        assert_eq!(LEGENDARY_FISH.len(), 5);
    }

    #[test]
    fn test_is_legendary_known_fish() {
        assert!(is_legendary("crimson_king"));
        assert!(is_legendary("glacier_pike"));
        assert!(is_legendary("phantom_eel"));
        assert!(is_legendary("golden_walleye"));
        assert!(is_legendary("ancient_coelacanth"));
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
        for &(_id, _map, _season, difficulty, _chance) in LEGENDARY_FISH {
            assert!(
                difficulty >= 0.80,
                "Legendary fish should have high difficulty, got {}",
                difficulty
            );
        }
    }

    #[test]
    fn test_legendary_spawn_chances_are_low() {
        for &(id, _map, _season, _difficulty, spawn_chance) in LEGENDARY_FISH {
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
        assert_eq!(defs.len(), 5);
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
        for &(id, _, _, _, _) in LEGENDARY_FISH {
            let price = legendary_sell_price(id);
            assert!(price > 0, "Legendary {} should have a positive sell price", id);
        }
    }

    #[test]
    fn test_try_roll_legendary_wrong_conditions_returns_none() {
        // Crimson King requires Beach + Summer. Try with Farm + Winter.
        // Over many rolls, should never return crimson_king.
        for _ in 0..1000 {
            let result = try_roll_legendary(MapId::Farm, Season::Winter);
            if let Some((id, _)) = result {
                assert_ne!(id, "crimson_king", "Should not roll crimson_king on Farm/Winter");
            }
        }
    }
}
