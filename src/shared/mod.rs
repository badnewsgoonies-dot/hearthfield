//! Shared components, resources, events, and states for Hearthfield.
//!
//! This is the type contract. Every domain plugin imports from here.
//! No domain imports from any other domain directly.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════
// GAME STATE — top-level state machine
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, States, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    Dialogue,
    Shop,
    Fishing,
    /// Reserved for future underground mine UI state.
    #[allow(dead_code)]
    Mining,
    Crafting,
    Inventory,
    /// Reserved for future scripted cutscene sequences.
    #[allow(dead_code)]
    Cutscene,
}

// ═══════════════════════════════════════════════════════════════════════
// CALENDAR
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

impl Season {
    pub fn next(self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Fall => 2,
            Season::Winter => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Weather {
    Sunny,
    Rainy,
    Stormy,
    Snowy, // Winter only
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub year: u32,
    pub season: Season,
    pub day: u8,           // 1-28
    pub hour: u8,          // 6-25 (25 = 1:00 AM next day)
    pub minute: u8,        // 0-59
    pub weather: Weather,
    pub time_scale: f32,   // game-minutes per real-second (default ~10)
    pub time_paused: bool,
    pub elapsed_real_seconds: f32, // accumulator for sub-minute ticks
}

impl Default for Calendar {
    fn default() -> Self {
        Self {
            year: 1,
            season: Season::Spring,
            day: 1,
            hour: 6,
            minute: 0,
            weather: Weather::Sunny,
            time_scale: 10.0,
            time_paused: false,
            elapsed_real_seconds: 0.0,
        }
    }
}

impl Calendar {
    pub fn day_of_week(&self) -> DayOfWeek {
        let total_days = (self.season.index() as u32 * 28) + (self.day as u32 - 1);
        match total_days % 7 {
            0 => DayOfWeek::Monday,
            1 => DayOfWeek::Tuesday,
            2 => DayOfWeek::Wednesday,
            3 => DayOfWeek::Thursday,
            4 => DayOfWeek::Friday,
            5 => DayOfWeek::Saturday,
            _ => DayOfWeek::Sunday,
        }
    }

    pub fn total_days_elapsed(&self) -> u32 {
        ((self.year - 1) * 112) + (self.season.index() as u32 * 28) + (self.day as u32 - 1)
    }

    pub fn is_festival_day(&self) -> bool {
        matches!(
            (self.season, self.day),
            (Season::Spring, 13)
                | (Season::Summer, 11)
                | (Season::Fall, 16)
                | (Season::Winter, 25)
        )
    }

    /// Returns time as a float (e.g. 14.5 = 2:30 PM) for schedule lookups.
    pub fn time_float(&self) -> f32 {
        self.hour as f32 + (self.minute as f32 / 60.0)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PLAYER
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Facing {
    Up,
    Down,
    Left,
    Right,
}

impl Default for Facing {
    fn default() -> Self {
        Facing::Down
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolKind {
    Hoe,
    WateringCan,
    Axe,
    Pickaxe,
    FishingRod,
    Scythe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolTier {
    Basic,
    Copper,
    Iron,
    Gold,
    Iridium,
}

impl ToolTier {
    pub fn upgrade_cost(&self) -> u32 {
        match self {
            ToolTier::Basic => 0,
            ToolTier::Copper => 2_000,
            ToolTier::Iron => 5_000,
            ToolTier::Gold => 10_000,
            ToolTier::Iridium => 25_000,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            ToolTier::Basic => Some(ToolTier::Copper),
            ToolTier::Copper => Some(ToolTier::Iron),
            ToolTier::Iron => Some(ToolTier::Gold),
            ToolTier::Gold => Some(ToolTier::Iridium),
            ToolTier::Iridium => None,
        }
    }

    /// Gold cost to upgrade FROM this tier to the next.
    #[allow(dead_code)]
    pub fn upgrade_cost_gold(&self) -> u32 {
        match self {
            ToolTier::Basic => 2000,
            ToolTier::Copper => 5000,
            ToolTier::Iron => 10000,
            ToolTier::Gold => 25000,
            ToolTier::Iridium => 0,
        }
    }

    /// Number of bars required to upgrade FROM this tier.
    #[allow(dead_code)]
    pub fn upgrade_bars_needed(&self) -> u8 {
        match self {
            ToolTier::Basic | ToolTier::Copper | ToolTier::Iron | ToolTier::Gold => 5,
            ToolTier::Iridium => 0,
        }
    }

    /// The bar item needed to upgrade FROM this tier.
    #[allow(dead_code)]
    pub fn upgrade_bar_item(&self) -> Option<&'static str> {
        match self {
            ToolTier::Basic => Some("copper_bar"),
            ToolTier::Copper => Some("iron_bar"),
            ToolTier::Iron => Some("gold_bar"),
            ToolTier::Gold => Some("iridium_bar"),
            ToolTier::Iridium => None,
        }
    }

    /// Stamina cost multiplier. Better tools use less stamina.
    pub fn stamina_multiplier(&self) -> f32 {
        match self {
            ToolTier::Basic => 1.0,
            ToolTier::Copper => 0.85,
            ToolTier::Iron => 0.7,
            ToolTier::Gold => 0.55,
            ToolTier::Iridium => 0.4,
        }
    }

    /// Days the blacksmith takes for any upgrade.
    #[allow(dead_code)]
    pub fn upgrade_days(&self) -> u8 { 2 }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Player;

#[derive(Component, Debug, Clone)]
pub struct PlayerMovement {
    pub facing: Facing,
    pub is_moving: bool,
    pub speed: f32,
    pub move_cooldown: Timer,
    pub anim_state: PlayerAnimState,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            facing: Facing::Down,
            is_moving: false,
            speed: 80.0,
            move_cooldown: Timer::from_seconds(0.0, TimerMode::Once),
            anim_state: PlayerAnimState::Idle,
        }
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub stamina: f32,
    pub max_stamina: f32,
    pub health: f32,
    pub max_health: f32,
    pub equipped_tool: ToolKind,
    pub tools: HashMap<ToolKind, ToolTier>,
    pub gold: u32,
    pub current_map: MapId,
}

impl Default for PlayerState {
    fn default() -> Self {
        let mut tools = HashMap::new();
        tools.insert(ToolKind::Hoe, ToolTier::Basic);
        tools.insert(ToolKind::WateringCan, ToolTier::Basic);
        tools.insert(ToolKind::Axe, ToolTier::Basic);
        tools.insert(ToolKind::Pickaxe, ToolTier::Basic);
        tools.insert(ToolKind::FishingRod, ToolTier::Basic);
        tools.insert(ToolKind::Scythe, ToolTier::Basic);

        Self {
            stamina: 100.0,
            max_stamina: 100.0,
            health: 100.0,
            max_health: 100.0,
            equipped_tool: ToolKind::Hoe,
            tools,
            gold: 500,
            current_map: MapId::Farm,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INVENTORY
// ═══════════════════════════════════════════════════════════════════════

/// Unique identifier for every item type in the game.
/// Using string IDs for data-driven flexibility.
pub type ItemId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemCategory {
    Seed,
    Crop,
    AnimalProduct,
    ArtisanGood,
    Fish,
    Mineral,
    Gem,
    CraftingMaterial,
    Food,
    Tool,
    Furniture,
    Gift,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDef {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    pub sell_price: u32,
    pub buy_price: Option<u32>, // None = not buyable
    pub stack_size: u8,         // max per slot (1 for tools, 99 for most items)
    pub edible: bool,
    pub energy_restore: f32,    // if edible
    pub sprite_index: u32,      // atlas index
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item_id: ItemId,
    pub quantity: u8,
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    /// 36 slots: 0-11 = hotbar, 12-35 = backpack
    pub slots: Vec<Option<InventorySlot>>,
    pub selected_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: vec![None; 36],
            selected_slot: 0,
        }
    }
}

impl Inventory {
    /// Try to add an item. Returns the quantity that couldn't fit.
    pub fn try_add(&mut self, item_id: &str, quantity: u8, max_stack: u8) -> u8 {
        let mut remaining = quantity;

        // First pass: stack onto existing slots with same item
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(ref mut s) = slot {
                if s.item_id == item_id && s.quantity < max_stack {
                    let space = max_stack - s.quantity;
                    let add = remaining.min(space);
                    s.quantity += add;
                    remaining -= add;
                }
            }
        }

        // Second pass: fill empty slots
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                let add = remaining.min(max_stack);
                *slot = Some(InventorySlot {
                    item_id: item_id.to_string(),
                    quantity: add,
                });
                remaining -= add;
            }
        }

        remaining
    }

    /// Check if items can fit without modifying the inventory. Returns leftover count.
    pub fn can_fit(&self, item_id: &str, quantity: u8, max_stack: u8) -> u8 {
        let mut remaining = quantity;

        // First pass: count space in existing stacks of the same item
        for slot in self.slots.iter() {
            if remaining == 0 {
                break;
            }
            if let Some(ref s) = slot {
                if s.item_id == item_id && s.quantity < max_stack {
                    let space = max_stack - s.quantity;
                    remaining = remaining.saturating_sub(space);
                }
            }
        }

        // Second pass: count empty slots
        for slot in self.slots.iter() {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                remaining = remaining.saturating_sub(max_stack);
            }
        }

        remaining
    }

    /// Remove quantity of an item. Returns how many were actually removed.
    pub fn try_remove(&mut self, item_id: &str, quantity: u8) -> u8 {
        let mut remaining = quantity;
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(ref mut s) = slot {
                if s.item_id == item_id {
                    let remove = remaining.min(s.quantity);
                    s.quantity -= remove;
                    remaining -= remove;
                    if s.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }
        quantity - remaining
    }

    pub fn count(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.quantity as u32)
            .sum()
    }

    pub fn has(&self, item_id: &str, quantity: u8) -> bool {
        self.count(item_id) >= quantity as u32
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ITEM REGISTRY — loaded from data
// ═══════════════════════════════════════════════════════════════════════

#[derive(Resource, Debug, Clone, Default)]
pub struct ItemRegistry {
    pub items: HashMap<ItemId, ItemDef>,
}

impl ItemRegistry {
    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FARMING
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SoilState {
    Untilled,
    Tilled,
    Watered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CropDef {
    pub id: ItemId,
    pub name: String,
    pub seed_id: ItemId,
    pub harvest_id: ItemId,
    pub seasons: Vec<Season>,
    pub growth_days: Vec<u8>, // days per stage (len = num stages)
    pub regrows: bool,
    pub regrow_days: u8, // days to regrow after harvest (if regrows)
    pub sell_price: u32,
    pub sprite_stages: Vec<u32>, // atlas indices per growth stage
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CropTile {
    pub crop_id: ItemId,
    pub current_stage: u8,
    pub days_in_stage: u8,
    pub watered_today: bool,
    pub days_without_water: u8,
    pub dead: bool,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SoilTile {
    pub state: SoilState,
    pub grid_x: i32,
    pub grid_y: i32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct CropRegistry {
    pub crops: HashMap<ItemId, CropDef>,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct FarmState {
    /// Tiles that have been tilled/watered. Key = (x, y).
    pub soil: HashMap<(i32, i32), SoilState>,
    /// Active crops. Key = (x, y).
    pub crops: HashMap<(i32, i32), CropTile>,
    /// Objects on the farm (trees, rocks, stumps). Key = (x, y).
    pub objects: HashMap<(i32, i32), FarmObject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FarmObject {
    Tree { health: u8 },
    Rock { health: u8 },
    Stump { health: u8 },
    Bush,
    Sprinkler,
    Scarecrow,
    Fence,
    Path,
    ShippingBin,
}

// ═══════════════════════════════════════════════════════════════════════
// ANIMALS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimalKind {
    Chicken,
    Cow,
    Sheep,
    Cat,
    Dog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnimalAge {
    Baby,
    Adult,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Animal {
    pub kind: AnimalKind,
    pub name: String,
    pub age: AnimalAge,
    pub days_old: u16,
    pub happiness: u8,    // 0-255
    pub fed_today: bool,
    pub petted_today: bool,
    pub product_ready: bool,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimalState {
    pub animals: Vec<Animal>,
    pub has_coop: bool,
    pub has_barn: bool,
    pub coop_level: u8, // 0=none, 1=basic, 2=big, 3=deluxe
    pub barn_level: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// WORLD & MAPS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MapId {
    Farm,
    Town,
    Beach,
    Forest,
    MineEntrance,
    Mine, // + floor number in MineState
    PlayerHouse,
    GeneralStore,
    AnimalShop,
    Blacksmith,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TileKind {
    Grass,
    Dirt,
    TilledSoil,
    WateredSoil,
    Water,
    Sand,
    Stone,
    WoodFloor,
    Path,
    Bridge,
    Void,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MapTransition {
    pub from_map: MapId,
    pub from_rect: (i32, i32, i32, i32), // x, y, w, h trigger area
    pub to_map: MapId,
    pub to_pos: (i32, i32),
}

// ═══════════════════════════════════════════════════════════════════════
// NPCs & RELATIONSHIPS
// ═══════════════════════════════════════════════════════════════════════

pub type NpcId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GiftPreference {
    Loved,   // +80 points
    Liked,   // +45 points
    Neutral, // +20 points
    Disliked, // -20 points
    Hated,   // -40 points
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcDef {
    pub id: NpcId,
    pub name: String,
    pub birthday_season: Season,
    pub birthday_day: u8,
    pub gift_preferences: HashMap<ItemId, GiftPreference>,
    pub default_dialogue: Vec<String>,
    pub heart_dialogue: HashMap<u8, Vec<String>>, // hearts reached → new lines
    pub is_marriageable: bool,
    pub sprite_index: u32,
    pub portrait_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub time: f32,      // e.g. 9.0 = 9:00 AM
    pub map: MapId,
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcSchedule {
    /// Day-of-week → list of (time, location) pairs
    pub weekday: Vec<ScheduleEntry>,
    pub weekend: Vec<ScheduleEntry>,
    /// Season overrides
    pub rain_override: Option<Vec<ScheduleEntry>>,
    pub festival_override: Option<Vec<ScheduleEntry>>,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Relationships {
    /// NPC id → friendship points (0-1000, 100 per heart)
    pub friendship: HashMap<NpcId, u32>,
    pub gifted_today: HashMap<NpcId, bool>,
    pub spouse: Option<NpcId>,
}

impl Relationships {
    pub fn hearts(&self, npc_id: &str) -> u8 {
        let points = self.friendship.get(npc_id).copied().unwrap_or(0);
        (points / 100).min(10) as u8
    }

    pub fn add_friendship(&mut self, npc_id: &str, amount: i32) {
        let entry = self.friendship.entry(npc_id.to_string()).or_insert(0);
        *entry = (*entry as i32 + amount).clamp(0, 1000) as u32;
    }
}

#[derive(Component, Debug, Clone)]
pub struct Npc {
    pub id: NpcId,
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct NpcRegistry {
    pub npcs: HashMap<NpcId, NpcDef>,
    pub schedules: HashMap<NpcId, NpcSchedule>,
}

// ═══════════════════════════════════════════════════════════════════════
// ECONOMY & SHOPS
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShopId {
    GeneralStore,
    AnimalShop,
    Blacksmith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopListing {
    pub item_id: ItemId,
    pub price: u32,
    pub season_available: Option<Season>, // None = always
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ShopData {
    pub listings: HashMap<ShopId, Vec<ShopListing>>,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShippingBin {
    pub items: Vec<InventorySlot>,
}

// ═══════════════════════════════════════════════════════════════════════
// CRAFTING & COOKING
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<(ItemId, u8)>, // (item_id, quantity)
    pub result: ItemId,
    pub result_quantity: u8,
    pub is_cooking: bool, // true = cooking, false = crafting
    pub unlocked_by_default: bool,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct RecipeRegistry {
    pub recipes: HashMap<String, Recipe>,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnlockedRecipes {
    pub ids: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════
// FISHING
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FishLocation {
    River,
    Ocean,
    Pond,
    MinePool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishDef {
    pub id: ItemId,
    pub name: String,
    pub location: FishLocation,
    pub seasons: Vec<Season>,
    pub time_range: (f32, f32),    // e.g. (6.0, 20.0) = 6AM-8PM
    pub weather_required: Option<Weather>,
    pub rarity: Rarity,
    pub difficulty: f32,           // 0.0 = trivial, 1.0 = legendary
    pub sell_price: u32,
    pub sprite_index: u32,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct FishRegistry {
    pub fish: HashMap<ItemId, FishDef>,
}

// ═══════════════════════════════════════════════════════════════════════
// MINING
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MineEnemy {
    GreenSlime,
    Bat,
    RockCrab,
}

#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct MineState {
    pub current_floor: u8,             // 0 = not in mine
    pub deepest_floor_reached: u8,     // for elevator
    pub elevator_floors: Vec<u8>,      // unlocked elevator stops (every 5)
}

#[derive(Component, Debug, Clone)]
pub struct MineRock {
    pub health: u8,
    pub drop_item: ItemId,
    pub drop_quantity: u8,
}

#[derive(Component, Debug, Clone)]
pub struct MineMonster {
    pub kind: MineEnemy,
    pub health: f32,
    #[allow(dead_code)]
    pub max_health: f32,
    pub damage: f32,
    #[allow(dead_code)]
    pub speed: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// EVENTS — cross-domain communication
// ═══════════════════════════════════════════════════════════════════════

#[derive(Event, Debug, Clone)]
pub struct DayEndEvent {
    pub day: u8,
    pub season: Season,
    pub year: u32,
}

#[derive(Event, Debug, Clone)]
pub struct SeasonChangeEvent {
    pub new_season: Season,
    pub year: u32,
}

#[derive(Event, Debug, Clone)]
pub struct ItemPickupEvent {
    pub item_id: ItemId,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone)]
pub struct ItemRemovedEvent {
    pub item_id: ItemId,
    #[allow(dead_code)]
    pub quantity: u8,
}

#[derive(Event, Debug, Clone)]
pub struct DialogueStartEvent {
    pub npc_id: NpcId,
    pub lines: Vec<String>,
    pub portrait_index: Option<u32>,
}

#[derive(Event, Debug, Clone)]
pub struct DialogueEndEvent;

#[derive(Event, Debug, Clone)]
pub struct ShopTransactionEvent {
    #[allow(dead_code)]
    pub shop_id: ShopId,
    pub item_id: ItemId,
    pub quantity: u8,
    #[allow(dead_code)]
    pub total_cost: u32,
    pub is_purchase: bool, // true = buy, false = sell
}

#[derive(Event, Debug, Clone)]
pub struct ToolUseEvent {
    pub tool: ToolKind,
    pub tier: ToolTier,
    pub target_x: i32,
    pub target_y: i32,
}

#[derive(Event, Debug, Clone)]
pub struct MapTransitionEvent {
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

#[derive(Event, Debug, Clone)]
pub struct StaminaDrainEvent {
    pub amount: f32,
}

#[derive(Event, Debug, Clone)]
pub struct GoldChangeEvent {
    pub amount: i32, // positive = gain, negative = spend
    pub reason: String,
}

#[derive(Event, Debug, Clone)]
pub struct GiftGivenEvent {
    pub npc_id: NpcId,
    pub item_id: ItemId,
    #[allow(dead_code)]
    pub preference: GiftPreference,
}

#[derive(Event, Debug, Clone)]
pub struct CropHarvestedEvent {
    pub crop_id: ItemId,
    pub harvest_id: ItemId,
    pub quantity: u8,
    pub x: i32,
    pub y: i32,
    /// Phase 3: quality of the harvested crop (None = Normal for backward compat).
    pub quality: Option<ItemQuality>,
}

#[derive(Event, Debug, Clone)]
pub struct AnimalProductEvent {
    pub animal_kind: AnimalKind,
    pub product_id: ItemId,
}

#[derive(Event, Debug, Clone)]
pub struct PlaySfxEvent {
    pub sfx_id: String,
}

#[derive(Event, Debug, Clone)]
pub struct PlayMusicEvent {
    pub track_id: String,
    #[allow(dead_code)]
    pub fade_in: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// SAVE DATA
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SaveData {
    pub version: u32,
    pub calendar: Calendar,
    pub player_state: PlayerState,
    pub inventory: Inventory,
    pub farm_state: FarmState,
    pub animal_state: AnimalState,
    pub relationships: Relationships,
    pub mine_state: MineState,
    pub unlocked_recipes: UnlockedRecipes,
    pub shipping_bin: ShippingBin,
    pub total_gold_earned: u64,
    pub total_items_shipped: u64,
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

pub const TILE_SIZE: f32 = 16.0;
pub const PIXEL_SCALE: f32 = 3.0; // render scale (16px × 3 = 48px on screen)
pub const SCREEN_WIDTH: f32 = 960.0;
pub const SCREEN_HEIGHT: f32 = 540.0;

pub const DAYS_PER_SEASON: u8 = 28;
#[allow(dead_code)]
pub const SEASONS_PER_YEAR: u8 = 4;

#[allow(dead_code)]
pub const MAX_STAMINA: f32 = 100.0;
#[allow(dead_code)]
pub const MAX_HEALTH: f32 = 100.0;

pub const HOTBAR_SLOTS: usize = 12;
#[allow(dead_code)]
pub const BACKPACK_SLOTS: usize = 24;
#[allow(dead_code)]
pub const TOTAL_INVENTORY_SLOTS: usize = HOTBAR_SLOTS + BACKPACK_SLOTS;

#[allow(dead_code)]
pub const FRIENDSHIP_PER_HEART: u32 = 100;
#[allow(dead_code)]
pub const MAX_HEARTS: u32 = 10;
#[allow(dead_code)]
pub const MAX_FRIENDSHIP: u32 = MAX_HEARTS * FRIENDSHIP_PER_HEART;

// ═══════════════════════════════════════════════════════════════════════
// Z-LAYER CONSTANTS (2D depth ordering)
// ═══════════════════════════════════════════════════════════════════════

/// Ground tiles (grass, dirt, water, paths). Never overlaps with entities.
pub const Z_GROUND: f32 = 0.0;

/// Farm overlays (tilled soil, watered state). Just above ground.
pub const Z_FARM_OVERLAY: f32 = 10.0;

/// Y-sorted entity layer. Player, NPCs, animals, trees, rocks, machines,
/// chests — everything that can visually overlap shares this base.
/// Actual Z = Z_ENTITY_BASE - world_y * Z_Y_SORT_SCALE
pub const Z_ENTITY_BASE: f32 = 100.0;

/// Scale factor for Y-sort offset. With a max map of ~100 tiles = 1600px,
/// max offset = 1600 * 0.01 = 16.0. Entity range: 84.0 to 100.0.
pub const Z_Y_SORT_SCALE: f32 = 0.01;

/// Effects above entities (fishing bobber, festival items).
pub const Z_EFFECTS: f32 = 200.0;

/// Seasonal overlays (falling leaves, flower petals).
pub const Z_SEASONAL: f32 = 300.0;

/// Weather particles (rain, snow, fog). Always on top of world.
pub const Z_WEATHER: f32 = 400.0;

// ═══════════════════════════════════════════════════════════════════════
// RENDERING COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker: this entity participates in Y-sort depth ordering.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct YSorted;

/// Lossless position for physics/gameplay. Transform is derived from this.
/// Movement systems write LogicalPosition. A PostUpdate system rounds it
/// to integer pixels and writes Transform.translation.xy.
#[derive(Component, Debug, Clone, Default)]
pub struct LogicalPosition(pub Vec2);

/// Player animation state. Determines which atlas region and frame-advance
/// logic to use. Only one state active at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAnimState {
    #[default]
    Idle,
    Walk,
    ToolUse {
        tool: ToolKind,
        /// Which frame of the tool animation we're on (0-based).
        frame: u8,
        /// Total frames in this tool animation.
        total_frames: u8,
    },
}

/// Fired when a tool animation reaches its "impact" frame.
#[derive(Event, Debug, Clone)]
pub struct ToolImpactEvent {
    pub tool: ToolKind,
    pub grid_x: i32,
    pub grid_y: i32,
    pub player: Entity,
}

// ═══════════════════════════════════════════════════════════════════════
// PHASE 3 ADDITIONS
// ═══════════════════════════════════════════════════════════════════════

/// Item quality affects sell price multiplier and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ItemQuality {
    #[default]
    Normal,
    Silver,
    Gold,
    Iridium,
}

impl ItemQuality {
    pub fn sell_multiplier(&self) -> f32 {
        match self {
            ItemQuality::Normal => 1.0,
            ItemQuality::Silver => 1.25,
            ItemQuality::Gold => 1.5,
            ItemQuality::Iridium => 2.0,
        }
    }

    #[allow(dead_code)]
    pub fn next(&self) -> Option<ItemQuality> {
        match self {
            ItemQuality::Normal => Some(ItemQuality::Silver),
            ItemQuality::Silver => Some(ItemQuality::Gold),
            ItemQuality::Gold => Some(ItemQuality::Iridium),
            ItemQuality::Iridium => None,
        }
    }
}

/// Quality-aware inventory slot for storage chests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityStack {
    pub item_id: String,
    pub quantity: u8,
    pub quality: ItemQuality,
}

/// Eating food restores stamina/health.
#[derive(Event, Debug, Clone)]
pub struct ConsumeItemEvent {
    pub item_id: String,
    #[allow(dead_code)]
    pub quality: ItemQuality,
}

/// Stamina recovery from food, sleep, or spa.
#[derive(Event, Debug, Clone)]
pub struct StaminaRestoreEvent {
    pub amount: f32,
    pub source: StaminaSource,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StaminaSource {
    Food(String),
    Sleep,
    Spa,
}

/// Animal purchase request from shop.
#[derive(Event, Debug, Clone)]
#[allow(dead_code)]
pub struct AnimalPurchaseEvent {
    pub animal_type: AnimalKind,
    pub cost: u32,
    pub name: String,
}

/// Toast notification for player feedback.
#[derive(Event, Debug, Clone)]
pub struct ToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

/// Chest/storage container on farm.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct StorageChest {
    pub slots: Vec<Option<QualityStack>>,
    pub capacity: usize,
    pub grid_pos: (i32, i32),
}

impl StorageChest {
    pub fn new(capacity: usize, x: i32, y: i32) -> Self {
        Self {
            slots: vec![None; capacity],
            capacity,
            grid_pos: (x, y),
        }
    }
}

/// Day/night ambient light level (0.0 = midnight dark, 1.0 = noon bright).
#[derive(Resource, Debug, Clone)]
pub struct DayNightTint {
    pub intensity: f32,
    pub tint: (f32, f32, f32),
}

impl Default for DayNightTint {
    fn default() -> Self {
        Self {
            intensity: 1.0,
            tint: (1.0, 1.0, 1.0),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// PHASE 4 ADDITIONS
// ═══════════════════════════════════════════════════════════════════════

/// House upgrade tier. Determines available features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum HouseTier {
    #[default]
    Basic,
    Big,
    Deluxe,
}

/// Tracks house upgrade state.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct HouseState {
    pub tier: HouseTier,
    pub has_kitchen: bool,     // Big+ house
    pub has_nursery: bool,     // Deluxe house
}

/// Romance/relationship stage with marriage candidates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum RelationshipStage {
    #[default]
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    Dating,
    Engaged,
    Married,
}

/// Marriage state tracking.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarriageState {
    pub spouse: Option<String>,
    pub wedding_date: Option<(u8, u8, u16)>, // (day, season_idx, year)
    pub days_married: u32,
    pub spouse_happiness: i16, // -100 to 100
}

/// Give bouquet to begin dating (requires 8+ hearts).
#[derive(Event, Debug, Clone)]
pub struct BouquetGivenEvent {
    pub npc_name: String,
}

/// Give mermaid pendant to propose (requires 10 hearts + dating + big house).
#[derive(Event, Debug, Clone)]
pub struct ProposalEvent {
    pub npc_name: String,
}

/// Wedding happens 3 days after accepted proposal.
#[derive(Event, Debug, Clone)]
pub struct WeddingEvent {
    pub npc_name: String,
}

/// Spouse daily action (fires at 8:00 AM game time).
#[derive(Event, Debug, Clone)]
pub struct SpouseActionEvent {
    pub action: SpouseAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpouseAction {
    WaterCrops(u8),
    FeedAnimals,
    RepairFence,
    GiveBreakfast(ItemId),
    StandOnPorch,
}

/// Quest/bulletin board system.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuestLog {
    pub active: Vec<Quest>,
    pub completed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quest {
    pub id: String,
    pub title: String,
    pub description: String,
    pub giver: String,
    pub objective: QuestObjective,
    pub reward_gold: u32,
    pub reward_items: Vec<(ItemId, u8)>,
    pub reward_friendship: i16,
    pub days_remaining: Option<u8>,
    pub accepted_day: (u8, u8, u16), // (day, season_idx, year)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjective {
    Deliver { item_id: ItemId, quantity: u8, delivered: u8 },
    Catch { fish_id: String, delivered: bool },
    Harvest { crop_id: String, quantity: u8, harvested: u8 },
    Mine { item_id: ItemId, quantity: u8, collected: u8 },
    Talk { npc_name: String, talked: bool },
    Slay { monster_kind: String, quantity: u8, slain: u8 },
}

/// New quest posted on bulletin board.
#[derive(Event, Debug, Clone)]
pub struct QuestPostedEvent {
    #[allow(dead_code)]
    pub quest: Quest,
}

/// Quest accepted by player.
#[derive(Event, Debug, Clone)]
pub struct QuestAcceptedEvent {
    pub quest_id: String,
}

/// Quest completed.
#[derive(Event, Debug, Clone)]
pub struct QuestCompletedEvent {
    pub quest_id: String,
    #[allow(dead_code)]
    pub reward_gold: u32,
}

/// Sprinkler types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SprinklerKind {
    Basic,
    Quality,
    Iridium,
}

impl SprinklerKind {
    pub fn range(&self) -> u8 {
        match self {
            SprinklerKind::Basic => 1,
            SprinklerKind::Quality => 1,
            SprinklerKind::Iridium => 2,
        }
    }
    pub fn includes_diagonals(&self) -> bool {
        !matches!(self, SprinklerKind::Basic)
    }
}

/// Placed sprinkler on the farm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacedSprinkler {
    pub kind: SprinklerKind,
    pub tile_x: i32,
    pub tile_y: i32,
}

/// All placed sprinklers.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct SprinklerState {
    pub sprinklers: Vec<PlacedSprinkler>,
}

/// Sprinkler placement event.
#[derive(Event, Debug, Clone)]
pub struct PlaceSprinklerEvent {
    pub kind: SprinklerKind,
    pub tile_x: i32,
    pub tile_y: i32,
}

/// Cooking buff applied to player after eating.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoodBuff {
    pub buff_type: BuffType,
    pub magnitude: f32,
    pub minutes_remaining: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuffType {
    Speed,
    Mining,
    Fishing,
    Farming,
    Defense,
    Attack,
    Luck,
    MaxStamina,
}

/// Active buffs on the player.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActiveBuffs {
    pub buffs: Vec<FoodBuff>,
}

/// Eat food event (consumes item, applies buff + stamina restore).
#[derive(Event, Debug, Clone)]
pub struct EatFoodEvent {
    pub item_id: ItemId,
    pub stamina_restore: f32,
    pub buff: Option<FoodBuff>,
}

/// Year-end evaluation score (grandpa's shrine).
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationScore {
    pub total_points: u32,
    pub categories: HashMap<String, u32>,
    pub evaluated: bool,
    pub candles_lit: u8,
}

/// Trigger year-end evaluation.
#[derive(Event, Debug, Clone)]
pub struct EvaluationTriggerEvent;

/// Relationship stage tracking per NPC (stored in Relationships resource).
/// Maps NPC id → RelationshipStage.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipStages {
    pub stages: HashMap<NpcId, RelationshipStage>,
}

// ═══════════════════════════════════════════════════════════════════════
// PHASE 3 ADDITIONS
// ═══════════════════════════════════════════════════════════════════════

/// Upgrade tiers for farm buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BuildingTier {
    #[default]
    None,
    Basic,
    Big,
    Deluxe,
}

#[allow(dead_code)]
impl BuildingTier {
    pub fn next(&self) -> Option<Self> {
        match self {
            BuildingTier::None => Some(BuildingTier::Basic),
            BuildingTier::Basic => Some(BuildingTier::Big),
            BuildingTier::Big => Some(BuildingTier::Deluxe),
            BuildingTier::Deluxe => None,
        }
    }
    pub fn capacity(&self) -> usize {
        match self {
            BuildingTier::None => 0,
            BuildingTier::Basic => 4,
            BuildingTier::Big => 8,
            BuildingTier::Deluxe => 12,
        }
    }
}

/// Achievement tracking.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct Achievements {
    pub unlocked: Vec<String>,
    pub progress: HashMap<String, u32>,
}

/// Tracks what the player has shipped at least once (for collection tracking).
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShippingLog {
    pub shipped_items: HashMap<ItemId, u32>,
}

/// Tutorial/hint system state.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct TutorialState {
    pub hints_shown: Vec<String>,
    pub tutorial_complete: bool,
    pub current_objective: Option<String>,
}

/// Contextual hint event — shows a non-intrusive tip when the player does something new.
#[derive(Event, Debug, Clone)]
pub struct HintEvent {
    #[allow(dead_code)]
    pub hint_id: String,
    pub message: String,
}

/// Achievement unlocked event.
#[derive(Event, Debug, Clone)]
#[allow(dead_code)]
pub struct AchievementUnlockedEvent {
    pub achievement_id: String,
    pub name: String,
    pub description: String,
}

/// Building upgrade request.
#[derive(Event, Debug, Clone)]
pub struct BuildingUpgradeEvent {
    pub building: BuildingKind,
    #[allow(dead_code)]
    pub from_tier: BuildingTier,
    pub to_tier: BuildingTier,
    #[allow(dead_code)]
    pub cost_gold: u32,
    #[allow(dead_code)]
    pub cost_materials: Vec<(ItemId, u8)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingKind {
    House,
    Coop,
    Barn,
    Silo,
}

/// Tracks total play statistics for achievements and end-of-year summary.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayStats {
    pub crops_harvested: u64,
    pub fish_caught: u64,
    pub items_shipped: u64,
    pub gifts_given: u64,
    pub mine_floors_cleared: u64,
    pub animals_petted: u64,
    pub recipes_cooked: u64,
    pub total_gold_earned: u64,
    pub total_steps_taken: u64,
    pub days_played: u64,
    pub festivals_attended: u64,
}

/// Unified input blocking resource.
///
/// Systems that need exclusive input (dialogue, shops, fishing minigame, menus)
/// push a block tag; player movement and tool use check `is_blocked()` before
/// processing.  Uses TypeId so each blocker is a distinct type — no double-free
/// risk and the block lifetime is tied to the system that owns it.
#[derive(Resource, Default, Debug)]
pub struct InputBlocks(pub std::collections::HashSet<std::any::TypeId>);

#[allow(dead_code)]
impl InputBlocks {
    pub fn is_blocked(&self) -> bool { !self.0.is_empty() }
    pub fn block<T: 'static>(&mut self) { self.0.insert(std::any::TypeId::of::<T>()); }
    pub fn unblock<T: 'static>(&mut self) { self.0.remove(&std::any::TypeId::of::<T>()); }
}

/// Screen transition style.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TransitionStyle {
    FadeBlack { duration: f32 },
    Cut,
}

/// Request a screen transition with visual effect.
#[derive(Event, Debug, Clone)]
#[allow(dead_code)]
pub struct ScreenTransitionEvent {
    pub to: GameState,
    pub style: TransitionStyle,
}

/// Cutscene step for data-driven scripted sequences (festivals, story events).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CutsceneStep {
    FadeOut(f32),
    FadeIn(f32),
    Wait(f32),
    ShowText(String, f32),
    Teleport(MapId),
    PlayBgm(String),
    PlaySfx(String),
    SetFlag(String, bool),
    StartDialogue(String),
    WaitForDialogueEnd,
}

/// Cutscene queue resource — runner pops front, executes, advances.
#[derive(Resource, Debug, Clone, Default)]
#[allow(dead_code)]
pub struct CutsceneQueue {
    pub steps: std::collections::VecDeque<CutsceneStep>,
    pub active: bool,
    pub step_timer: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// INPUT & MENU ABSTRACTION
// ═══════════════════════════════════════════════════════════════════════

/// All player actions as a single-frame snapshot.
/// Written by input reader systems. Consumed by all domains.
/// Reset to defaults at the start of each frame.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerInput {
    // Movement (continuous — pressed, not just_pressed)
    pub move_axis: Vec2,

    // Actions (just_pressed this frame)
    pub interact: bool,           // F — talk, pick up, open chest, shipping bin
    pub tool_use: bool,           // Space / LMB — swing tool
    pub tool_secondary: bool,     // R / RMB — eat food, place item

    // Menu toggles (just_pressed)
    pub open_inventory: bool,     // E
    pub open_crafting: bool,      // C
    pub open_map: bool,           // M
    pub open_journal: bool,       // J — quests/achievements
    pub pause: bool,              // Escape

    // Tool selection
    pub tool_next: bool,          // ] / scroll up
    pub tool_prev: bool,          // [ / scroll down
    pub tool_slot: Option<u8>,    // 1-9 → Some(0..8)

    // Fishing
    pub fishing_reel: bool,       // held (pressed, not just_pressed)

    // Mining combat (same as tool_use, context determines meaning)
    pub attack: bool,

    // UI navigation (menus, dialogue)
    pub ui_confirm: bool,         // Enter / E
    pub ui_cancel: bool,          // Escape
    pub ui_up: bool,
    pub ui_down: bool,
    pub ui_left: bool,
    pub ui_right: bool,

    // Meta
    pub any_key: bool,            // splash/title "press any key"
    pub skip_cutscene: bool,      // Space during cutscene
    pub quicksave: bool,          // F5
    pub quickload: bool,          // F9
}

/// Which input context is active. Determines which PlayerInput fields get written.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputContext {
    #[default]
    Gameplay,
    Menu,           // inventory, shop, crafting, chest, pause
    Dialogue,       // interact to advance, ui_up/down for choices
    Fishing,        // fishing_reel only
    Cutscene,       // skip_cutscene only
    Disabled,       // loading, transitions — nothing written
}

/// Keyboard binding table. Hardcoded defaults now, remappable later.
#[derive(Resource, Debug, Clone)]
pub struct KeyBindings {
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub interact: KeyCode,
    pub tool_use: KeyCode,
    pub tool_secondary: KeyCode,
    pub open_inventory: KeyCode,
    pub open_crafting: KeyCode,
    pub open_map: KeyCode,
    pub open_journal: KeyCode,
    pub pause: KeyCode,
    pub tool_next: KeyCode,
    pub tool_prev: KeyCode,
    pub ui_confirm: KeyCode,
    pub ui_cancel: KeyCode,
    pub skip_cutscene: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_up: KeyCode::KeyW,
            move_down: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            interact: KeyCode::KeyF,
            tool_use: KeyCode::Space,
            tool_secondary: KeyCode::KeyR,
            open_inventory: KeyCode::KeyE,
            open_crafting: KeyCode::KeyC,
            open_map: KeyCode::KeyM,
            open_journal: KeyCode::KeyJ,
            pause: KeyCode::Escape,
            tool_next: KeyCode::BracketRight,
            tool_prev: KeyCode::BracketLeft,
            ui_confirm: KeyCode::Enter,
            ui_cancel: KeyCode::Escape,
            skip_cutscene: KeyCode::Space,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MENU THEME & BUILDER TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Centralized menu styling. All menus read from this.
#[derive(Resource, Debug, Clone)]
pub struct MenuTheme {
    pub bg_overlay: Color,
    pub panel_bg: Color,
    pub panel_border: Color,
    pub panel_border_width: f32,
    pub panel_padding: f32,
    pub panel_gap: f32,
    pub panel_width: f32,

    pub button_bg_normal: Color,
    pub button_bg_selected: Color,
    pub button_bg_pressed: Color,
    pub button_border_normal: Color,
    pub button_border_selected: Color,
    pub button_height: f32,
    pub button_width: f32,
    pub button_border_width: f32,

    pub text_color: Color,
    pub text_color_selected: Color,
    pub text_color_disabled: Color,
    pub title_font_size: f32,
    pub button_font_size: f32,
    pub hint_font_size: f32,
}

impl Default for MenuTheme {
    fn default() -> Self {
        Self {
            bg_overlay: Color::srgba(0.0, 0.0, 0.0, 0.6),
            panel_bg: Color::srgba(0.1, 0.08, 0.06, 0.95),
            panel_border: Color::srgba(0.4, 0.35, 0.25, 0.8),
            panel_border_width: 3.0,
            panel_padding: 24.0,
            panel_gap: 12.0,
            panel_width: 320.0,

            button_bg_normal: Color::srgba(0.2, 0.17, 0.14, 0.8),
            button_bg_selected: Color::srgba(0.35, 0.3, 0.2, 0.95),
            button_bg_pressed: Color::srgba(0.45, 0.38, 0.25, 1.0),
            button_border_normal: Color::srgba(0.3, 0.25, 0.18, 0.6),
            button_border_selected: Color::srgb(1.0, 0.9, 0.5),
            button_height: 48.0,
            button_width: 260.0,
            button_border_width: 2.0,

            text_color: Color::srgba(0.9, 0.85, 0.7, 1.0),
            text_color_selected: Color::srgb(1.0, 0.95, 0.7),
            text_color_disabled: Color::srgba(0.5, 0.45, 0.4, 0.6),
            title_font_size: 42.0,
            button_font_size: 18.0,
            hint_font_size: 13.0,
        }
    }
}

/// Marker for any menu item that can be selected via keyboard or pointer.
#[derive(Component, Debug, Clone)]
pub struct MenuItem {
    pub index: usize,
}

/// Tracks which item is selected. Each menu manages its own cursor.
#[derive(Debug, Clone, Default)]
pub struct MenuCursor {
    pub index: usize,
    pub count: usize,
}

impl MenuCursor {
    pub fn new(count: usize) -> Self {
        Self { index: 0, count }
    }
    pub fn up(&mut self) {
        if self.count == 0 {
            return;
        }
        self.index = if self.index == 0 {
            self.count - 1
        } else {
            self.index - 1
        };
    }
    pub fn down(&mut self) {
        if self.count == 0 {
            return;
        }
        self.index = (self.index + 1) % self.count;
    }
    pub fn set(&mut self, idx: usize) {
        if idx < self.count {
            self.index = idx;
        }
    }
}

/// Frame-scoped menu actions from either keyboard or pointer.
/// Each menu's update system reads this to know what happened.
#[derive(Resource, Debug, Default)]
pub struct MenuAction {
    pub set_cursor: Option<usize>,
    pub activate: bool,
    pub cancel: bool,
    pub move_up: bool,
    pub move_down: bool,
    pub move_left: bool,
    pub move_right: bool,
}

// ═══════════════════════════════════════════════════════════════════════
// TOOL UTILITY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════

/// Returns all tile positions that a watering-can action covers, based on tier
/// and the direction the player is facing.
///
/// | Tier     | Pattern                                  |
/// |----------|------------------------------------------|
/// | Basic    | 1 tile — the target tile itself           |
/// | Copper   | 3 tiles in a line along `facing`          |
/// | Iron     | 5 tiles in a line along `facing`          |
/// | Gold     | 3 × 3 area (9 tiles) centred on target    |
/// | Iridium  | 6 × 6 area (36 tiles) centred on target   |
pub fn watering_can_area(
    tier: ToolTier,
    target_x: i32,
    target_y: i32,
    facing: Facing,
) -> Vec<(i32, i32)> {
    match tier {
        ToolTier::Basic => {
            vec![(target_x, target_y)]
        }
        ToolTier::Copper => {
            line_tiles(target_x, target_y, facing, 3)
        }
        ToolTier::Iron => {
            line_tiles(target_x, target_y, facing, 5)
        }
        ToolTier::Gold => {
            square_area(target_x, target_y, 1)
        }
        ToolTier::Iridium => {
            let half = 3_i32;
            let mut tiles = Vec::with_capacity(36);
            for dy in -(half - 1)..=half {
                for dx in -(half - 1)..=half {
                    tiles.push((target_x + dx, target_y + dy));
                }
            }
            tiles
        }
    }
}

/// Returns the stamina cost for a single tool-use action at the given tier.
pub fn tool_stamina_cost(base_cost: f32, tier: ToolTier) -> f32 {
    base_cost * tier.stamina_multiplier()
}

/// Build a straight line of `length` tiles starting at `(sx, sy)` and walking
/// one step per tile in the direction of `facing`.
fn line_tiles(sx: i32, sy: i32, facing: Facing, length: u8) -> Vec<(i32, i32)> {
    let (dx, dy) = facing_delta(facing);
    (0..length as i32)
        .map(|i| (sx + dx * i, sy + dy * i))
        .collect()
}

/// Build a square area of side `2 * radius + 1` centred on `(cx, cy)`.
fn square_area(cx: i32, cy: i32, radius: i32) -> Vec<(i32, i32)> {
    let side = 2 * radius + 1;
    let mut tiles = Vec::with_capacity((side * side) as usize);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            tiles.push((cx + dx, cy + dy));
        }
    }
    tiles
}

/// Returns the (dx, dy) unit step for each facing direction.
fn facing_delta(facing: Facing) -> (i32, i32) {
    match facing {
        Facing::Up    => (0,  1),
        Facing::Down  => (0, -1),
        Facing::Left  => (-1, 0),
        Facing::Right => (1,  0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── InputBlocks lifecycle ────────────────────────────────────────

    struct BlockerA;
    struct BlockerB;

    #[test]
    fn test_input_blocks_default_not_blocked() {
        let blocks = InputBlocks::default();
        assert!(!blocks.is_blocked());
    }

    #[test]
    fn test_input_blocks_block_unblock_lifecycle() {
        let mut blocks = InputBlocks::default();
        blocks.block::<BlockerA>();
        assert!(blocks.is_blocked());
        blocks.unblock::<BlockerA>();
        assert!(!blocks.is_blocked());
    }

    #[test]
    fn test_input_blocks_multiple_blockers() {
        let mut blocks = InputBlocks::default();
        blocks.block::<BlockerA>();
        blocks.block::<BlockerB>();
        assert!(blocks.is_blocked());
        blocks.unblock::<BlockerA>();
        assert!(blocks.is_blocked(), "Still blocked by BlockerB");
        blocks.unblock::<BlockerB>();
        assert!(!blocks.is_blocked());
    }

    #[test]
    fn test_input_blocks_double_block_same_type() {
        let mut blocks = InputBlocks::default();
        blocks.block::<BlockerA>();
        blocks.block::<BlockerA>(); // should be idempotent (HashSet)
        assert!(blocks.is_blocked());
        blocks.unblock::<BlockerA>();
        assert!(!blocks.is_blocked());
    }

    // ── BuildingTier ────────────────────────────────────────────────

    #[test]
    fn test_building_tier_next_full_chain() {
        let mut tier = BuildingTier::None;
        tier = tier.next().unwrap(); // Basic
        assert_eq!(tier, BuildingTier::Basic);
        tier = tier.next().unwrap(); // Big
        assert_eq!(tier, BuildingTier::Big);
        tier = tier.next().unwrap(); // Deluxe
        assert_eq!(tier, BuildingTier::Deluxe);
        assert_eq!(tier.next(), None);
    }

    #[test]
    fn test_building_tier_capacity_increases() {
        let none_cap = BuildingTier::None.capacity();
        let basic_cap = BuildingTier::Basic.capacity();
        let big_cap = BuildingTier::Big.capacity();
        let deluxe_cap = BuildingTier::Deluxe.capacity();
        assert!(none_cap < basic_cap);
        assert!(basic_cap < big_cap);
        assert!(big_cap < deluxe_cap);
    }

    // ── Season ──────────────────────────────────────────────────────

    #[test]
    fn test_season_next_wraps_winter_to_spring() {
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn test_season_full_cycle() {
        let mut season = Season::Spring;
        season = season.next(); // Summer
        assert_eq!(season, Season::Summer);
        season = season.next(); // Fall
        assert_eq!(season, Season::Fall);
        season = season.next(); // Winter
        assert_eq!(season, Season::Winter);
        season = season.next(); // Spring (wrap)
        assert_eq!(season, Season::Spring);
    }

    #[test]
    fn test_season_index_ordering() {
        assert_eq!(Season::Spring.index(), 0);
        assert_eq!(Season::Summer.index(), 1);
        assert_eq!(Season::Fall.index(), 2);
        assert_eq!(Season::Winter.index(), 3);
    }

    // ── ToolTier ────────────────────────────────────────────────────

    #[test]
    fn test_tool_tier_next_chain() {
        assert_eq!(ToolTier::Basic.next(), Some(ToolTier::Copper));
        assert_eq!(ToolTier::Copper.next(), Some(ToolTier::Iron));
        assert_eq!(ToolTier::Iron.next(), Some(ToolTier::Gold));
        assert_eq!(ToolTier::Gold.next(), Some(ToolTier::Iridium));
        assert_eq!(ToolTier::Iridium.next(), None);
    }

    #[test]
    fn test_tool_tier_upgrade_cost_increases() {
        let basic = ToolTier::Basic.upgrade_cost_gold();
        let copper = ToolTier::Copper.upgrade_cost_gold();
        let iron = ToolTier::Iron.upgrade_cost_gold();
        let gold = ToolTier::Gold.upgrade_cost_gold();
        assert!(basic < copper);
        assert!(copper < iron);
        assert!(iron < gold);
    }

    #[test]
    fn test_tool_tier_iridium_no_upgrade() {
        assert_eq!(ToolTier::Iridium.next(), None);
        assert_eq!(ToolTier::Iridium.upgrade_cost_gold(), 0);
        assert_eq!(ToolTier::Iridium.upgrade_bars_needed(), 0);
        assert_eq!(ToolTier::Iridium.upgrade_bar_item(), None);
    }

    #[test]
    fn test_tool_tier_stamina_multiplier_decreases() {
        assert!(ToolTier::Basic.stamina_multiplier() > ToolTier::Copper.stamina_multiplier());
        assert!(ToolTier::Copper.stamina_multiplier() > ToolTier::Iron.stamina_multiplier());
        assert!(ToolTier::Iron.stamina_multiplier() > ToolTier::Gold.stamina_multiplier());
        assert!(ToolTier::Gold.stamina_multiplier() > ToolTier::Iridium.stamina_multiplier());
    }

    // ── Inventory ───────────────────────────────────────────────────

    #[test]
    fn test_inventory_has_empty() {
        let inv = Inventory::default();
        assert!(!inv.has("turnip", 1));
        assert_eq!(inv.count("turnip"), 0);
    }

    #[test]
    fn test_inventory_try_add_and_has() {
        let mut inv = Inventory::default();
        let leftover = inv.try_add("turnip", 5, 99);
        assert_eq!(leftover, 0);
        assert!(inv.has("turnip", 5));
        assert!(inv.has("turnip", 1));
        assert!(!inv.has("turnip", 6));
    }

    #[test]
    fn test_inventory_try_remove() {
        let mut inv = Inventory::default();
        inv.try_add("turnip", 10, 99);
        let removed = inv.try_remove("turnip", 3);
        assert_eq!(removed, 3);
        assert_eq!(inv.count("turnip"), 7);
    }

    #[test]
    fn test_inventory_try_remove_more_than_available() {
        let mut inv = Inventory::default();
        inv.try_add("turnip", 2, 99);
        let removed = inv.try_remove("turnip", 5);
        assert_eq!(removed, 2);
        assert_eq!(inv.count("turnip"), 0);
        assert!(!inv.has("turnip", 1));
    }

    #[test]
    fn test_inventory_try_remove_nonexistent() {
        let mut inv = Inventory::default();
        let removed = inv.try_remove("turnip", 1);
        assert_eq!(removed, 0);
    }

    #[test]
    fn test_inventory_slot_cleared_when_empty() {
        let mut inv = Inventory::default();
        inv.try_add("turnip", 1, 99);
        assert!(inv.slots.iter().any(|s| s.is_some()));
        inv.try_remove("turnip", 1);
        // After removing all, the slot should be None
        let turnip_slots: Vec<_> = inv.slots.iter()
            .filter(|s| s.as_ref().map_or(false, |s| s.item_id == "turnip"))
            .collect();
        assert!(turnip_slots.is_empty());
    }

    // ── ItemQuality ─────────────────────────────────────────────────

    #[test]
    fn test_item_quality_sell_multiplier() {
        assert!((ItemQuality::Normal.sell_multiplier() - 1.0).abs() < f32::EPSILON);
        assert!((ItemQuality::Silver.sell_multiplier() - 1.25).abs() < f32::EPSILON);
        assert!((ItemQuality::Gold.sell_multiplier() - 1.5).abs() < f32::EPSILON);
        assert!((ItemQuality::Iridium.sell_multiplier() - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_item_quality_next() {
        assert_eq!(ItemQuality::Normal.next(), Some(ItemQuality::Silver));
        assert_eq!(ItemQuality::Silver.next(), Some(ItemQuality::Gold));
        assert_eq!(ItemQuality::Gold.next(), Some(ItemQuality::Iridium));
        assert_eq!(ItemQuality::Iridium.next(), None);
    }

    // ── Relationships ───────────────────────────────────────────────

    #[test]
    fn test_relationships_hearts_calculation() {
        let mut rel = Relationships::default();
        rel.friendship.insert("elena".to_string(), 500);
        assert_eq!(rel.hearts("elena"), 5);
    }

    #[test]
    fn test_relationships_add_friendship_clamps() {
        let mut rel = Relationships::default();
        rel.add_friendship("elena", 2000);
        // Should clamp to 1000
        assert_eq!(*rel.friendship.get("elena").unwrap(), 1000);
    }

    #[test]
    fn test_relationships_add_friendship_negative_clamps_to_zero() {
        let mut rel = Relationships::default();
        rel.add_friendship("elena", 50);
        rel.add_friendship("elena", -100);
        assert_eq!(*rel.friendship.get("elena").unwrap(), 0);
    }

    // ── SprinklerKind ───────────────────────────────────────────────

    #[test]
    fn test_sprinkler_kind_range() {
        assert_eq!(SprinklerKind::Basic.range(), 1);
        assert_eq!(SprinklerKind::Quality.range(), 1);
        assert_eq!(SprinklerKind::Iridium.range(), 2);
    }

    #[test]
    fn test_sprinkler_kind_diagonals() {
        assert!(!SprinklerKind::Basic.includes_diagonals());
        assert!(SprinklerKind::Quality.includes_diagonals());
        assert!(SprinklerKind::Iridium.includes_diagonals());
    }

    // ── Calendar ────────────────────────────────────────────────────

    #[test]
    fn test_calendar_default() {
        let cal = Calendar::default();
        assert_eq!(cal.year, 1);
        assert_eq!(cal.season, Season::Spring);
        assert_eq!(cal.day, 1);
        assert_eq!(cal.hour, 6);
        assert_eq!(cal.minute, 0);
    }

    #[test]
    fn test_calendar_time_float_midnight() {
        let mut cal = Calendar::default();
        cal.hour = 24;
        cal.minute = 0;
        assert!((cal.time_float() - 24.0).abs() < f32::EPSILON);
    }
}
