use bevy::prelude::*;
use crate::shared::*;

// ──────────────────────────────────────────────────────────────────────────────
// CRAFTING UI STATE
// ──────────────────────────────────────────────────────────────────────────────

/// Resource tracking the state of the crafting UI while in GameState::Crafting.
#[derive(Resource, Debug, Clone, Default)]
pub struct CraftingUiState {
    /// All recipes visible to the player (non-cooking, unlocked).
    pub available_recipes: Vec<String>,
    /// Index of currently highlighted recipe.
    pub selected_index: usize,
    /// True if the crafting UI was opened from a cooking surface (kitchen).
    pub is_cooking_mode: bool,
    /// Notification message shown after crafting (e.g. "Crafted Torch x3").
    pub feedback_message: Option<String>,
    /// Timer to clear feedback message.
    pub feedback_timer: f32,
}

impl CraftingUiState {
    pub fn selected_recipe_id(&self) -> Option<&str> {
        self.available_recipes
            .get(self.selected_index)
            .map(String::as_str)
    }

    pub fn move_selection_up(&mut self) {
        if !self.available_recipes.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.available_recipes.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn move_selection_down(&mut self) {
        if !self.available_recipes.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.available_recipes.len();
        }
    }

    pub fn set_feedback(&mut self, msg: impl Into<String>) {
        self.feedback_message = Some(msg.into());
        self.feedback_timer = 3.0; // seconds
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// EVENTS
// ──────────────────────────────────────────────────────────────────────────────

/// Send to open the crafting bench (transitions to GameState::Crafting).
#[derive(Event, Debug, Clone)]
pub struct OpenCraftingEvent {
    /// True = opened from kitchen (cooking mode), False = crafting bench.
    pub cooking_mode: bool,
}

/// Send to close the crafting UI and return to Playing.
#[derive(Event, Debug, Clone)]
pub struct CloseCraftingEvent;

/// Send to request crafting a recipe. UI sends this when the player confirms.
#[derive(Event, Debug, Clone)]
pub struct CraftItemEvent {
    pub recipe_id: String,
}

// ──────────────────────────────────────────────────────────────────────────────
// SYSTEMS
// ──────────────────────────────────────────────────────────────────────────────

/// Runs in Playing — listens for OpenCraftingEvent and transitions to Crafting state.
/// Populates the CraftingUiState with the list of unlocked recipes.
pub fn handle_open_crafting(
    mut events: EventReader<OpenCraftingEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    mut ui_state: ResMut<CraftingUiState>,
    unlocked: Res<UnlockedRecipes>,
    recipe_registry: Res<RecipeRegistry>,
) {
    for event in events.read() {
        let cooking_mode = event.cooking_mode;

        // Build the list of available recipes filtered by:
        //   1. Unlocked
        //   2. Cooking/crafting mode match
        let mut available: Vec<String> = unlocked
            .ids
            .iter()
            .filter_map(|id| recipe_registry.recipes.get(id))
            .filter(|r| r.is_cooking == cooking_mode)
            .map(|r| r.id.clone())
            .collect();

        // Sort alphabetically for a consistent UI order
        available.sort();

        *ui_state = CraftingUiState {
            available_recipes: available,
            selected_index: 0,
            is_cooking_mode: cooking_mode,
            feedback_message: None,
            feedback_timer: 0.0,
        };

        info!(
            "Opening {} UI with {} recipes",
            if cooking_mode { "cooking" } else { "crafting" },
            ui_state.available_recipes.len()
        );

        next_state.set(GameState::Crafting);
    }
}

/// Runs in Crafting — listens for CloseCraftingEvent and returns to Playing.
pub fn handle_close_crafting(
    mut events: EventReader<CloseCraftingEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    player_input: Res<PlayerInput>,
) {
    // Close on explicit event OR on Escape key
    let esc_pressed = player_input.ui_cancel;
    let has_event = events.read().next().is_some();

    if esc_pressed || has_event {
        info!("Closing crafting UI");
        next_state.set(GameState::Playing);
    }
}

/// Runs in Crafting (non-cooking mode) — processes a CraftItemEvent.
/// Validates ingredients, consumes them, produces the result item.
pub fn handle_craft_item(
    mut events: EventReader<CraftItemEvent>,
    mut inventory: ResMut<Inventory>,
    recipe_registry: Res<RecipeRegistry>,
    item_registry: Res<ItemRegistry>,
    unlocked: Res<UnlockedRecipes>,
    mut ui_state: ResMut<CraftingUiState>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    // Also handle keyboard input for navigation and confirming craft
    // (The UI plugin handles the actual rendering; we only handle logic here)

    for event in events.read() {
        let recipe_id = &event.recipe_id;

        // Verify unlocked
        if !unlocked.ids.contains(recipe_id) {
            warn!("Recipe '{}' is not unlocked", recipe_id);
            ui_state.set_feedback(format!("Recipe not unlocked: {}", recipe_id));
            continue;
        }

        let Some(recipe) = recipe_registry.recipes.get(recipe_id) else {
            warn!("Recipe '{}' not found in registry", recipe_id);
            continue;
        };

        // Skip cooking recipes here (handled by cooking system)
        if recipe.is_cooking {
            continue;
        }

        // Check all ingredients are present
        if !has_all_ingredients(&inventory, recipe) {
            let missing = missing_ingredients_description(&inventory, recipe);
            warn!("Cannot craft '{}' — missing: {}", recipe.name, missing);
            ui_state.set_feedback(format!("Missing materials: {}", missing));
            sfx_events.send(PlaySfxEvent {
                sfx_id: "craft_fail".to_string(),
            });
            continue;
        }

        // Consume all ingredients
        consume_ingredients(&mut inventory, recipe);

        // Produce the result
        let max_stack = item_registry
            .get(&recipe.result)
            .map(|d| d.stack_size)
            .unwrap_or(99);

        let leftover = inventory.try_add(&recipe.result, recipe.result_quantity, max_stack);
        if leftover > 0 {
            // Inventory full — refund ingredients
            warn!("Inventory full after crafting '{}' — refunding materials", recipe.name);
            refund_ingredients(&mut inventory, recipe, &item_registry);
            ui_state.set_feedback("Inventory is full!".to_string());
            continue;
        }

        // Emit pickup event so other systems (UI, etc.) know an item was gained
        pickup_events.send(ItemPickupEvent {
            item_id: recipe.result.clone(),
            quantity: recipe.result_quantity,
        });

        let feedback = if recipe.result_quantity > 1 {
            format!("Crafted {} x{}", recipe.name, recipe.result_quantity)
        } else {
            format!("Crafted {}", recipe.name)
        };
        info!("{}", feedback);
        ui_state.set_feedback(feedback);

        sfx_events.send(PlaySfxEvent {
            sfx_id: "craft_success".to_string(),
        });
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// HELPER FUNCTIONS
// ──────────────────────────────────────────────────────────────────────────────

/// Returns true if the inventory has all required ingredients for a recipe.
pub fn has_all_ingredients(inventory: &Inventory, recipe: &Recipe) -> bool {
    for (item_id, qty) in &recipe.ingredients {
        // "any_fish" is a wildcard — resolved separately in cooking
        if item_id == "any_fish" {
            continue;
        }
        if !inventory.has(item_id, *qty) {
            return false;
        }
    }
    true
}

/// Returns a human-readable list of missing ingredients.
fn missing_ingredients_description(inventory: &Inventory, recipe: &Recipe) -> String {
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

/// Consume all ingredients from inventory.
pub fn consume_ingredients(inventory: &mut Inventory, recipe: &Recipe) {
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        let removed = inventory.try_remove(item_id, *qty);
        if removed < *qty {
            warn!(
                "consume_ingredients: only removed {} of {} '{}' — inventory may be inconsistent",
                removed, qty, item_id
            );
        }
    }
}

/// Refund all ingredients back to inventory after a failed craft.
pub fn refund_ingredients(inventory: &mut Inventory, recipe: &Recipe, registry: &ItemRegistry) {
    for (item_id, qty) in &recipe.ingredients {
        if item_id == "any_fish" {
            continue;
        }
        let max_stack = registry.get(item_id).map(|d| d.stack_size).unwrap_or(99);
        inventory.try_add(item_id, *qty, max_stack);
    }
}
