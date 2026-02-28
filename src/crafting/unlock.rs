use bevy::prelude::*;
use crate::shared::*;
use super::recipes::{
    make_crafting_recipe, make_cooking_recipe,
    ALL_CRAFTING_RECIPE_IDS, ALL_COOKING_RECIPE_IDS,
};

// ──────────────────────────────────────────────────────────────────────────────
// UNLOCK EVENT
// ──────────────────────────────────────────────────────────────────────────────

/// Send this event to unlock a recipe for the player.
/// Any domain can send this (e.g. economy when buying a recipe, npcs on friendship reward).
#[derive(Event, Debug, Clone)]
pub struct UnlockRecipeEvent {
    pub recipe_id: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// FRIENDSHIP UNLOCK TABLE
// ──────────────────────────────────────────────────────────────────────────────

/// Maps (npc_id, hearts_required) → recipe_id that is unlocked.
/// When the relationships system grants hearts that cross a threshold, it sends
/// an UnlockRecipeEvent — but the crafting domain defines WHICH recipe is earned.
pub struct FriendshipRecipeUnlock {
    pub npc_id: &'static str,
    pub hearts: u8,
    pub recipe_id: &'static str,
}

pub const FRIENDSHIP_RECIPE_UNLOCKS: &[FriendshipRecipeUnlock] = &[
    FriendshipRecipeUnlock { npc_id: "elena",  hearts: 3, recipe_id: "salad" },
    FriendshipRecipeUnlock { npc_id: "elena",  hearts: 7, recipe_id: "fruit_salad" },
    FriendshipRecipeUnlock { npc_id: "marco", hearts: 2, recipe_id: "pancakes" },
    FriendshipRecipeUnlock { npc_id: "marco", hearts: 6, recipe_id: "pizza" },
    FriendshipRecipeUnlock { npc_id: "vera",   hearts: 4, recipe_id: "pumpkin_soup" },
    FriendshipRecipeUnlock { npc_id: "vera",   hearts: 8, recipe_id: "cheese_omelette" },
    FriendshipRecipeUnlock { npc_id: "tom",    hearts: 3, recipe_id: "fish_stew" },
    FriendshipRecipeUnlock { npc_id: "lyra",   hearts: 2, recipe_id: "spaghetti" },
    FriendshipRecipeUnlock { npc_id: "lyra",   hearts: 6, recipe_id: "ice_cream" },
    FriendshipRecipeUnlock { npc_id: "orion",  hearts: 5, recipe_id: "bomb" },
    FriendshipRecipeUnlock { npc_id: "orion",  hearts: 9, recipe_id: "mega_bomb" },
    FriendshipRecipeUnlock { npc_id: "seraph", hearts: 4, recipe_id: "quality_sprinkler" },
    FriendshipRecipeUnlock { npc_id: "seraph", hearts: 8, recipe_id: "bee_house" },
    FriendshipRecipeUnlock { npc_id: "kai",    hearts: 3, recipe_id: "keg" },
    FriendshipRecipeUnlock { npc_id: "kai",    hearts: 7, recipe_id: "oil_maker" },
    FriendshipRecipeUnlock { npc_id: "nina",   hearts: 6, recipe_id: "cake" },
    FriendshipRecipeUnlock { npc_id: "nina",   hearts: 9, recipe_id: "cookie" },
    FriendshipRecipeUnlock { npc_id: "gregor", hearts: 3, recipe_id: "furnace" },
    FriendshipRecipeUnlock { npc_id: "gregor", hearts: 6, recipe_id: "preserves_jar" },
    FriendshipRecipeUnlock { npc_id: "gregor", hearts: 9, recipe_id: "cheese_press" },
];

// ──────────────────────────────────────────────────────────────────────────────
// MILESTONE UNLOCK TABLE (unlocked based on game progress)
// ──────────────────────────────────────────────────────────────────────────────

/// These recipes unlock when the player first acquires a certain item
/// (triggered externally via ItemPickupEvent → UnlockRecipeEvent chain in other domains,
/// or seeded here for data-driven reference).
pub struct MilestoneRecipeUnlock {
    pub trigger_item: &'static str,
    pub recipe_id: &'static str,
}

pub const MILESTONE_RECIPE_UNLOCKS: &[MilestoneRecipeUnlock] = &[
    MilestoneRecipeUnlock { trigger_item: "copper_ore",  recipe_id: "furnace" },
    MilestoneRecipeUnlock { trigger_item: "copper_bar",  recipe_id: "sprinkler" },
    MilestoneRecipeUnlock { trigger_item: "copper_bar",  recipe_id: "recycler" },
    MilestoneRecipeUnlock { trigger_item: "iron_bar",    recipe_id: "loom" },
    MilestoneRecipeUnlock { trigger_item: "iron_bar",    recipe_id: "bomb" },
    MilestoneRecipeUnlock { trigger_item: "gold_bar",    recipe_id: "seed_maker" },
    MilestoneRecipeUnlock { trigger_item: "gold_bar",    recipe_id: "worm_bin" },
    MilestoneRecipeUnlock { trigger_item: "hardwood",    recipe_id: "oil_maker" },
    MilestoneRecipeUnlock { trigger_item: "maple_syrup", recipe_id: "bee_house" },
    MilestoneRecipeUnlock { trigger_item: "truffle",     recipe_id: "oil_maker" },
];

// ──────────────────────────────────────────────────────────────────────────────
// SHOP-UNLOCKABLE RECIPES
// ──────────────────────────────────────────────────────────────────────────────

/// Recipe IDs that can be purchased from the general store.
/// The economy domain manages actual shop listings; this is for reference.
pub const SHOP_RECIPE_IDS: &[&str] = &[
    "fence",
    "path",
    "chest",
    "torch",
    "campfire",
    "preserves_jar",
    "cheese_press",
    "loom",
    "bread",
    "cooked_fish",
    "baked_potato",
];

// ──────────────────────────────────────────────────────────────────────────────
// SYSTEMS
// ──────────────────────────────────────────────────────────────────────────────

/// Runs once when entering Playing state.
/// Seeds the UnlockedRecipes resource with all default-unlocked recipes.
pub fn initialize_unlocked_recipes(
    mut unlocked: ResMut<UnlockedRecipes>,
    mut recipe_registry: ResMut<RecipeRegistry>,
) {
    // Populate recipe registry if not already done (data plugin may have done this)
    if recipe_registry.recipes.is_empty() {
        super::recipes::populate_recipe_registry(&mut recipe_registry);
        info!(
            "CraftingPlugin: seeded RecipeRegistry with {} recipes",
            recipe_registry.recipes.len()
        );
    }

    // Grant all default recipes that aren't already unlocked
    let mut newly_unlocked = 0usize;
    for (id, recipe) in &recipe_registry.recipes {
        if recipe.unlocked_by_default && !unlocked.ids.contains(id) {
            unlocked.ids.push(id.clone());
            newly_unlocked += 1;
        }
    }

    if newly_unlocked > 0 {
        info!(
            "CraftingPlugin: unlocked {} default recipes ({} total)",
            newly_unlocked,
            unlocked.ids.len()
        );
    }
}

/// Handles UnlockRecipeEvent — adds a recipe to the player's unlocked list.
pub fn handle_unlock_recipe(
    mut events: EventReader<UnlockRecipeEvent>,
    mut unlocked: ResMut<UnlockedRecipes>,
    recipe_registry: Res<RecipeRegistry>,
) {
    for event in events.read() {
        let recipe_id = &event.recipe_id;

        if unlocked.ids.contains(recipe_id) {
            // Already unlocked — no-op (not an error)
            continue;
        }

        if recipe_registry.recipes.contains_key(recipe_id.as_str()) {
            unlocked.ids.push(recipe_id.clone());
            info!("Unlocked recipe: '{}'", recipe_id);
        } else {
            warn!("UnlockRecipeEvent: recipe '{}' not found in registry", recipe_id);
        }
    }
}

/// Monitors ItemPickupEvent and sends UnlockRecipeEvents for milestone unlocks.
/// This allows the crafting domain to react to items acquired anywhere in the game.
pub fn check_milestone_recipe_unlocks(
    mut item_events: EventReader<ItemPickupEvent>,
    unlocked: Res<UnlockedRecipes>,
    recipe_registry: Res<RecipeRegistry>,
    mut unlock_events: EventWriter<UnlockRecipeEvent>,
) {
    for event in item_events.read() {
        for milestone in MILESTONE_RECIPE_UNLOCKS {
            if event.item_id == milestone.trigger_item {
                let recipe_id = milestone.recipe_id.to_string();
                // Only emit if not already unlocked and recipe exists
                if !unlocked.ids.contains(&recipe_id)
                    && recipe_registry.recipes.contains_key(&recipe_id)
                {
                    info!(
                        "Milestone unlock: picking up '{}' unlocks recipe '{}'",
                        event.item_id, recipe_id
                    );
                    unlock_events.send(UnlockRecipeEvent { recipe_id });
                }
            }
        }
    }
}

/// Checks friendship levels and sends UnlockRecipeEvents when thresholds are crossed.
/// Runs in Playing state — checks after any relationship change.
pub fn check_friendship_recipe_unlocks(
    relationships: Res<Relationships>,
    unlocked: Res<UnlockedRecipes>,
    recipe_registry: Res<RecipeRegistry>,
    mut unlock_events: EventWriter<UnlockRecipeEvent>,
) {
    if !relationships.is_changed() {
        return;
    }

    for entry in FRIENDSHIP_RECIPE_UNLOCKS {
        let current_hearts = relationships.hearts(entry.npc_id);
        if current_hearts >= entry.hearts {
            let recipe_id = entry.recipe_id.to_string();
            if !unlocked.ids.contains(&recipe_id)
                && recipe_registry.recipes.contains_key(&recipe_id)
            {
                info!(
                    "Friendship unlock: {} at {} hearts unlocks recipe '{}'",
                    entry.npc_id, entry.hearts, recipe_id
                );
                unlock_events.send(UnlockRecipeEvent { recipe_id });
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// UNUSED IDS SILENCER (ensures all constants compile without dead_code warnings)
// ──────────────────────────────────────────────────────────────────────────────
#[allow(dead_code)]
fn _use_constants() {
    let _ = ALL_CRAFTING_RECIPE_IDS;
    let _ = ALL_COOKING_RECIPE_IDS;
    let _ = SHOP_RECIPE_IDS;
    let _: Option<Recipe> = make_crafting_recipe("sprinkler");
    let _: Option<Recipe> = make_cooking_recipe("fried_egg");
}
