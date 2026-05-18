use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Consume all ingredients from inventory.
pub fn crafting_consume_system(inventory: &mut Inventory, recipe: &Recipe) {
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


