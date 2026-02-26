use bevy::prelude::*;
use crate::shared::*;
use super::bench::{CraftItemEvent, CraftingUiState};

// ──────────────────────────────────────────────────────────────────────────────
// KITCHEN STATE
// ──────────────────────────────────────────────────────────────────────────────

/// Item ids that count as "any fish" for cooking purposes.
/// The cooking system resolves the wildcard by scanning the player inventory.
const FISH_IDS: &[&str] = &[
    "sardine", "catfish", "tuna", "pike", "tilapia",
    "woodskip", "pufferfish", "sunfish", "super_cucumber", "ghostfish",
    "eel", "octopus", "red_snapper", "squid", "sea_cucumber",
    "tiger_trout", "largemouth_bass", "smallmouth_bass", "carp", "bullhead",
];

/// Runs in Crafting mode when cooking_mode == true.
/// Very similar to crafting but:
///   1. Requires the player to have a kitchen (house upgrade flag).
///   2. Handles the "any_fish" wildcard ingredient.
///   3. Produces food items that restore stamina.
pub fn handle_cook_item(
    mut events: EventReader<CraftItemEvent>,
    mut inventory: ResMut<Inventory>,
    recipe_registry: Res<RecipeRegistry>,
    item_registry: Res<ItemRegistry>,
    unlocked: Res<UnlockedRecipes>,
    mut ui_state: ResMut<CraftingUiState>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut player_state: ResMut<PlayerState>,
) {
    for event in events.read() {
        let recipe_id = &event.recipe_id;

        // Only handle cooking recipes in cooking mode
        if !ui_state.is_cooking_mode {
            continue;
        }

        // Verify unlocked
        if !unlocked.ids.contains(recipe_id) {
            warn!("Cooking recipe '{}' is not unlocked", recipe_id);
            ui_state.set_feedback(format!("Recipe not unlocked: {}", recipe_id));
            continue;
        }

        let Some(recipe) = recipe_registry.recipes.get(recipe_id) else {
            warn!("Cooking recipe '{}' not found in registry", recipe_id);
            continue;
        };

        if !recipe.is_cooking {
            // Crafting recipe, handled by bench system
            continue;
        }

        // Resolve "any_fish" wildcard
        let fish_item = if recipe.ingredients.iter().any(|(id, _)| id == "any_fish") {
            find_any_fish_in_inventory(&inventory)
        } else {
            None
        };

        // Check ingredients (skip wildcard — handle separately)
        if !has_all_non_wildcard_ingredients(&inventory, recipe) {
            let missing = missing_non_wildcard_description(&inventory, recipe);
            warn!("Cannot cook '{}' — missing: {}", recipe.name, missing);
            ui_state.set_feedback(format!("Missing ingredients: {}", missing));
            sfx_events.write(PlaySfxEvent {
                sfx_id: "craft_fail".to_string(),
            });
            continue;
        }

        // Validate wildcard ingredient
        let has_any_fish_ingredient = recipe
            .ingredients
            .iter()
            .any(|(id, _)| id == "any_fish");

        if has_any_fish_ingredient {
            if fish_item.is_none() {
                warn!("Cannot cook '{}' — no fish in inventory", recipe.name);
                ui_state.set_feedback("Need fish to cook this recipe!".to_string());
                sfx_events.write(PlaySfxEvent {
                    sfx_id: "craft_fail".to_string(),
                });
                continue;
            }
        }

        // Consume normal ingredients
        consume_non_wildcard_ingredients(&mut inventory, recipe);

        // Consume fish wildcard if needed
        if let Some(ref fish_id) = fish_item {
            if has_any_fish_ingredient {
                inventory.try_remove(fish_id, 1);
                info!("Consumed fish '{}' for cooking '{}'", fish_id, recipe.name);
            }
        }

        // Produce the result
        let max_stack = item_registry
            .get(&recipe.result)
            .map(|d| d.stack_size)
            .unwrap_or(99);

        let leftover = inventory.try_add(&recipe.result, recipe.result_quantity, max_stack);
        if leftover > 0 {
            warn!("Inventory full after cooking '{}' — refunding materials", recipe.name);
            // Refund normal ingredients
            refund_non_wildcard_ingredients(&mut inventory, recipe, &item_registry);
            // Refund fish
            if let Some(ref fish_id) = fish_item {
                if has_any_fish_ingredient {
                    let fish_stack = item_registry
                        .get(fish_id.as_str())
                        .map(|d| d.stack_size)
                        .unwrap_or(99);
                    inventory.try_add(fish_id, 1, fish_stack);
                }
            }
            ui_state.set_feedback("Inventory is full!".to_string());
            continue;
        }

        // Emit pickup event
        pickup_events.write(ItemPickupEvent {
            item_id: recipe.result.clone(),
            quantity: recipe.result_quantity,
        });

        // Apply stamina restoration if the result item is edible
        if let Some(item_def) = item_registry.get(&recipe.result) {
            if item_def.edible && item_def.energy_restore > 0.0 {
                let restore = item_def.energy_restore;
                let new_stamina = (player_state.stamina + restore).min(player_state.max_stamina);
                info!(
                    "Cooking '{}' restored {:.0} stamina (from {:.0} to {:.0})",
                    recipe.name, restore, player_state.stamina, new_stamina
                );
                player_state.stamina = new_stamina;
            }
        }

        let feedback = if recipe.result_quantity > 1 {
            format!("Cooked {} x{}", recipe.name, recipe.result_quantity)
        } else {
            format!("Cooked {}!", recipe.name)
        };
        info!("{}", feedback);
        ui_state.set_feedback(feedback);

        sfx_events.write(PlaySfxEvent {
            sfx_id: "cook_success".to_string(),
        });

        // Cooking also costs a small amount of stamina (fire-tending effort)
        stamina_events.write(StaminaDrainEvent { amount: 2.0 });
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// WILDCARD & INGREDIENT HELPERS (cooking-specific)
// ──────────────────────────────────────────────────────────────────────────────

/// Finds the first fish item in the player's inventory, returns its item id.
fn find_any_fish_in_inventory(inventory: &Inventory) -> Option<ItemId> {
    for slot in inventory.slots.iter().flatten() {
        if FISH_IDS.contains(&slot.item_id.as_str()) {
            return Some(slot.item_id.clone());
        }
        // Also accept items with category-based naming convention (e.g., anything ending in _fish)
        if slot.item_id.ends_with("_fish") || slot.item_id.starts_with("fish_") {
            return Some(slot.item_id.clone());
        }
    }
    None
}

/// Check all non-wildcard ingredients.
fn has_all_non_wildcard_ingredients(inventory: &Inventory, recipe: &Recipe) -> bool {
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        if !inventory.has(item_id, *qty) {
            return false;
        }
    }
    true
}

/// Human-readable list of missing non-wildcard ingredients.
fn missing_non_wildcard_description(inventory: &Inventory, recipe: &Recipe) -> String {
    let mut parts = Vec::new();
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        let have = inventory.count(item_id) as u8;
        if have < *qty {
            parts.push(format!("{} (have {}/{})", item_id, have, qty));
        }
    }
    parts.join(", ")
}

/// Consume all non-wildcard ingredients.
fn consume_non_wildcard_ingredients(inventory: &mut Inventory, recipe: &Recipe) {
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        let removed = inventory.try_remove(item_id, *qty);
        if removed < *qty {
            warn!(
                "consume_non_wildcard_ingredients: only removed {} of {} '{}'",
                removed, qty, item_id
            );
        }
    }
}

/// Refund all non-wildcard ingredients.
fn refund_non_wildcard_ingredients(
    inventory: &mut Inventory,
    recipe: &Recipe,
    registry: &ItemRegistry,
) {
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        let max_stack = registry.get(item_id).map(|d| d.stack_size).unwrap_or(99);
        inventory.try_add(item_id, *qty, max_stack);
    }
}

