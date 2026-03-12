use crate::shared::*;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE TYPES
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MachineType {
    Furnace,
    PreservesJar,
    CheesePress,
    Loom,
    Keg,
    OilMaker,
    MayonnaiseMachine,
    Tapper,
    BeeHouse,
    RecyclingMachine,
    CrabPot,
}

impl MachineType {
    /// Processing time in game-hours.
    pub fn processing_hours(&self) -> f32 {
        match self {
            MachineType::Furnace => 0.5,            // 30 game-minutes
            MachineType::PreservesJar => 4.0,       // 240 game-minutes
            MachineType::CheesePress => 3.0,        // 180 game-minutes
            MachineType::Loom => 4.0,               // 240 game-minutes
            MachineType::Keg => 72.0,               // 3 days × 24h
            MachineType::OilMaker => 24.0,          // 1 day
            MachineType::MayonnaiseMachine => 24.0, // 1 day
            MachineType::Tapper => 168.0,           // 7 days × 24h
            MachineType::BeeHouse => 96.0,          // 4 days × 24h
            MachineType::RecyclingMachine => 24.0,  // 1 day
            MachineType::CrabPot => 24.0,           // 1 day
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
            MachineType::MayonnaiseMachine => "Mayonnaise Machine",
            MachineType::Tapper => "Tapper",
            MachineType::BeeHouse => "Bee House",
            MachineType::RecyclingMachine => "Recycling Machine",
            MachineType::CrabPot => "Crab Pot",
        }
    }
}

