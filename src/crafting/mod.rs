#![allow(dead_code, unused_imports)]

use crate::shared::*;
use bevy::prelude::*;

mod bench;
mod buffs;
mod cooking;
pub mod machines;
mod recipes;
mod unlock;

pub use bench::{
    consume_ingredients, handle_craft_item, handle_open_crafting, has_all_ingredients,
    refund_ingredients, CraftItemEvent, CraftingUiState, OpenCraftingEvent,
};
pub use buffs::food_buff_for_item;
pub use machines::{
    item_to_machine_type, machine_atlas_index, CollectMachineOutputEvent, InsertMachineInputEvent,
    MachineAnimTimer, MachineParticle, MachineType, PlaceMachineEvent, ProceduralMachineSprites,
    ProcessingMachine, ProcessingMachineRegistry, SavedMachine,
};
pub use recipes::{
    make_cooking_recipe, make_crafting_recipe, populate_recipe_registry, ALL_COOKING_RECIPE_IDS,
    ALL_CRAFTING_RECIPE_IDS,
};
pub use unlock::UnlockRecipeEvent;

pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app
            // Crafting-specific resources
            .init_resource::<CraftingUiState>()
            .init_resource::<ProcessingMachineRegistry>()
            .init_resource::<ProceduralMachineSprites>()
            // Crafting-specific events
            .add_event::<CraftItemEvent>()
            .add_event::<OpenCraftingEvent>()
            .add_event::<InsertMachineInputEvent>()
            .add_event::<CollectMachineOutputEvent>()
            .add_event::<PlaceMachineEvent>()
            .add_event::<UnlockRecipeEvent>()
            // Startup: register default recipe unlocks once we enter Playing
            .add_systems(
                OnEnter(GameState::Playing),
                unlock::initialize_unlocked_recipes,
            )
            // Playing state systems
            .add_systems(
                Update,
                (
                    // Processing machine real-time tick
                    machines::tick_processing_machines,
                    // Machine placement from hotbar
                    machines::handle_place_machine,
                    // Machine interaction (insert / collect)
                    machines::handle_insert_machine_input,
                    machines::handle_collect_machine_output,
                    // Day-end: finalize any machines that finished
                    machines::handle_day_end_processing,
                    // C key → open crafting (must run before handle_open_crafting)
                    bench::trigger_crafting_key.before(bench::handle_open_crafting),
                    // Open crafting bench
                    bench::handle_open_crafting,
                    // Recipe unlock checks
                    unlock::check_milestone_recipe_unlocks,
                    unlock::check_friendship_recipe_unlocks,
                    unlock::handle_unlock_recipe,
                    // Food buff systems
                    buffs::handle_eat_food,
                    buffs::tick_buff_durations,
                    buffs::apply_buff_effects,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Crafting state systems
            .add_systems(
                Update,
                (
                    // Craft (non-cooking) items
                    bench::handle_craft_item,
                    // Cook food items
                    cooking::handle_cook_item,
                )
                    .run_if(in_state(GameState::Crafting)),
            )
            // ------------------------------------------------------------------
            // Machine sprite animation — runs in PostUpdate after spawning
            // ------------------------------------------------------------------
            .add_systems(
                PostUpdate,
                (
                    machines::animate_processing_machines,
                    machines::shake_active_machines,
                    machines::spawn_machine_particles,
                    machines::update_machine_particles,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Cleanup particles when leaving Playing state
            .add_systems(
                OnExit(GameState::Playing),
                machines::despawn_machine_particles,
            );
    }
}
