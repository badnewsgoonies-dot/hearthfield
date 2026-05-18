use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// When the player collects output from a machine, add it to inventory.
pub fn crafting_output_system(
    mut events: EventReader<CollectMachineOutputEvent>,
    mut machines: Query<&mut ProcessingMachine>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in events.read() {
        let Ok(mut machine) = machines.get_mut(event.machine_entity) else {
            warn!("CollectMachineOutputEvent: entity not found");
            continue;
        };

        if !machine.is_ready {
            warn!(
                "Cannot collect from {} — output not ready yet",
                machine.machine_type.display_name()
            );
            toast_events.send(ToastEvent {
                message: format!(
                    "{} is still processing...",
                    machine.machine_type.display_name()
                ),
                duration_secs: 2.0,
            });
            continue;
        }

        let Some(ref output_id) = machine.output_item.clone() else {
            warn!("Machine is_ready but has no output_item — this is a bug");
            machine.is_ready = false;
            machine.input_item = None;
            continue;
        };

        let output_display = item_registry
            .get(output_id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| output_id.clone());

        let machine_name = machine.machine_type.display_name();

        let max_stack = item_registry
            .get(output_id)
            .map(|d| d.stack_size.get())
            .unwrap_or(99);

        let leftover = inventory.try_add(output_id, 1, max_stack);
        if leftover == 0 {
            // Successfully added to inventory
            pickup_events.send(ItemPickupEvent {
                item_id: output_id.clone(),
                quantity: 1,
            });

            info!("Collected '{}' from {}", output_id, machine_name);

            toast_events.send(ToastEvent {
                message: format!("You pull {} out of the {}.", output_display, machine_name),
                duration_secs: 3.0,
            });

            // Reset machine state
            machine.input_item = None;
            machine.output_item = None;
            machine.processing_time_remaining = 0.0;
            machine.is_ready = false;

            sfx_events.send(PlaySfxEvent {
                sfx_id: "item_pickup".to_string(),
            });
        } else {
            warn!("Inventory full — cannot collect output from machine");
            toast_events.send(ToastEvent {
                message: "Inventory full! Can't collect output.".to_string(),
                duration_secs: 3.0,
            });
        }
    }
}


