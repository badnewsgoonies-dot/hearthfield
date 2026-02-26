use bevy::prelude::*;
use crate::shared::*;

mod machines;
mod recipes;
mod bench;
mod cooking;
mod unlock;

pub use machines::{
    MachineType, ProcessingMachine, ProcessingMachineRegistry,
    InsertMachineInputEvent, CollectMachineOutputEvent,
};
pub use recipes::{
    make_crafting_recipe, make_cooking_recipe, populate_recipe_registry,
    ALL_CRAFTING_RECIPE_IDS, ALL_COOKING_RECIPE_IDS,
};
pub use bench::{CraftingUiState, OpenCraftingEvent, CloseCraftingEvent, CraftItemEvent};
pub use unlock::UnlockRecipeEvent;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Crafting-specific resources
            .init_resource::<CraftingUiState>()
            .init_resource::<ProcessingMachineRegistry>()
            // Crafting-specific events
            .add_event::<CraftItemEvent>()
            .add_event::<OpenCraftingEvent>()
            .add_event::<CloseCraftingEvent>()
            .add_event::<InsertMachineInputEvent>()
            .add_event::<CollectMachineOutputEvent>()
            .add_event::<UnlockRecipeEvent>()
            // Startup: register default recipe unlocks once we enter Playing
            .add_systems(OnEnter(GameState::Playing), unlock::initialize_unlocked_recipes)
            // Playing state systems
            .add_systems(
                Update,
                (
                    // Processing machine real-time tick
                    machines::tick_processing_machines,
                    // Machine interaction (insert / collect)
                    machines::handle_insert_machine_input,
                    machines::handle_collect_machine_output,
                    // Day-end: finalize any machines that finished
                    machines::handle_day_end_processing,
                    // Open crafting bench
                    bench::handle_open_crafting,
                    // Recipe unlock checks
                    unlock::check_milestone_recipe_unlocks,
                    unlock::check_friendship_recipe_unlocks,
                    unlock::handle_unlock_recipe,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Crafting state systems
            .add_systems(
                Update,
                (
                    // Close crafting UI
                    bench::handle_close_crafting,
                    // Craft (non-cooking) items
                    bench::handle_craft_item,
                    // Cook food items
                    cooking::handle_cook_item,
                )
                    .run_if(in_state(GameState::Crafting)),
            );
    }
}
