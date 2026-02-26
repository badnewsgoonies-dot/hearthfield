use bevy::prelude::*;
use crate::shared::*;

mod machines;
mod recipes;
mod bench;
mod cooking;
mod unlock;

pub use machines::*;
pub use recipes::*;
pub use bench::*;
pub use cooking::*;
pub use unlock::*;

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
            // Startup: register default recipe unlocks
            .add_systems(OnEnter(GameState::Playing), unlock::initialize_unlocked_recipes)
            // Playing state: processing machines advance on time, respond to day end
            .add_systems(
                Update,
                (
                    machines::tick_processing_machines,
                    machines::handle_insert_machine_input,
                    machines::handle_collect_machine_output,
                    machines::handle_day_end_processing,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Open/close crafting UI
            .add_systems(
                Update,
                bench::handle_open_crafting.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                bench::handle_close_crafting.run_if(in_state(GameState::Crafting)),
            )
            // Crafting state: crafting bench interaction
            .add_systems(
                Update,
                (
                    bench::handle_craft_item,
                    cooking::handle_cook_item,
                )
                    .run_if(in_state(GameState::Crafting)),
            )
            // Recipe unlocking
            .add_systems(
                Update,
                unlock::handle_unlock_recipe.run_if(in_state(GameState::Playing)),
            );
    }
}
