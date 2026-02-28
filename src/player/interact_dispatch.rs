//! World Interaction Dispatcher (F key)
//!
//! Single system that finds the nearest `Interactable` entity within range,
//! dispatches the appropriate event, and sets `InteractionClaimed` so that
//! legacy F-key systems skip.

use bevy::prelude::*;
use crate::shared::*;

// Domain event imports — use pub re-exports from domain mod.rs.
use crate::economy::shipping::ShipItemEvent;
use crate::crafting::{
    OpenCraftingEvent, CollectMachineOutputEvent, InsertMachineInputEvent, ProcessingMachine,
};

pub fn dispatch_world_interaction(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    inventory: Res<Inventory>,
    player_query: Query<&LogicalPosition, With<Player>>,
    interactable_query: Query<(&Transform, &Interactable, Entity)>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
    // Event writers
    mut ship_events: EventWriter<ShipItemEvent>,
    mut craft_events: EventWriter<OpenCraftingEvent>,
    mut machine_insert_events: EventWriter<InsertMachineInputEvent>,
    mut machine_collect_events: EventWriter<CollectMachineOutputEvent>,
    mut next_state: ResMut<NextState<GameState>>,
    // For machines
    machine_query: Query<&ProcessingMachine>,
    // UI feedback
    mut toast_events: EventWriter<ToastEvent>,
) {
    if input_blocks.is_blocked() || !player_input.interact {
        return;
    }

    let Ok(player_pos) = player_query.get_single() else {
        return;
    };
    let range = TILE_SIZE * 1.5;

    // Find nearest interactable within range.
    let mut best: Option<(f32, &Interactable, Entity)> = None;
    for (tf, inter, entity) in &interactable_query {
        let d = player_pos.0.distance(tf.translation.truncate());
        if d <= range {
            if best.is_none() || d < best.unwrap().0 {
                best = Some((d, inter, entity));
            }
        }
    }

    let Some((_dist, interactable, entity)) = best else {
        return;
    };

    match interactable.kind {
        InteractionKind::ShippingBin => {
            let slot_idx = inventory.selected_slot;
            let Some(ref slot) = inventory.slots.get(slot_idx).and_then(|s| s.as_ref()) else {
                interaction_claimed.0 = true;
                toast_events.send(ToastEvent {
                    message: "No item selected to ship.".into(),
                    duration_secs: 2.0,
                });
                return;
            };
            interaction_claimed.0 = true;
            ship_events.send(ShipItemEvent {
                item_id: slot.item_id.clone(),
                quantity: 1,
            });
        }

        InteractionKind::CraftingBench => {
            interaction_claimed.0 = true;
            craft_events.send(OpenCraftingEvent {
                cooking_mode: false,
            });
        }

        InteractionKind::Machine => {
            interaction_claimed.0 = true;
            if let Ok(machine) = machine_query.get(entity) {
                if machine.output_item.is_some() {
                    machine_collect_events.send(CollectMachineOutputEvent {
                        machine_entity: entity,
                    });
                } else {
                    let slot_idx = inventory.selected_slot;
                    if let Some(ref slot) =
                        inventory.slots.get(slot_idx).and_then(|s| s.as_ref())
                    {
                        machine_insert_events.send(InsertMachineInputEvent {
                            machine_entity: entity,
                            item_id: slot.item_id.clone(),
                            quantity: 1,
                        });
                    }
                }
            }
        }

        InteractionKind::BuildingUpgrade => {
            interaction_claimed.0 = true;
            next_state.set(GameState::BuildingUpgrade);
        }
    }
}
