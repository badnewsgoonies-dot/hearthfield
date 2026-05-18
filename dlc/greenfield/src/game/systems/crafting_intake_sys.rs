use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// When the player inserts an item into a machine, start processing.
pub fn crafting_intake_system(
    mut events: EventReader<InsertMachineInputEvent>,
    mut machines: Query<&mut ProcessingMachine>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in events.read() {
        let Ok(mut machine) = machines.get_mut(event.machine_entity) else {
            warn!("InsertMachineInputEvent: entity not found");
            continue;
        };

        if !machine.is_empty() {
            warn!(
                "Cannot insert into {} — already has input or output ready",
                machine.machine_type.display_name()
            );
            toast_events.send(ToastEvent {
                message: format!(
                    "{} is already hard at work.",
                    machine.machine_type.display_name()
                ),
                duration_secs: 2.0,
            });
            continue;
        }

        // Validate item exists
        if item_registry.get(&event.item_id).is_none() {
            warn!("InsertMachineInputEvent: unknown item '{}'", event.item_id);
            continue;
        }

        // Resolve what the machine will produce
        let Some((output_id, _output_qty)) =
            resolve_machine_output(machine.machine_type, &event.item_id)
        else {
            warn!(
                "{} cannot process item '{}'",
                machine.machine_type.display_name(),
                event.item_id
            );
            toast_events.send(ToastEvent {
                message: format!(
                    "{} needs a different ingredient to start transforming anything.",
                    machine.machine_type.display_name()
                ),
                duration_secs: 2.5,
            });
            continue;
        };

        // Remove input from inventory
        let removed = inventory.try_remove(&event.item_id, event.quantity);
        if removed < event.quantity {
            warn!(
                "Not enough '{}' in inventory (needed {}, removed {})",
                event.item_id, event.quantity, removed
            );
            // Refund what was taken
            if removed > 0 {
                let max_stack = item_registry
                    .get(&event.item_id)
                    .map(|d| d.stack_size.get())
                    .unwrap_or(99);
                inventory.try_add(&event.item_id, removed, max_stack);
            }
            toast_events.send(ToastEvent {
                message: format!("Not enough {} in inventory.", event.item_id),
                duration_secs: 2.5,
            });
            continue;
        }

        // Build friendly input name for the toast
        let input_display = item_registry
            .get(&event.item_id)
            .map(|d| d.name.as_str())
            .unwrap_or(&event.item_id)
            .to_string();

        let machine_name = machine.machine_type.display_name();

        // Start processing
        let processing_hours = machine.machine_type.processing_hours();
        machine.input_item = Some(event.item_id.clone());
        machine.output_item = Some(output_id);
        machine.processing_time_remaining = processing_hours;
        machine.is_ready = false;

        info!(
            "Started processing '{}' in {} ({}h remaining)",
            event.item_id, machine_name, processing_hours
        );

        toast_events.send(ToastEvent {
            message: format!("The {} starts working on {}.", machine_name, input_display),
            duration_secs: 3.0,
        });

        sfx_events.send(PlaySfxEvent {
            sfx_id: "machine_insert".to_string(),
        });
    }
}


