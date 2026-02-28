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