/// Atlas index in furniture.png for each machine type.
pub fn machine_atlas_index(machine_type: MachineType) -> usize {
    match machine_type {
        MachineType::Furnace => 22,
        MachineType::PreservesJar => 23,
        MachineType::Keg => 24,
        MachineType::CheesePress => 25,
        MachineType::Loom => 26,
        MachineType::OilMaker => 19,
        MachineType::MayonnaiseMachine => 20,
        MachineType::Tapper => 21,
        MachineType::BeeHouse => 27,
        MachineType::RecyclingMachine => 28,
        MachineType::CrabPot => 29,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE ANIMATION
// ──────────────────────────────────────────────────────────────────────────────

/// Number of animation frames in row 0 of machine_anim.png (64x48 tiles, 31 cols).
pub const MACHINE_ANIM_FRAMES: usize = 31;

/// Drives the processing-machine sprite animation.  When the machine is actively
/// processing, the timer cycles through frames 0..MACHINE_ANIM_FRAMES in row 0
/// of machine_anim.png at ~10 fps.  When idle (or ready), it snaps to frame 0.
#[derive(Component, Debug, Clone)]
pub struct MachineAnimTimer {
    pub timer: Timer,
    pub current_frame: usize,
}

impl Default for MachineAnimTimer {
    fn default() -> Self {
        Self {
            // ~10 fps animation
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
            current_frame: 0,
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE PARTICLE / SHAKE COMPONENTS
// ──────────────────────────────────────────────────────────────────────────────

/// Marker for steam/spark particles spawned above active machines.
#[derive(Component)]
pub struct MachineParticle {
    pub lifetime: f32,
    pub elapsed: f32,
    pub velocity_y: f32,
}

/// Cooldown to throttle particle spawning per machine entity.
#[derive(Component)]
pub struct MachineParticleTimer {
    pub timer: Timer,
}

impl Default for MachineParticleTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.35, TimerMode::Repeating),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// PROCEDURAL MACHINE SPRITE CACHE
// ──────────────────────────────────────────────────────────────────────────────

/// Caches procedurally-generated machine sprite handles so they are only built once.
#[derive(Resource, Default)]
pub struct ProceduralMachineSprites {
    pub sprites: std::collections::HashMap<String, Handle<Image>>,
}

/// Generate a 16x16 RGBA procedural image for a machine type.
fn generate_machine_image(machine_type: MachineType) -> Image {
    let w: usize = 16;
    let h: usize = 16;
    let mut data = vec![0u8; w * h * 4];

    let set = |data: &mut Vec<u8>, x: usize, y: usize, r: u8, g: u8, b: u8, a: u8| {
        if x < w && y < h {
            let i = (y * w + x) * 4;
            data[i] = r;
            data[i + 1] = g;
            data[i + 2] = b;
            data[i + 3] = a;
        }
    };

    let fill_rect = |data: &mut Vec<u8>,
                     x0: usize,
                     y0: usize,
                     x1: usize,
                     y1: usize,
                     r: u8,
                     g: u8,
                     b: u8,
                     a: u8| {
        for y in y0..=y1.min(h - 1) {
            for x in x0..=x1.min(w - 1) {
                let i = (y * w + x) * 4;
                data[i] = r;
                data[i + 1] = g;
                data[i + 2] = b;
                data[i + 3] = a;
            }
        }
    };

    match machine_type {
        MachineType::Furnace => {
            // Dark gray box with orange glow on top (fire)
            fill_rect(&mut data, 3, 4, 12, 14, 60, 60, 65, 255); // body
            fill_rect(&mut data, 4, 5, 11, 13, 50, 50, 55, 255); // inner
            fill_rect(&mut data, 5, 6, 10, 10, 30, 28, 25, 255); // opening
                                                                 // Fire glow
            set(&mut data, 6, 2, 255, 160, 30, 255);
            set(&mut data, 7, 1, 255, 100, 20, 200);
            set(&mut data, 8, 2, 255, 140, 25, 255);
            set(&mut data, 9, 3, 255, 180, 40, 230);
            set(&mut data, 7, 3, 255, 200, 60, 255);
            set(&mut data, 8, 3, 255, 220, 80, 255);
            // Fire opening
            set(&mut data, 6, 7, 255, 120, 20, 220);
            set(&mut data, 7, 7, 255, 160, 40, 255);
            set(&mut data, 8, 7, 255, 180, 50, 255);
            set(&mut data, 9, 7, 255, 140, 30, 220);
        }
        MachineType::PreservesJar => {
            // Green/brown jar shape with lid
            fill_rect(&mut data, 5, 3, 10, 4, 120, 90, 50, 255); // lid
            fill_rect(&mut data, 4, 5, 11, 13, 70, 120, 60, 255); // jar body
            fill_rect(&mut data, 5, 6, 10, 12, 80, 140, 70, 255); // lighter center
                                                                  // Highlight stripe
            fill_rect(&mut data, 6, 7, 6, 11, 100, 170, 90, 200);
        }
        MachineType::CheesePress => {
            // Wooden box with handle detail
            fill_rect(&mut data, 2, 6, 13, 14, 140, 100, 50, 255); // wooden base
            fill_rect(&mut data, 3, 7, 12, 13, 160, 115, 60, 255); // lighter wood
            fill_rect(&mut data, 4, 3, 11, 5, 130, 90, 45, 255); // press top
                                                                 // Handle
            fill_rect(&mut data, 7, 1, 8, 3, 80, 80, 85, 255);
            set(&mut data, 6, 1, 90, 90, 95, 255);
            set(&mut data, 9, 1, 90, 90, 95, 255);
        }
        MachineType::Loom => {
            // Wooden frame shape
            fill_rect(&mut data, 2, 12, 13, 14, 140, 100, 50, 255); // base
                                                                    // Vertical posts
            fill_rect(&mut data, 3, 2, 4, 12, 130, 90, 45, 255);
            fill_rect(&mut data, 11, 2, 12, 12, 130, 90, 45, 255);
            // Top bar
            fill_rect(&mut data, 3, 2, 12, 3, 150, 110, 55, 255);
            // Threads
            for x in 5..11 {
                set(&mut data, x, 5, 220, 220, 200, 180);
                set(&mut data, x, 7, 220, 220, 200, 180);
                set(&mut data, x, 9, 220, 220, 200, 180);
            }
        }
        MachineType::Keg => {
            // Barrel shape (brown rectangle with hoops)
            fill_rect(&mut data, 4, 3, 11, 13, 130, 85, 40, 255); // barrel body
            fill_rect(&mut data, 5, 4, 10, 12, 150, 100, 50, 255); // lighter center
                                                                   // Hoops (darker bands)
            fill_rect(&mut data, 3, 5, 12, 5, 100, 100, 110, 255);
            fill_rect(&mut data, 3, 11, 12, 11, 100, 100, 110, 255);
            // Spigot
            set(&mut data, 12, 8, 80, 80, 85, 255);
            set(&mut data, 13, 8, 90, 90, 95, 255);
        }
        MachineType::OilMaker => {
            // Metal cylinder with spout
            fill_rect(&mut data, 4, 4, 11, 14, 140, 145, 150, 255); // cylinder
            fill_rect(&mut data, 5, 5, 10, 13, 160, 165, 170, 255); // lighter
                                                                    // Top cap
            fill_rect(&mut data, 5, 3, 10, 4, 120, 125, 130, 255);
            // Spout
            fill_rect(&mut data, 11, 9, 13, 10, 120, 125, 130, 255);
            set(&mut data, 14, 10, 200, 180, 50, 200); // oil drip
        }
        _ => {
            // Generic machine fallback — gray box with darker border
            fill_rect(&mut data, 3, 3, 12, 13, 100, 95, 90, 255);
            fill_rect(&mut data, 4, 4, 11, 12, 120, 115, 110, 255);
        }
    }

    let mut img = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img
}

/// Returns a cached handle for the procedural machine sprite, generating it on first call.
fn get_procedural_machine_sprite(
    machine_type: MachineType,
    cache: &mut ProceduralMachineSprites,
    images: &mut Assets<Image>,
) -> Handle<Image> {
    let key = format!("{:?}", machine_type);
    cache
        .sprites
        .entry(key)
        .or_insert_with(|| {
            let img = generate_machine_image(machine_type);
            images.add(img)
        })
        .clone()
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
            "iron_ore" => Some(("iron_bar".to_string(), 1)),
            "gold_ore" => Some(("gold_bar".to_string(), 1)),
            "iridium_ore" => Some(("iridium_bar".to_string(), 1)),
            "coal" => Some(("coal".to_string(), 1)), // passthrough (no-op but valid)
            "quartz" => Some(("refined_quartz".to_string(), 1)),
            _ => None,
        },
        MachineType::PreservesJar => match input {
            // Fruits → Jelly
            "blueberry" => Some(("blueberry_jelly".to_string(), 1)),
            "strawberry" => Some(("strawberry_jelly".to_string(), 1)),
            "melon" => Some(("melon_jelly".to_string(), 1)),
            "apple" => Some(("apple_jelly".to_string(), 1)),
            "cranberry" => Some(("cranberry_sauce".to_string(), 1)),
            "ancient_fruit" => Some(("ancient_jelly".to_string(), 1)),
            // Vegetables → Pickles
            "turnip" => Some(("pickled_turnip".to_string(), 1)),
            "potato" => Some(("pickled_potato".to_string(), 1)),
            "cauliflower" => Some(("pickled_cauliflower".to_string(), 1)),
            "pumpkin" => Some(("pickled_pumpkin".to_string(), 1)),
            "eggplant" => Some(("pickled_eggplant".to_string(), 1)),
            "yam" => Some(("pickled_yam".to_string(), 1)),
            "tomato" => Some(("pickled_tomato".to_string(), 1)),
            "corn" => Some(("pickled_corn".to_string(), 1)),
            _ => None,
        },
        MachineType::CheesePress => match input {
            "milk" => Some(("cheese".to_string(), 1)),
            "large_milk" => Some(("large_cheese".to_string(), 1)),
            _ => None,
        },
        MachineType::Loom => match input {
            "wool" => Some(("cloth".to_string(), 1)),
            _ => None,
        },
        MachineType::Keg => match input {
            "wheat" => Some(("beer".to_string(), 1)),
            "hops" => Some(("pale_ale".to_string(), 1)),
            "blueberry" => Some(("blueberry_wine".to_string(), 1)),
            "strawberry" => Some(("strawberry_wine".to_string(), 1)),
            "melon" => Some(("melon_wine".to_string(), 1)),
            "pumpkin" => Some(("pumpkin_juice".to_string(), 1)),
            "corn" => Some(("oil".to_string(), 1)),
            "apple" => Some(("apple_cider".to_string(), 1)),
            "ancient_fruit" => Some(("ancient_fruit_wine".to_string(), 1)),
            "honey" => Some(("mead".to_string(), 1)),
            _ => None,
        },
        MachineType::OilMaker => match input {
            "sunflower" => Some(("oil".to_string(), 1)),
            "corn" => Some(("oil".to_string(), 1)),
            "truffle" => Some(("truffle_oil".to_string(), 1)),
            _ => None,
        },
        MachineType::MayonnaiseMachine => match input {
            "egg" => Some(("mayonnaise".to_string(), 1)),
            "large_egg" => Some(("mayonnaise".to_string(), 2)),
            "duck_egg" => Some(("mayonnaise".to_string(), 2)),
            _ => None,
        },
        MachineType::Tapper => match input {
            // Tapper outputs are time-based, not input-based.
            // For the machine system, just accept sap as a "prime" input.
            "sap" => Some(("maple_syrup".to_string(), 1)),
            "hardwood" => Some(("oak_resin".to_string(), 1)),
            "wood" => Some(("pine_tar".to_string(), 1)),
            _ => None,
        },
        MachineType::BeeHouse => match input {
            // Bee houses produce honey without specific input.
            "honey" => Some(("honey".to_string(), 1)),
            _ => Some(("honey".to_string(), 1)),
        },
        MachineType::RecyclingMachine => match input {
            "trash" => Some(("stone".to_string(), 3)),
            "driftwood" => Some(("wood".to_string(), 3)),
            "old_glasses" => Some(("refined_quartz".to_string(), 1)),
            "newspaper" => Some(("cloth".to_string(), 1)),
            _ => None,
        },
        MachineType::CrabPot => match input {
            // Crab pots use bait and produce random shellfish.
            "bait" => Some(("crab".to_string(), 1)),
            _ => None,
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

/// Serializable snapshot of a placed processing machine for save/load.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMachine {
    pub grid_x: i32,
    pub grid_y: i32,
    pub machine_type: MachineType,
    pub input_item: Option<ItemId>,
    pub output_item: Option<ItemId>,
    pub processing_time_remaining: f32,
    pub is_ready: bool,
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
        "furnace" => Some(MachineType::Furnace),
        "preserves_jar" => Some(MachineType::PreservesJar),
        "cheese_press" => Some(MachineType::CheesePress),
        "loom" => Some(MachineType::Loom),
        "keg" => Some(MachineType::Keg),
        "oil_maker" => Some(MachineType::OilMaker),
        "mayonnaise_machine" => Some(MachineType::MayonnaiseMachine),
        "tapper" => Some(MachineType::Tapper),
        "bee_house" => Some(MachineType::BeeHouse),
        "recycling_machine" => Some(MachineType::RecyclingMachine),
        "crab_pot" => Some(MachineType::CrabPot),
        _ => None,
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

                let output_name = machine.output_item.as_deref().unwrap_or("item").to_string();
                let machine_name = machine.machine_type.display_name();

                info!(
                    "{} finished processing {:?} → {:?}",
                    machine_name, machine.input_item, machine.output_item
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
                message: format!("{} is already busy!", machine.machine_type.display_name()),
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
            event.item_id, machine_name, processing_hours
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

            info!("Collected '{}' from {}", output_id, machine_name);

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
#[allow(clippy::too_many_arguments)]
pub fn handle_place_machine(
    mut commands: Commands,
    mut events: EventReader<PlaceMachineEvent>,
    mut inventory: ResMut<Inventory>,
    mut machine_registry: ResMut<ProcessingMachineRegistry>,
    item_registry: Res<ItemRegistry>,
    furniture: Res<crate::world::objects::FurnitureAtlases>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut proc_sprites: ResMut<ProceduralMachineSprites>,
    mut images: ResMut<Assets<Image>>,
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

        // Spawn machine entity — prefer animated atlas, fall back to static sprite, then color
        let display_label = machine_type.display_name().to_string();
        let machine_sprite = if furniture.machine_anim_layout != Handle::default() {
            // Use animated sprite sheet (row 0, frame 0 = idle)
            let mut s = Sprite::from_atlas_image(
                furniture.machine_anim_image.clone(),
                TextureAtlas {
                    layout: furniture.machine_anim_layout.clone(),
                    index: 0,
                },
            );
            s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
            s
        } else if furniture.loaded {
            let mut s = Sprite::from_image(furniture.processing_machine_image.clone());
            s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
            s
        } else {
            // Procedural sprite fallback — recognizable pixel art per machine type
            let handle =
                get_procedural_machine_sprite(machine_type, &mut proc_sprites, &mut images);
            let mut s = Sprite::from_image(handle);
            s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
            s
        };
        let machine_entity = commands
            .spawn((
                ProcessingMachine::new(machine_type),
                GridPosition::new(event.grid_x, event.grid_y),
                machine_sprite,
                Transform::from_xyz(world_x, world_y, Z_ENTITY_BASE),
                LogicalPosition(Vec2::new(world_x, world_y)),
                YSorted,
                MachineAnimTimer::default(),
                MachineParticleTimer::default(),
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

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE ANIMATION SYSTEM
// ──────────────────────────────────────────────────────────────────────────────

/// Drives the animated sprite for processing machines.
///
/// When a machine is actively processing (`is_processing()` returns true), the
/// system advances through animation frames 0..MACHINE_ANIM_FRAMES at ~10 fps.
/// When idle or ready for collection, the sprite snaps to frame 0 (idle pose).
pub fn animate_processing_machines(
    time: Res<Time>,
    mut query: Query<(&ProcessingMachine, &mut MachineAnimTimer, &mut Sprite)>,
) {
    for (machine, mut anim, mut sprite) in query.iter_mut() {
        if machine.is_processing() {
            // Advance animation timer
            anim.timer.tick(time.delta());
            if anim.timer.just_finished() {
                anim.current_frame = (anim.current_frame + 1) % MACHINE_ANIM_FRAMES;
            }
            // Update atlas index — row 0 frames only
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = anim.current_frame;
            }
        } else {
            // Idle / ready — reset to frame 0
            if anim.current_frame != 0 {
                anim.current_frame = 0;
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = 0;
                }
            }
            anim.timer.reset();
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// ACTIVE MACHINE SHAKE — subtle vibration for processing machines
// ──────────────────────────────────────────────────────────────────────────────

/// Applies a subtle positional oscillation (shake) to machines that are actively
/// processing. The shake is +/-0.5 pixels at 5 Hz on both axes. When idle, the
/// transform snaps back to the canonical LogicalPosition.
pub fn shake_active_machines(
    time: Res<Time>,
    mut query: Query<(&ProcessingMachine, &LogicalPosition, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    for (machine, logical_pos, mut transform) in query.iter_mut() {
        if machine.is_processing() {
            // 5 Hz oscillation, +/-0.5 pixel amplitude
            let shake_x = (t * 5.0 * std::f32::consts::TAU).sin() * 0.5;
            let shake_y = (t * 5.0 * std::f32::consts::TAU + 1.0).cos() * 0.5;
            transform.translation.x = logical_pos.0.x + shake_x;
            transform.translation.y = logical_pos.0.y + shake_y;
        } else {
            // Snap back to exact position when idle
            transform.translation.x = logical_pos.0.x;
            transform.translation.y = logical_pos.0.y;
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// MACHINE PARTICLES — tiny steam puffs above active machines
// ──────────────────────────────────────────────────────────────────────────────

/// Spawns tiny 1x1 white "steam" sprites above actively processing machines.
/// Particles float upward and fade out over ~0.8 seconds.
pub fn spawn_machine_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&ProcessingMachine, &Transform, &mut MachineParticleTimer)>,
) {
    for (machine, transform, mut ptimer) in query.iter_mut() {
        if !machine.is_processing() {
            ptimer.timer.reset();
            continue;
        }

        ptimer.timer.tick(time.delta());
        if !ptimer.timer.just_finished() {
            continue;
        }

        // Spawn a small particle above the machine
        let offset_x = ((time.elapsed_secs() * 17.3).sin() * 4.0).round();
        let spawn_pos = Vec3::new(
            transform.translation.x + offset_x,
            transform.translation.y + TILE_SIZE * 0.5 + 2.0,
            Z_EFFECTS,
        );

        commands.spawn((
            MachineParticle {
                lifetime: 0.8,
                elapsed: 0.0,
                velocity_y: 12.0,
            },
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.6), Vec2::new(1.0, 1.0)),
            Transform::from_translation(spawn_pos),
        ));
    }
}

/// Moves machine particles upward and fades them out, despawning when expired.
pub fn update_machine_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut MachineParticle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut particle, mut transform, mut sprite) in query.iter_mut() {
        particle.elapsed += dt;
        if particle.elapsed >= particle.lifetime {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Float upward
        transform.translation.y += particle.velocity_y * dt;

        // Fade out
        let progress = particle.elapsed / particle.lifetime;
        let alpha = (1.0 - progress).clamp(0.0, 1.0) * 0.6;
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }
}

/// Cleanup: despawn all machine particles (called on state exit).
pub fn despawn_machine_particles(
    mut commands: Commands,
    query: Query<Entity, With<MachineParticle>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
