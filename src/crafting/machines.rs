use bevy::prelude::*;
use crate::shared::*;

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE TYPES
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MachineType {
    Furnace,
    PreservesJar,
    CheesePress,
    Loom,
    Keg,
    OilMaker,
}

impl MachineType {
    /// Processing time in game-hours.
    pub fn processing_hours(&self) -> f32 {
        match self {
            MachineType::Furnace => 1.0,
            MachineType::PreservesJar => 72.0,  // 3 days × 24h
            MachineType::CheesePress => 24.0,   // 1 day
            MachineType::Loom => 24.0,
            MachineType::Keg => 72.0,
            MachineType::OilMaker => 24.0,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            MachineType::Furnace => "Furnace",
            MachineType::PreservesJar => "Preserves Jar",
            MachineType::CheesePress => "Cheese Press",
            MachineType::Loom => "Loom",
            MachineType::Keg => "Keg",
            MachineType::OilMaker => "Oil Maker",
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// PROCESSING MACHINE COMPONENT
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Component, Debug, Clone)]
pub struct ProcessingMachine {
    pub machine_type: MachineType,
    pub input_item: Option<ItemId>,
    pub output_item: Option<ItemId>,
    /// Remaining processing time in game hours.
    pub processing_time_remaining: f32,
    pub is_ready: bool,
}

impl ProcessingMachine {
    pub fn new(machine_type: MachineType) -> Self {
        Self {
            machine_type,
            input_item: None,
            output_item: None,
            processing_time_remaining: 0.0,
            is_ready: false,
        }
    }

    pub fn is_processing(&self) -> bool {
        self.input_item.is_some() && !self.is_ready
    }

    pub fn is_empty(&self) -> bool {
        self.input_item.is_none() && !self.is_ready
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE CONVERSION TABLES
// ──────────────────────────────────────────────────────────────────────────────

/// Returns (output_item_id, output_quantity) given machine type and input item id.
pub fn resolve_machine_output(machine: MachineType, input: &str) -> Option<(ItemId, u8)> {
    match machine {
        MachineType::Furnace => match input {
            "copper_ore" => Some(("copper_bar".to_string(), 1)),
            "iron_ore"   => Some(("iron_bar".to_string(), 1)),
            "gold_ore"   => Some(("gold_bar".to_string(), 1)),
            "coal"       => Some(("coal".to_string(), 1)), // passthrough (no-op but valid)
            "quartz"     => Some(("refined_quartz".to_string(), 1)),
            _            => None,
        },
        MachineType::PreservesJar => match input {
            // Fruits → Jelly
            "blueberry"     => Some(("blueberry_jelly".to_string(), 1)),
            "strawberry"    => Some(("strawberry_jelly".to_string(), 1)),
            "melon"         => Some(("melon_jelly".to_string(), 1)),
            "apple"         => Some(("apple_jelly".to_string(), 1)),
            "cranberry"     => Some(("cranberry_sauce".to_string(), 1)),
            "ancient_fruit" => Some(("ancient_jelly".to_string(), 1)),
            // Vegetables → Pickles
            "turnip"        => Some(("pickled_turnip".to_string(), 1)),
            "potato"        => Some(("pickled_potato".to_string(), 1)),
            "cauliflower"   => Some(("pickled_cauliflower".to_string(), 1)),
            "pumpkin"       => Some(("pickled_pumpkin".to_string(), 1)),
            "eggplant"      => Some(("pickled_eggplant".to_string(), 1)),
            "yam"           => Some(("pickled_yam".to_string(), 1)),
            "tomato"        => Some(("pickled_tomato".to_string(), 1)),
            "corn"          => Some(("pickled_corn".to_string(), 1)),
            _               => None,
        },
        MachineType::CheesePress => match input {
            "milk"       => Some(("cheese".to_string(), 1)),
            "large_milk" => Some(("large_cheese".to_string(), 1)),
            _            => None,
        },
        MachineType::Loom => match input {
            "wool"       => Some(("cloth".to_string(), 1)),
            _            => None,
        },
        MachineType::Keg => match input {
            "wheat"         => Some(("beer".to_string(), 1)),
            "hops"          => Some(("pale_ale".to_string(), 1)),
            "blueberry"     => Some(("blueberry_wine".to_string(), 1)),
            "strawberry"    => Some(("strawberry_wine".to_string(), 1)),
            "melon"         => Some(("melon_wine".to_string(), 1)),
            "pumpkin"       => Some(("pumpkin_juice".to_string(), 1)),
            "corn"          => Some(("oil".to_string(), 1)),
            "apple"         => Some(("apple_cider".to_string(), 1)),
            "ancient_fruit" => Some(("ancient_fruit_wine".to_string(), 1)),
            "honey"         => Some(("mead".to_string(), 1)),
            _               => None,
        },
        MachineType::OilMaker => match input {
            "sunflower" => Some(("oil".to_string(), 1)),
            "corn"      => Some(("oil".to_string(), 1)),
            "truffle"   => Some(("truffle_oil".to_string(), 1)),
            _           => None,
        },
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// PROCESSING MACHINE REGISTRY (tracks placed machines on farm)
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Resource, Debug, Clone, Default)]
pub struct ProcessingMachineRegistry {
    /// Mapping of grid position to entity for placed machines.
    pub machines: std::collections::HashMap<(i32, i32), Entity>,
}

// ──────────────────────────────────────────────────────────────────────────────
// EVENTS
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone)]
pub struct InsertMachineInputEvent {
    pub machine_entity: Entity,
    pub item_id: ItemId,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone)]
