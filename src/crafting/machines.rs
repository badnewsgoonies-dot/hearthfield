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

/// Atlas index in furniture.png for each machine type.
fn machine_atlas_index(machine_type: MachineType) -> usize {
    match machine_type {
        MachineType::Furnace => 22,
        MachineType::PreservesJar => 23,
        MachineType::Keg => 24,
        MachineType::CheesePress => 25,
        MachineType::Loom => 26,
        MachineType::OilMaker => 19,
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
// ITEM → MACHINE TYPE MAPPING
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the MachineType that corresponds to a placeable item id, or None if
/// the item is not a placeable machine.
pub fn item_to_machine_type(item_id: &str) -> Option<MachineType> {
    match item_id {
        "furnace"      => Some(MachineType::Furnace),
        "preserves_jar" => Some(MachineType::PreservesJar),
        "cheese_press" => Some(MachineType::CheesePress),
        "loom"         => Some(MachineType::Loom),
        "keg"          => Some(MachineType::Keg),
        "oil_maker"    => Some(MachineType::OilMaker),
        _              => None,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE PLACEMENT EVENT
// ──────────────────────────────────────────────────────────────────────────────

/// Fired when the player wants to place a machine on the farm at a grid tile.
/// The player plugin sends this when the hotbar contains a machine item and
/// the player activates the "use/place" action.
#[derive(Event, Debug, Clone)]
pub struct PlaceMachineEvent {
    pub item_id: ItemId,
    pub grid_x: i32,
    pub grid_y: i32,
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
    mut toast_events: EventWriter<ToastEvent>,
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

                let output_name = machine
                    .output_item
                    .as_deref()
                    .unwrap_or("item")
                    .to_string();
                let machine_name = machine.machine_type.display_name();

                info!(
                    "{} finished processing {:?} → {:?}",
                    machine_name,
                    machine.input_item,
                    machine.output_item
                );

                toast_events.send(ToastEvent {
                    message: format!("{} is ready in your {}!", output_name, machine_name),
                    duration_secs: 4.0,
                });
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
                sfx_events.send(PlaySfxEvent {
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
                    "{} is already busy!",
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
                    "{} can't process that item.",
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
                    .map(|d| d.stack_size)
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
            event.item_id,
            machine_name,
            processing_hours
        );

        toast_events.send(ToastEvent {
            message: format!("Processing {} in {}...", input_display, machine_name),
            duration_secs: 3.0,
        });

        sfx_events.send(PlaySfxEvent {
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
            .map(|d| d.stack_size)
            .unwrap_or(99);

        let leftover = inventory.try_add(output_id, 1, max_stack);
        if leftover == 0 {
            // Successfully added to inventory
            pickup_events.send(ItemPickupEvent {
                item_id: output_id.clone(),
                quantity: 1,
            });

            info!(
                "Collected '{}' from {}",
                output_id,
                machine_name
            );

            toast_events.send(ToastEvent {
                message: format!("{} collected from your {}!", output_display, machine_name),
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

/// Handles PlaceMachineEvent — spawns a ProcessingMachine entity at the given farm grid tile.
/// Consumes the machine item from the player's inventory and registers the entity in
/// ProcessingMachineRegistry so that other systems can look it up by position.
pub fn handle_place_machine(
    mut commands: Commands,
    mut events: EventReader<PlaceMachineEvent>,
    mut inventory: ResMut<Inventory>,
    mut machine_registry: ResMut<ProcessingMachineRegistry>,
    item_registry: Res<ItemRegistry>,
    furniture: Res<crate::world::objects::FurnitureAtlases>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for event in events.read() {
        // Validate item id maps to a machine type
        let Some(machine_type) = item_to_machine_type(&event.item_id) else {
            warn!(
                "PlaceMachineEvent: item '{}' is not a placeable machine",
                event.item_id
            );
            continue;
        };

        // Check the tile isn't already occupied
        let pos = (event.grid_x, event.grid_y);
        if machine_registry.machines.contains_key(&pos) {
            toast_events.send(ToastEvent {
                message: "There's already a machine here!".to_string(),
                duration_secs: 2.0,
            });
            continue;
        }

        // Consume one machine item from inventory
        let removed = inventory.try_remove(&event.item_id, 1);
        if removed < 1 {
            warn!(
                "PlaceMachineEvent: no '{}' in inventory to place",
                event.item_id
            );
            toast_events.send(ToastEvent {
                message: format!("You don't have a {} to place.", event.item_id),
                duration_secs: 2.5,
            });
            continue;
        }

        // Calculate world position from grid position
        let world_x = event.grid_x as f32 * TILE_SIZE;
        let world_y = event.grid_y as f32 * TILE_SIZE;

        // Spawn machine entity
        let display_label = machine_type.display_name().to_string();
        let machine_sprite = if furniture.loaded {
            let mut s = Sprite::from_atlas_image(
                furniture.image.clone(),
                TextureAtlas {
                    layout: furniture.layout.clone(),
                    index: machine_atlas_index(machine_type),
                },
            );
            s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
            s
        } else {
            Sprite::from_color(
                Color::srgb(0.6, 0.4, 0.2),
                Vec2::new(TILE_SIZE, TILE_SIZE),
            )
        };
        let machine_entity = commands
            .spawn((
                ProcessingMachine::new(machine_type),
                GridPosition::new(event.grid_x, event.grid_y),
                machine_sprite,
                Transform::from_xyz(world_x, world_y, Z_ENTITY_BASE),
                LogicalPosition(Vec2::new(world_x, world_y)),
                YSorted,
                Interactable {
                    kind: InteractionKind::Machine,
                    label: display_label,
                },
            ))
            .id();

        // Register in registry
        machine_registry.machines.insert(pos, machine_entity);

        let display_name = machine_type.display_name();
        info!(
            "Placed {} at grid ({}, {})",
            display_name, event.grid_x, event.grid_y
        );

        toast_events.send(ToastEvent {
            message: format!("{} placed on farm.", display_name),
            duration_secs: 2.5,
        });

        sfx_events.send(PlaySfxEvent {
            sfx_id: "place_machine".to_string(),
        });

        let _ = item_registry; // registry available for future use (stack_size lookups, etc.)
    }
}
