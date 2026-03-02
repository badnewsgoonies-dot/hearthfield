//! Data layer — populates all registries at game startup.
//!
//! This plugin runs in OnEnter(GameState::Loading), fills every registry
//! (ItemRegistry, CropRegistry, FishRegistry, RecipeRegistry, NpcRegistry,
//! ShopData) from the hard-coded game-design data defined in submodules,
//! then transitions the game into GameState::MainMenu.
//!
//! No other domain needs to seed these resources. All domain plugins can
//! safely read them once GameState has advanced past Loading.

mod items;
mod crops;
mod fish;
mod recipes;
mod npcs;
mod shops;

use bevy::prelude::*;
use crate::shared::*;

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), load_all_data);
    }
}

/// Single system that populates every registry and then transitions to MainMenu.
///
/// Order of population matters only in that ItemRegistry should be ready
/// before other registries reference item IDs — which they do internally
/// (string IDs), so there is no hard dependency on execution order here.
fn load_all_data(
    mut item_registry: ResMut<ItemRegistry>,
    mut crop_registry: ResMut<CropRegistry>,
    mut fish_registry: ResMut<FishRegistry>,
    mut recipe_registry: ResMut<RecipeRegistry>,
    mut npc_registry: ResMut<NpcRegistry>,
    mut shop_data: ResMut<ShopData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    info!("DataPlugin: populating registries…");

    items::populate_items(&mut item_registry);
    info!(
        "  Items loaded: {}",
        item_registry.items.len()
    );

    crops::populate_crops(&mut crop_registry);
    info!(
        "  Crops loaded: {}",
        crop_registry.crops.len()
    );

    fish::populate_fish(&mut fish_registry);
    info!(
        "  Fish loaded: {}",
        fish_registry.fish.len()
    );

    recipes::populate_recipes(&mut recipe_registry);
    info!(
        "  Recipes loaded: {}",
        recipe_registry.recipes.len()
    );

    npcs::populate_npcs(&mut npc_registry);
    info!(
        "  NPCs loaded: {}, Schedules loaded: {}",
        npc_registry.npcs.len(),
        npc_registry.schedules.len()
    );

    shops::populate_shops(&mut shop_data);
    let total_listings: usize = shop_data.listings.values().map(|v| v.len()).sum();
    info!(
        "  Shop listings loaded: {} across {} shops",
        total_listings,
        shop_data.listings.len()
    );

    info!("DataPlugin: all registries populated. Transitioning to MainMenu.");
    next_state.set(GameState::MainMenu);
}

#[cfg(test)]
mod tests {
    use super::{crops, fish, items};
    use crate::shared::{CropRegistry, FishRegistry, ItemRegistry};

    #[test]
    fn all_crop_definitions_have_valid_data() {
        let mut crop_registry = CropRegistry::default();
        crops::populate_crops(&mut crop_registry);

        assert!(
            !crop_registry.crops.is_empty(),
            "crop registry should not be empty"
        );

        for crop in crop_registry.crops.values() {
            assert!(
                !crop.id.trim().is_empty(),
                "crop has empty id: {:?}",
                crop
            );
            assert!(
                !crop.name.trim().is_empty(),
                "crop has empty name: id={}",
                crop.id
            );
            assert!(
                !crop.growth_days.is_empty(),
                "crop has no growth stages: id={}",
                crop.id
            );
            assert!(
                crop.growth_days.iter().all(|&d| d > 0),
                "crop has non-positive growth stage days: id={}, growth_days={:?}",
                crop.id,
                crop.growth_days
            );
            assert!(
                crop.sell_price > 0,
                "crop has non-positive sell_price: id={}, sell_price={}",
                crop.id,
                crop.sell_price
            );
            assert!(
                !crop.seasons.is_empty(),
                "crop has no seasons set: id={}",
                crop.id
            );
        }
    }

    #[test]
    fn all_fish_definitions_have_valid_data() {
        let mut fish_registry = FishRegistry::default();
        fish::populate_fish(&mut fish_registry);

        assert!(
            !fish_registry.fish.is_empty(),
            "fish registry should not be empty"
        );

        for fish_def in fish_registry.fish.values() {
            assert!(
                !fish_def.id.trim().is_empty(),
                "fish has empty id: {:?}",
                fish_def
            );
            assert!(
                !fish_def.name.trim().is_empty(),
                "fish has empty name: id={}",
                fish_def.id
            );

            let d = fish_def.difficulty;
            let valid_difficulty = (0.0..=1.0).contains(&d) || (1.0..=100.0).contains(&d);
            assert!(
                valid_difficulty,
                "fish has out-of-range difficulty: id={}, difficulty={}",
                fish_def.id,
                d
            );

            assert!(
                fish_def.sell_price > 0,
                "fish has non-positive sell_price: id={}, sell_price={}",
                fish_def.id,
                fish_def.sell_price
            );
        }
    }

    #[test]
    fn item_registry_contains_crop_harvests_and_fish_items() {
        let mut item_registry = ItemRegistry::default();
        let mut crop_registry = CropRegistry::default();
        let mut fish_registry = FishRegistry::default();

        items::populate_items(&mut item_registry);
        crops::populate_crops(&mut crop_registry);
        fish::populate_fish(&mut fish_registry);

        for crop in crop_registry.crops.values() {
            assert!(
                item_registry.items.contains_key(&crop.harvest_id),
                "missing crop harvest item in item registry: crop_id={}, harvest_id={}",
                crop.id,
                crop.harvest_id
            );
        }

        for fish_def in fish_registry.fish.values() {
            assert!(
                item_registry.items.contains_key(&fish_def.id),
                "missing fish item in item registry: fish_id={}",
                fish_def.id
            );
        }
    }

    #[test]
    fn crop_and_fish_ids_have_no_duplicates() {
        let mut crop_registry = CropRegistry::default();
        let mut fish_registry = FishRegistry::default();

        crops::populate_crops(&mut crop_registry);
        fish::populate_fish(&mut fish_registry);

        // Duplicate IDs in populate_* would overwrite previous entries in the HashMap.
        // These expected counts lock in that no such overwrites occurred.
        const EXPECTED_CROP_COUNT: usize = 15;
        const EXPECTED_FISH_COUNT: usize = 20;

        assert_eq!(
            crop_registry.crops.len(),
            EXPECTED_CROP_COUNT,
            "crop registry size mismatch; this can indicate duplicate crop IDs"
        );
        assert_eq!(
            fish_registry.fish.len(),
            EXPECTED_FISH_COUNT,
            "fish registry size mismatch; this can indicate duplicate fish IDs"
        );
    }
}