pub struct CollectMachineOutputEvent {
    pub machine_entity: Entity,
}

// ──────────────────────────────────────────────────────────────────────────────
// SYSTEMS
// ──────────────────────────────────────────────────────────────────────────────

/// Advance processing timers based on game time. Each real second, game time
/// advances at time_scale game-minutes per second. We convert to hours to
/// decrement processing_time_remaining.
pub fn tick_processing_machines(
    time: Res<Time>,
    calendar: Res<Calendar>,
    mut machines: Query<&mut ProcessingMachine>,
) {
    if calendar.time_paused {
        return;
    }

    // Game minutes advanced this frame = time_scale * delta_seconds
    // Convert to hours: divide by 60
    let game_hours_delta = (calendar.time_scale * time.delta_secs()) / 60.0;

    for mut machine in machines.iter_mut() {
        if machine.is_processing() && machine.processing_time_remaining > 0.0 {
            machine.processing_time_remaining -= game_hours_delta;
            if machine.processing_time_remaining <= 0.0 {
                machine.processing_time_remaining = 0.0;
                machine.is_ready = true;
                info!(
                    "{} finished processing {:?} → {:?}",
                    machine.machine_type.display_name(),
                    machine.input_item,
                    machine.output_item
                );
            }
        }
    }
}

/// Handle DayEndEvent — additional processing tick for time-based machines
/// (mainly for machines measured in days, like Preserves Jar / Cheese Press / Loom).
/// The `tick_processing_machines` system already handles this via real time accumulation,
/// but this system re-checks and finalizes any machines that should have completed.
pub fn handle_day_end_processing(
    mut day_end_events: EventReader<DayEndEvent>,
    mut machines: Query<&mut ProcessingMachine>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for _event in day_end_events.read() {
        for mut machine in machines.iter_mut() {
            // Force-complete any machine that has been processing for >= full processing time
            // (safety net in case real-time ticking missed the boundary)
            if machine.is_processing() && machine.processing_time_remaining <= 0.0 {
                machine.is_ready = true;
                sfx_events.write(PlaySfxEvent {
                    sfx_id: "machine_ready".to_string(),
                });
            }
        }
    }
}

/// When the player inserts an item into a machine, start processing.
pub fn handle_insert_machine_input(
    mut events: EventReader<InsertMachineInputEvent>,
    mut machines: Query<&mut ProcessingMachine>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
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
                    .map(|d| d.stack_size)
                    .unwrap_or(99);
                inventory.try_add(&event.item_id, removed, max_stack);
            }
            continue;
        }

        // Start processing
        let processing_hours = machine.machine_type.processing_hours();
        machine.input_item = Some(event.item_id.clone());
        machine.output_item = Some(output_id);
        machine.processing_time_remaining = processing_hours;
        machine.is_ready = false;

        info!(
            "Started processing '{}' in {} ({}h remaining)",
            event.item_id,
            machine.machine_type.display_name(),
            processing_hours
        );

        sfx_events.write(PlaySfxEvent {
            sfx_id: "machine_insert".to_string(),
        });
    }
}

/// When the player collects output from a machine, add it to inventory.
pub fn handle_collect_machine_output(
    mut events: EventReader<CollectMachineOutputEvent>,
    mut machines: Query<&mut ProcessingMachine>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
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
            continue;
        }

        let Some(ref output_id) = machine.output_item.clone() else {
            warn!("Machine is_ready but has no output_item — this is a bug");
            machine.is_ready = false;
            machine.input_item = None;
            continue;
        };

        let max_stack = item_registry
            .get(output_id)
            .map(|d| d.stack_size)
            .unwrap_or(99);

        let leftover = inventory.try_add(output_id, 1, max_stack);
        if leftover == 0 {
            // Successfully added to inventory
            pickup_events.write(ItemPickupEvent {
                item_id: output_id.clone(),
                quantity: 1,
            });

            info!(
                "Collected '{}' from {}",
                output_id,
                machine.machine_type.display_name()
            );

            // Reset machine state
            machine.input_item = None;
            machine.output_item = None;
            machine.processing_time_remaining = 0.0;
            machine.is_ready = false;

            sfx_events.write(PlaySfxEvent {
                sfx_id: "item_pickup".to_string(),
            });
        } else {
            warn!("Inventory full — cannot collect output from machine");
        }
    }
}
