use bevy::prelude::*;

pub const TARGET_FPS: f32 = 60.0;

pub const GAME_TITLE: &str = "Greenfield Demo";

pub const RECORDING_VERSION: u32 = 1;

#[derive(Resource, Debug, Default)]
pub struct AudioManager;

#[derive(Resource, Debug, Default)]
pub struct SettingsCache;

#[derive(Resource, Debug, Default)]
pub struct TurnClock {
    pub turn: u32,
    pub elapsed_secs: f32,
}

#[derive(Resource, Debug, Default)]
pub struct RecordingBuffer {
    pub started_at_secs: f32,
    pub event_count: u32,
}

#[derive(Resource, Debug, Default)]
pub struct GameConfig {
    pub target_fps: f32,
    pub recording_enabled: bool,
}

#[derive(Resource, Debug, Default)]
pub struct EnemyCatalog;

#[derive(Resource, Debug, Default)]
pub struct LootTable;

#[derive(Resource, Debug, Default)]
pub struct SpawnerRegistry;

#[derive(Resource, Debug, Default)]
pub struct GameScore {
    pub total: u32,
    pub high: u32,
}

#[derive(Resource, Debug, Default)]
pub struct LevelProgress {
    pub level: u32,
    pub xp: u32,
}

pub const MAX_HEALTH: i32 = 100;

pub const BASE_DAMAGE: i32 = 10;

pub const CRIT_MULTIPLIER: f32 = 1.5;

pub const RESPAWN_TIME: f32 = 3.0;

pub const MAX_ENEMIES: u32 = 50;

pub const MAX_INVENTORY: u32 = 64;

pub const XP_PER_LEVEL: u32 = 100;

pub const CRIT_CHANCE: f32 = 0.15;

#[derive(Resource, Debug, Default)]
pub struct BenchmarkBudget;

pub const BENCHMARK_TIMEOUT_MS: u64 = 5000;

#[derive(Resource, Debug, Default)]
pub struct McpScaleRes01;

#[derive(Resource, Debug, Default)]
pub struct McpScaleRes02;

#[derive(Resource, Debug, Default)]
pub struct McpScaleRes03;

#[derive(Resource, Debug, Default)]
pub struct McpScaleRes04;

pub const MCP_SCALE_LIMIT_01: u32 = 100;

pub const MCP_SCALE_LIMIT_02: u32 = 200;

pub const MCP_SCALE_LIMIT_03: u32 = 300;

pub const MCP_SCALE_LIMIT_04: u32 = 400;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes01;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes02;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes03;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes04;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes05;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes06;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes07;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes08;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes09;

#[derive(Resource, Debug, Default)]
pub struct BigBatchRes10;

pub const BIG_BATCH_LIMIT_01: u32 = 100;

pub const BIG_BATCH_LIMIT_02: u32 = 200;

pub const BIG_BATCH_LIMIT_03: u32 = 300;

pub const BIG_BATCH_LIMIT_04: u32 = 400;

pub const BIG_BATCH_LIMIT_05: u32 = 500;

pub const BIG_BATCH_LIMIT_06: u32 = 600;

pub const BIG_BATCH_LIMIT_07: u32 = 700;

pub const BIG_BATCH_LIMIT_08: u32 = 800;

pub const BIG_BATCH_LIMIT_09: u32 = 900;

pub const BIG_BATCH_LIMIT_10: u32 = 1000;

#[derive(Resource, Debug, Default)]
pub struct FinisherRes01;

#[derive(Resource, Debug, Default)]
pub struct FinisherRes02;

#[derive(Resource, Debug, Default)]
pub struct FinisherRes03;

#[derive(Resource, Debug, Default)]
pub struct BareRes01;

#[derive(Resource, Debug, Default)]
pub struct BareRes02;

#[derive(Resource, Debug, Default)]
pub struct BareRes03;

#[derive(Resource, Debug, Default)]
pub struct BareRes04;

#[derive(Resource, Debug, Default)]
pub struct BareRes05;

#[derive(Resource, Debug, Default)]
pub struct BareRes06;

#[derive(Resource, Debug, Default)]
pub struct BareRes07;

#[derive(Resource, Debug, Default)]
pub struct BareRes08;

#[derive(Resource, Debug, Default)]
pub struct BareRes09;

#[derive(Resource, Debug, Default)]
pub struct BareRes10;

#[derive(Resource, Debug, Default)]
pub struct TrialA3Res;

#[derive(Resource, Debug, Default)]
pub struct BroadcastScore;

#[derive(Resource, Debug, Default)]
pub struct CombatClock;

#[derive(Resource, Debug, Default)]
pub struct CurrentCombatant;

pub const COMBAT_TIMEOUT_MS: u64 = 5000;

pub const MAX_DAMAGE_PER_TICK: u32 = 50;

pub const CRITICAL_HIT_MULTIPLIER: u32 = 3;

#[derive(Resource, Debug, Default)]
pub struct ActiveInventory;

#[derive(Resource, Debug, Default)]
pub struct InventoryConfig;

#[derive(Resource, Debug, Default)]
pub struct ItemCatalog;

pub const MAX_INVENTORY_SIZE: u32 = 64;

pub const MAX_STACK_SIZE: u32 = 99;

pub const PICKUP_RADIUS_PX: u32 = 32;

pub const DROP_DELAY_MS: u64 = 250;

#[derive(Resource, Debug, Default)]
pub struct ActiveCrafting;

#[derive(Resource, Debug, Default)]
pub struct RecipeBook;

#[derive(Resource, Debug, Default)]
pub struct CraftingConfig;

pub const MAX_CRAFTING_SLOTS: u32 = 8;

pub const CRAFTING_DURATION_MS: u64 = 3000;

pub const MAX_RECIPES: u32 = 128;

pub const MAX_OUTPUT_STACK: u32 = 50;

pub const CRAFTING_TICK_RATE: u32 = 60;

#[derive(Resource, Debug, Default)]
pub struct FrameTelemetry;

#[derive(Resource, Debug, Default)]
pub struct TickCounter {
    pub value: u32,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct PlayerHealth {
    pub hp: f32,
    pub max_hp: f32,

}

#[derive(Resource, Default)]
pub struct CooldownClock {
pub remaining: f32,
}

#[derive(Resource, Debug, Default)]
pub struct QuestLog {
    pub active: Vec<Quest>,
    pub completed: Vec<String>,
}

pub const MAX_ACTIVE_QUESTS: u32 = 8;

pub const QUEST_XP_REWARD: u32 = 50;

#[derive(Resource, Debug, Default)]
pub struct MusicState {
    pub current_layer: u8,
    pub crossfade_progress: u32,
    pub current_track: Option<Entity>,
    pub current_track_id: String,
}

pub const MUSIC_MAX_LAYERS: u8 = 4;

pub const CROSSFADE_DURATION_MS: u64 = 1500;

#[derive(Resource, Debug, Default)]
pub struct CheckpointState {
    pub last_id: u32,
    pub visits: u32,
}

pub const MAX_CHECKPOINTS: u32 = 32;

pub const CHECKPOINT_COOLDOWN_SECS: u64 = 5;


// ============================================================
// AUTO-INTEGRATED FROM HEARTHFIELD via substrate planner
// ============================================================
pub const BACKPACK_SLOTS: usize = 24;
pub const HOTBAR_SLOTS: usize = 12;
pub const TOTAL_INVENTORY_SLOTS: usize = HOTBAR_SLOTS + BACKPACK_SLOTS;
/// Additional bar size per skill level in pixels.
pub const BAR_SIZE_PER_LEVEL_PX: f32 = 3.0;
/// Base catch bar size in pixels.
pub const BASE_BAR_SIZE_PX: f32 = 40.0;
/// Bite wait reduction per level in seconds.
pub const BITE_WAIT_REDUCTION_PER_LEVEL: f32 = 0.5;
/// XP thresholds for each level (1–10). Index 0 = level 1 threshold.
pub const LEVEL_THRESHOLDS: [u32; 10] = [10, 25, 50, 100, 200, 350, 550, 800, 1100, 1500];
/// Max skill level.
pub const MAX_LEVEL: u32 = 10;
pub const MINIGAME_BAR_HEIGHT: f32 = 200.0;
pub const MINIGAME_BAR_WIDTH: f32 = 40.0;
/// Overlap ratio required for a "perfect catch" bonus (quality upgrade).
pub const PERFECT_CATCH_THRESHOLD: f32 = 0.90;
pub const PIXEL_SCALE: f32 = 3.0; // render scale (16px × 3 = 48px on screen)
pub const PROGRESS_BAR_HEIGHT: f32 = 12.0;
pub const PROGRESS_BAR_WIDTH: f32 = 120.0;
pub const PROGRESS_BAR_Y: f32 = -130.0;
pub const SCREEN_WIDTH: f32 = 960.0;
pub const Z_UI_BG: f32 = 50.0;
/// Fade speed for map transitions: 1.0 / 0.42s = ~2.38 alpha/s.
pub const MAP_TRANSITION_FADE_SPEED: f32 = 1.0 / 0.42;
pub const MAP_TRANSITION_HOLD_TIME: f32 = 0.16;
/// Fade speed for save/load handoffs: 1.0 / 0.58s = ~1.72 alpha/s.
pub const SAVE_LOAD_FADE_SPEED: f32 = 1.0 / 0.58;
pub const SAVE_LOAD_HOLD_TIME: f32 = 0.28;
pub const DAYS_PER_SEASON: u8 = 28;
pub const SEASONS_PER_YEAR: u8 = 4;
pub const COLLISION_FILE: &str = "/tmp/hearthfield-collision.json";
pub const STATE_FILE: &str = "/tmp/hearthfield-state.json";
pub const FRIENDSHIP_PER_HEART: u32 = 100;
pub const MAX_FRIENDSHIP: u32 = MAX_HEARTS * FRIENDSHIP_PER_HEART;
pub const MAX_HEARTS: u32 = 10;
pub const FADE_DURATION: f32 = 0.5;
pub const TILE_SIZE: f32 = 16.0;

#[derive(Resource, Debug, Clone)]
pub struct CaughtFishEntry {
    pub fish_id: String,
    pub times_caught: u32,
    pub first_caught_day: u32,
    pub first_caught_season: Season,
}

/// Cutscene queue resource — runner pops front, executes, advances.
#[derive(Resource, Debug, Clone, Default)]
pub struct CutsceneQueue {
    pub steps: std::collections::VecDeque<CutsceneStep>,
    pub active: bool,
    pub step_timer: f32,
}

/// Cutscene step for data-driven scripted sequences (festivals, story events).
#[allow(dead_code)]
#[derive(Debug, Clone)]
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
    /// Start dialogue with custom lines (not from NPC registry).
    StartDialogueCustom {
        npc_id: String,
        lines: Vec<String>,
        portrait_index: Option<u32>,
    },
    WaitForDialogueEnd,
}

#[derive(Resource, Debug, Clone)]
pub struct DialogueEndEvent;

pub struct DialoguePrompt;

pub struct DialogueText;

/// Tracks every species the player has ever caught.
#[derive(Resource, Debug, Clone, Default)]
pub struct FishEncyclopedia {
    /// fish_id → CaughtFishEntry
    pub entries: HashMap<String, CaughtFishEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    Dialogue,
    Shop,
    Fishing,
    Mining,
    Crafting,
    Inventory,
    Journal,
    Cutscene,
    BuildingUpgrade,
    RelationshipsView,
    MapView,
    FishEncyclopedia,
}

#[derive(Resource, Debug, Clone)]
pub struct Inventory {
    /// `TOTAL_INVENTORY_SLOTS` slots: 0-11 = hotbar, 12-35 = backpack
    pub slots: Vec<Option<InventorySlot>>,
    pub selected_slot: usize,
}

/// Unique identifier for every item pub type in the game.
/// Using string IDs for data-driven flexibility.
pub type ItemId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapId {
    Farm,
    Town,
    TownWest,
    Beach,
    Forest,
    DeepForest,
    CoralIsland,
    MineEntrance,
    Mine, // + floor number in MineState
    PlayerHouse,
    TownHouseWest,
    TownHouseEast,
    GeneralStore,
    AnimalShop,
    Blacksmith,
    Library,
    Tavern,
    SnowMountain,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct MineFloor(pub u8);


#[derive(Resource, Debug, Clone, Default)]
pub struct MineState {
    pub current_floor: MineFloor,         // 0 = not in mine
    pub deepest_floor_reached: MineFloor, // for elevator
    pub elevator_floors: Vec<u8>,  // unlocked elevator stops (every 5)
}

pub type NpcId = String;

/// All player actions as a single-frame snapshot.
/// Written by input reader systems. Consumed by all domains.
/// Reset to defaults at the start of each frame.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlayerInput {
    // Movement (continuous — pressed, not just_pressed)
    pub move_axis: Vec2,

    // Actions (just_pressed this frame)
    pub interact: bool,       // F — talk, pick up, open chest, shipping bin
    pub tool_use: bool,       // Space / LMB — swing tool
    pub tool_secondary: bool, // R / RMB — eat food, place item

    // Menu toggles (just_pressed)
    pub open_inventory: bool,     // E
    pub open_crafting: bool,      // C
    pub open_map: bool,           // M
    pub open_journal: bool,       // J — quests/achievements
    pub open_relationships: bool, // L — relationships screen
    pub pause: bool,              // Escape

    // Tool selection
    pub tool_next: bool,       // ] / scroll up
    pub tool_prev: bool,       // [ / scroll down
    pub tool_slot: Option<u8>, // 1-9 → Some(0..8)

    // Fishing
    pub fishing_reel: bool, // held (pressed, not just_pressed)

    // Mining combat (same as tool_use, context determines meaning)
    pub attack: bool,

    // UI navigation (menus, dialogue)
    pub ui_confirm: bool, // Enter / E
    pub ui_cancel: bool,  // Escape
    pub ui_up: bool,
    pub ui_down: bool,
    pub ui_left: bool,
    pub ui_right: bool,
    pub tab_pressed: bool, // Tab key (panel switch, mode cycle)

    // Meta
    pub any_key: bool,       // splash/title "press any key"
    pub skip_cutscene: bool, // Space during cutscene
    pub quicksave: bool,     // F5
    pub quickload: bool,     // F9
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Season {
    Spring,
    Summer,
    Fall,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildingKind {
    House,
    Coop,
    Barn,
    Silo,
}

/// Upgrade tiers for farm buildings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildingTier {
    #[default]
    None,
    Basic,
    Big,
    Deluxe,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct Gold(pub u32);


#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct Stamina(pub f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolTier {
    Basic,
    Copper,
    Iron,
    Gold,
    Iridium,
}

/// All runtime state for the fishing minigame.
#[derive(Resource, Debug)]
pub struct FishingMinigameState {
    /// Fish zone center position, 0.0 (bottom) to 100.0 (top).
    pub fish_zone_center: f32,
    /// Current velocity of the fish zone.
    pub fish_zone_velocity: f32,
    /// Direction timer — fish changes direction occasionally.
    pub direction_change_timer: Timer,
    /// Catch bar center position, 0.0 to 100.0.
    pub catch_bar_center: f32,
    /// Catch bar half-height (25 base, boosted by tackle).
    pub catch_bar_half: f32,
    /// Fish zone half-height (based on difficulty — easier fish have bigger zones).
    pub fish_zone_half: f32,
    /// Progress: 0.0 → 100.0. Fills when overlapping, drains when not.
    pub progress: f32,
    /// Difficulty of the current fish (0.0–1.0).
    pub fish_difficulty: f32,
    /// Whether Space is currently held.
    pub space_held: bool,
    /// Time since minigame started (for sfx pacing etc.)
    pub elapsed: f32,
    /// Quick sound pulse for overlap
    pub overlap_sfx_cooldown: f32,
    /// Multiplier on the progress drain rate (1.0 = normal; 0.5 = Trap Bobber).
    pub progress_drain_multiplier: f32,
    /// Multiplier on the catch bar fall speed (1.0 = normal; 0.7 = Lead Bobber).
    pub catch_fall_multiplier: f32,
    /// Total time (seconds) the catch bar was overlapping the fish zone this game.
    pub overlap_time_total: f32,
    /// Total time (seconds) the minigame has been running (excluding the ramp-up grace period).
    pub minigame_total_time: f32,
}

/// Persistent fishing skill that improves as the player catches more fish.
#[derive(Resource, Debug, Clone, Default)]
pub struct FishingSkill {
    /// Cumulative XP earned from catching fish.
    pub xp: u32,
    /// Cumulative count of all fish successfully caught.
    pub total_catches: u32,
    /// Current skill level (0–10). Level 0 = beginner.
    pub level: u32,
    /// Fraction by which the bite timer is reduced (0.0 → 0.5).
    /// Applied as: effective_wait = base_wait * (1.0 - bite_speed_bonus).
    pub bite_speed_bonus: f32,
    /// Fraction added to the catch bar half-height (0.0 → 0.3).
    /// Applied as: catch_bar_half *= (1.0 + catch_zone_bonus).
    pub catch_zone_bonus: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

/// Specific tackle items that modify minigame parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TackleKind {
    /// No tackle equipped — no modifier.
    #[default]
    None,
    /// Spinner: enlarges the fish zone by 25% (easier target).
    Spinner,
    /// Trap Bobber: slows progress drain by 50% (more forgiving).
    TrapBobber,
    /// Lead Bobber: reduces catch bar fall speed by 30% (easier to hold up).
    LeadBobber,
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone)]
pub struct MapTransition {
    pub from_map: MapId,
    pub from_rect: (i32, i32, i32, i32), // x, y, w, h trigger area
    pub to_map: MapId,
    pub to_pos: (i32, i32),
}

#[derive(Clone, Copy)]
pub enum ScreenFadeTint {
    MapTransition,
    SaveLoad,
}

/// Sent by SavePlugin after a load completes.
#[derive(Resource, Debug, Clone)]
pub struct LoadCompleteEvent {
    pub slot: u8,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Sent by UI to trigger loading a specific slot.
#[derive(Resource, Debug, Clone)]
pub struct LoadRequestEvent {
    pub slot: u8,
}

#[derive(Resource, Debug, Clone)]
pub struct MapTransitionEvent {
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

pub struct SavePlugin;

/// Identifies which composite building sprite to use (if any).
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BuildingImage {
    Farmhouse,
    Barn,
    ChickenHouse,
    Well,
}

/// Caches loaded texture atlas handles for world objects.
/// Loaded lazily on first map spawn.
#[derive(Resource, Default)]
pub struct ObjectAtlases {
    pub loaded: bool,
    pub grass_biome_image: Handle<Image>,
    pub grass_biome_layout: Handle<TextureAtlasLayout>,
    pub item_icon_image: Handle<Image>,
    pub item_icon_layout: Handle<TextureAtlasLayout>,
    pub fences_image: Handle<Image>,
    pub fences_layout: Handle<TextureAtlasLayout>,
    // Tree sprites atlas (32×48 cells, seasonal variants)
    pub tree_sprites_image: Handle<Image>,
    pub tree_sprites_layout: Handle<TextureAtlasLayout>,
    // Building tilesets (Sprout Lands)
    pub house_walls_image: Handle<Image>,
    pub house_walls_layout: Handle<TextureAtlasLayout>,
    pub house_roof_image: Handle<Image>,
    pub house_roof_layout: Handle<TextureAtlasLayout>,
    pub doors_image: Handle<Image>,
    pub doors_layout: Handle<TextureAtlasLayout>,
    pub door_anim_image: Handle<Image>,
    pub door_anim_layout: Handle<TextureAtlasLayout>,
    pub hills_image: Handle<Image>,
    pub hills_layout: Handle<TextureAtlasLayout>,
    pub wood_bridge_image: Handle<Image>,
    pub wood_bridge_layout: Handle<TextureAtlasLayout>,
    pub tools_image: Handle<Image>,
    pub tools_layout: Handle<TextureAtlasLayout>,
    // Modern Farm composite building sprites (single images, no atlas)
    pub farmhouse_image: Handle<Image>,
    pub barn_image: Handle<Image>,
    pub chicken_house_image: Handle<Image>,
    pub well_image: Handle<Image>,
    // Individual biome-specific tree PNGs (no atlas — full-image sprites)
    pub tree_oak_green_image: Handle<Image>,   // 80×96 — Farm/Town spring+summer
    pub tree_oak_brown_image: Handle<Image>,   // 80×96 — Farm/Town fall
    pub tree_birch_green_image: Handle<Image>, // 48×80 — Forest/DeepForest
    pub tree_pine_blue_image: Handle<Image>,   // 64×96 — SnowMountain
    // Premium modern farm fence atlas (512×272, 32 cols × 17 rows, 16×16 tiles)
    // Contains multiple fence styles: hedges, planks, stone walls, gates, chains.
    pub modern_fences_image: Handle<Image>,
    pub modern_fences_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Debug, Clone)]
pub struct Calendar {
    pub year: u32,
    pub season: Season,
    pub day: u8,    // 1-28
    pub hour: u8,   // 6-25 (25 = 1:00 AM next day)
    pub minute: u8, // 0-59
    pub weather: Weather,
    pub time_scale: f32, // game-minutes per real-second (default 1/6 => 10 game-minutes per real-minute)
    pub time_paused: bool,
    pub elapsed_real_seconds: f32, // accumulator for sub-minute ticks
}

#[derive(Resource, Debug, Clone)]
pub struct DayEndEvent {
    pub day: u8,
    pub season: Season,
    pub year: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayOfWeek {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

/// Stores the weather of the most recently ended day so other domains can
/// check whether it rained *today* (the ended day) rather than tomorrow.
/// Updated every time a DayEndEvent is processed.
#[derive(Resource, Debug, Clone)]
pub struct PreviousDayWeather {
    pub weather: Weather,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Weather {
    Sunny,
    Rainy,
    Stormy,
    Snowy, // Winter only
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

/// A single ambient firefly particle.
#[derive(Resource, Debug)]
pub struct Firefly {
    /// Lifetime timer; the particle despawns when it finishes.
    pub timer: Timer,
    /// Current drift velocity in world units per second.
    pub drift_direction: Vec2,
    /// Baseline alpha before the pulse is applied.
    pub base_alpha: f32,
    /// Phase offset so nearby particles do not pulse in sync.
    pub pulse_phase: f32,
    /// Pulse frequency in radians per second.
    pub pulse_speed: f32,
}

/// Tracks the desired swarm size for the active dusk window.
#[derive(Resource, Debug, Default)]
pub struct FireflySwarmState {
    pub target_count: Option<usize>,
}

#[derive(Resource, Debug, Clone)]
pub struct PlayerState {
    pub stamina: Stamina,
    pub max_stamina: f32,
    pub health: Health,
    pub max_health: f32,
    pub equipped_tool: ToolKind,
    pub tools: HashMap<ToolKind, ToolTier>,
    pub gold: Gold,
    pub current_map: MapId,
        pub save_grid_x: i32,
        pub save_grid_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolKind {
    Hoe,
    WateringCan,
    Axe,
    Pickaxe,
    FishingRod,
    Scythe,
}

#[derive(Resource, Debug, Clone)]
pub struct CropTile {
    pub crop_id: ItemId,
    pub current_stage: u8,
    pub days_in_stage: u8,
    pub watered_today: bool,
    pub days_without_water: u8,
    pub dead: bool,
}

#[derive(Debug, Clone)]
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

#[derive(Resource, Debug, Clone, Default)]
pub struct FarmState {
    /// Tiles that have been tilled/watered. Key = (x, y).
    pub soil: HashMap<(i32, i32), SoilState>,
    /// Active crops. Key = (x, y).
    pub crops: HashMap<(i32, i32), CropTile>,
    /// Objects on the farm (trees, rocks, stumps). Key = (x, y).
    pub objects: HashMap<(i32, i32), FarmObject>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ShippingBin {
    pub items: Vec<InventorySlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoilState {
    Untilled,
    Tilled,
    Watered,
}

#[derive(Resource, Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<(ItemId, u8)>, // (item_id, quantity)
    pub result: ItemId,
    pub result_quantity: u8,
    pub is_cooking: bool, // true = cooking, false = crafting
    pub unlocked_by_default: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct InsertMachineInputEvent {
    pub machine_entity: Entity,
    pub item_id: ItemId,
    pub quantity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Resource, Debug, Clone)]
pub struct ItemDef {
    pub id: ItemId,
    pub name: String,
    pub description: String,
    pub category: ItemCategory,
    pub sell_price: u32,
    pub buy_price: Option<u32>, // None = not buyable
    pub stack_size: StackSize,  // max per slot (1 for tools, 99 for most items)
    pub edible: bool,
    pub energy_restore: f32, // if edible
    pub sprite_index: u32,   // atlas index
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ItemRegistry {
    pub items: HashMap<ItemId, ItemDef>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Resource, Debug, Clone)]
pub struct PlaySfxEvent {
    pub sfx_id: String,
}

#[derive(Resource, Debug, Clone)]
pub struct ProcessingMachine {
    pub machine_type: MachineType,
    pub input_item: Option<ItemId>,
    pub output_item: Option<ItemId>,
    /// Remaining processing time in game hours.
    pub processing_time_remaining: f32,
    pub is_ready: bool,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct StackSize(pub u8);


/// Toast notification for player feedback.
#[derive(Resource, Debug, Clone)]
pub struct ToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct CollectMachineOutputEvent {
    pub machine_entity: Entity,
}

#[derive(Resource, Debug, Clone)]
pub struct ItemPickupEvent {
    pub item_id: ItemId,
    pub quantity: u8,
}

/// Applied to a rock entity on pickaxe hit.
/// Sets the sprite colour to a bright overexposed white for 0.08 s then removes itself.
#[derive(Resource, Debug)]
pub struct DamageFlash {
    pub timer: Timer,
}

/// Tracks economy statistics for save data and achievements.
#[derive(Resource, Debug, Clone, Default)]
pub struct EconomyStats {
    pub total_gold_earned: u64,
    pub total_gold_spent: u64,
    pub total_items_shipped: u64,
    pub total_transactions: u64,
}

#[derive(Resource, Debug, Clone)]
pub struct GoldChangeEvent {
    pub amount: i32, // positive = gain, negative = spend
    pub reason: String,
}

/// Collision map for the current area.
#[derive(Resource, Default)]
pub struct CollisionMap {
    pub solid_tiles: std::collections::HashSet<(i32, i32)>,
    pub bounds: (i32, i32, i32, i32),
    pub initialised: bool,
}

/// Simple resource to track the currently loaded map ID.
#[derive(Resource, Debug, Clone)]
pub struct CurrentMapId {
    pub map_id: MapId,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Player {
    pub gold: u32,
}

#[derive(Resource, Debug, Clone)]
pub struct Animal {
    pub kind: AnimalKind,
    pub name: String,
    pub age: AnimalAge,
    pub days_old: u16,
    pub happiness: Happiness,
    pub fed_today: bool,
    pub petted_today: bool,
    pub product_ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimalAge {
    Baby,
    Adult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimalKind {
    Chicken,
    Cow,
    Sheep,
    Goat,
    Duck,
    Rabbit,
    Pig,
    Horse,
    Cat,
    Dog,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct AnimalState {
    pub animals: Vec<Animal>,
    pub has_coop: bool,
    pub has_barn: bool,
    pub coop_level: BuildingLevel, // 0=none, 1=basic, 2=big, 3=deluxe
    pub barn_level: BuildingLevel,
}

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct BuildingLevel(pub u8);


/// Year-end evaluation score (grandpa's shrine).
#[derive(Resource, Debug, Clone, Default)]
pub struct EvaluationScore {
    pub total_points: u32,
    pub categories: HashMap<String, u32>,
    pub evaluated: bool,
    pub candles_lit: u8,
}

/// Trigger year-end evaluation.
#[derive(Resource, Debug, Clone)]
pub struct EvaluationTriggerEvent;

#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct Friendship(pub u32);


#[derive(Resource, Default, Clone, Copy, Debug, PartialEq)]
pub struct Happiness(pub u8);


/// Accumulated statistics about crop harvests.
/// Key = crop_id, Value = (total_harvested_count, total_revenue_gold).
#[derive(Resource, Debug, Clone, Default)]
pub struct HarvestStats {
    pub crops: HashMap<String, (u32, u32)>,
}

/// Tracks house upgrade state.
#[derive(Resource, Debug, Clone, Default)]
pub struct HouseState {
    pub tier: HouseTier,
    pub has_kitchen: bool, // Big+ house
    pub has_nursery: bool, // Deluxe house
}

/// House upgrade tier. Determines available features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum HouseTier {
    #[default]
    Basic,
    Big,
    Deluxe,
}

/// Marriage state tracking.
#[derive(Resource, Debug, Clone, Default)]
pub struct MarriageState {
    pub spouse: Option<String>,
    pub wedding_date: Option<(u8, u8, u16)>, // (day, season_idx, year)
    pub days_married: u32,
    pub spouse_happiness: i16, // -100 to 100
}

/// Tracks total play statistics for achievements and end-of-year summary.
#[derive(Resource, Debug, Clone, Default)]
pub struct PlayStats {
    pub crops_harvested: u64,
    pub fish_caught: u64,
    pub items_shipped: u64,
    pub gifts_given: u64,
    pub mine_floors_cleared: u64,
        pub animal_products_collected: u64,
        pub food_eaten: u64,
    pub total_gold_earned: u64,
    pub total_steps_taken: u64,
    pub days_played: u64,
    pub festivals_attended: u64,
}

#[derive(Resource, Debug, Clone)]
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

#[derive(Resource, Debug, Clone, Default)]
pub struct Relationships {
    /// NPC id → friendship points (0-1000, 100 per heart)
    pub friendship: HashMap<NpcId, Friendship>,
    pub gifted_today: HashMap<NpcId, bool>,
    pub spouse: Option<NpcId>,
}

/// Tracks what the player has shipped at least once (for collection tracking).
#[derive(Resource, Debug, Clone, Default)]
pub struct ShippingLog {
    pub shipped_items: HashMap<ItemId, u32>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct UnlockedRecipes {
    pub ids: Vec<String>,
}

/// Cached sprite atlas handles for fishing-related sprites (fish, rods, etc.)
#[derive(Resource, Default)]
pub struct FishingAtlas {
    pub loaded: bool,
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum FadePhase {
    #[default]
    /// Not fading — instant switch mode.
    Idle,
    /// Fading out the current track. `timer` counts up from 0 to FADE_DURATION.
    FadingOut { timer: f32, pending_track: String },
    /// Fading in the new track. `timer` counts up from 0 to FADE_DURATION.
    FadingIn { timer: f32 },
}

#[derive(Resource, Debug, Clone, Default)]
pub struct MusicFade {
    pub phase: FadePhase,
}

/// UI-local resource tracking cursor and computed entries.
#[derive(Resource, Default)]
pub struct BuildingUpgradeMenuState {
    pub cursor: usize,
    entries: Vec<UpgradeEntry>,
    pub status_message: String,
    status_timer: f32,
}

/// A single entry in the upgrade menu.
#[derive(Resource, Clone, Debug)]
pub struct UpgradeEntry {
    building: BuildingKind,
    label: &'static str,
    from_tier: BuildingTier,
    to_tier: Option<BuildingTier>,
    cost_gold: u32,
    cost_materials: Vec<(&'static str, u8)>,
    /// false if already at max tier, or an upgrade is currently in progress
    available: bool,
    status_line: String,
}

#[derive(Resource, Default)]
pub struct EnemyAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

/// Tracks enemy attack cooldown.
#[derive(Resource, Debug)]
pub struct EnemyAttackCooldown {
    pub timer: Timer,
}

#[derive(Resource, Debug, Clone)]
pub struct EnemyBlueprint {
    pub x: i32,
    pub y: i32,
    pub kind: MineEnemy,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,
}

/// Drives idle animation for mine enemies. Each enemy gets a random initial
/// phase so they don't all animate in lockstep.
#[derive(Resource, Debug)]
pub struct EnemyIdleAnim {
    pub phase: f32,
}

/// Tracks enemy movement cooldown so they don't move every frame.
#[derive(Resource, Debug)]
pub struct EnemyMoveTick {
    pub timer: Timer,
}

/// Describes a single generated floor before it is spawned into the ECS.
#[derive(Resource, Debug, Clone)]
pub struct FloorBlueprint {
    #[allow(dead_code)]
    pub floor: u8,
    pub rocks: Vec<RockBlueprint>,
    pub enemies: Vec<EnemyBlueprint>,
    pub ladder_pos: (i32, i32),
    /// If true, the ladder is hidden inside a rock and only revealed when
    /// that rock is destroyed (or all rocks are destroyed).
    pub ladder_hidden: bool,
    /// Index into `rocks` that contains the hidden ladder (if any).
    #[allow(dead_code)]
    pub ladder_rock_index: Option<usize>,
    /// Player spawn position (near the entrance).
    pub spawn_pos: (i32, i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MineEnemy {
    GreenSlime,
    Bat,
    RockCrab,
}

/// Marker for all entities belonging to the current mine floor.
/// Used for bulk despawning when changing floors or leaving the mine.
#[derive(Resource, Debug)]
pub struct MineFloorEntity;

/// Grid position specifically for mine entities (mirrors GridPosition
/// but we'll just use the shared one).
/// We use this tag to identify which mine grid cell something occupies.
#[derive(Resource, Debug, Clone, Copy)]
pub struct MineGridPos {
    pub x: i32,
    pub y: i32,
}

#[allow(dead_code)]
#[derive(Resource, Debug, Clone)]
pub struct MineMonster {
    pub kind: MineEnemy,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub speed: f32,
}

#[derive(Resource, Debug, Clone)]
pub struct RockBlueprint {
    pub x: i32,
    pub y: i32,
    pub health: u8,
    pub drop_item: String,
    pub drop_quantity: u8,
    /// If true, the hidden ladder is under this rock.
    pub has_ladder: bool,
}


// ── HEARTHFIELD FALLBACK PROMOTIONS ──
// from hearthfield/ui/building_upgrade_menu.rs
/// Marker for individual row nodes so we can update highlight colours.
#[derive(Component)]
pub struct BuildingRow {
    pub index: usize,
}

// from hearthfield/ui/building_upgrade_menu.rs
/// Marker for the cost text of each row.
#[allow(dead_code)]
#[derive(Component)]
pub struct BuildingRowCost {
    pub index: usize,
}

// from hearthfield/ui/building_upgrade_menu.rs
/// Marker for the name/status text of each row.
#[allow(dead_code)]
#[derive(Component)]
pub struct BuildingRowText {
    pub index: usize,
}

// from hearthfield/ui/building_upgrade_menu.rs
/// Marker for the status feedback text at the bottom.
#[derive(Component)]
pub struct BuildingUpgradeStatusText;

// from hearthfield/ui/dialogue_box.rs
/// Tracks dialogue state within the UI
#[derive(Resource)]
pub struct DialogueUiState {
    pub npc_id: NpcId,
    pub lines: Vec<String>,
    pub current_line: usize,
    #[allow(dead_code)]
    pub portrait_index: Option<u32>,
    /// How many characters of the current line have been revealed (typewriter).
    pub chars_revealed: usize,
    /// Accumulated fractional characters for smooth typewriter pacing.
    pub char_accumulator: f32,
}

// from hearthfield/headless.rs
/// Resource tracking whether headless telemetry is active.
#[derive(Resource)]
pub struct HeadlessTelemetry {
    pub enabled: bool,
    pub frame: u64,
    /// Only write collision data when the map changes.
    pub last_collision_map: String,
}

// from hearthfield/shared/bounded_types.rs
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Health(pub f32);

// from hearthfield/shared/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySlot {
    pub item_id: ItemId,
    pub quantity: u8,
}

// from hearthfield/fishing/mod.rs
/// The dark background bar for the minigame.
#[derive(Component)]
pub struct MinigameBgBar;

// from hearthfield/fishing/mod.rs
/// The catch bar (green block).
#[derive(Component)]
pub struct MinigameCatchBar;

// from hearthfield/fishing/mod.rs
/// The fish zone (red/orange block).
#[derive(Component)]
pub struct MinigameFishZone;

// from hearthfield/fishing/mod.rs
/// Progress bar background.
#[derive(Component)]
pub struct MinigameProgressBg;

// from hearthfield/fishing/mod.rs
/// The progress bar fill.
#[derive(Component)]
pub struct MinigameProgressFill;

// from hearthfield/fishing/mod.rs
/// Marks the fishing minigame root UI container.
#[derive(Component)]
pub struct MinigameRoot;

// from hearthfield/shared/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestObjective {
    Deliver {
        item_id: ItemId,
        quantity: u8,
        delivered: u8,
    },
    Catch {
        fish_id: String,
        delivered: bool,
    },
    Harvest {
        crop_id: String,
        quantity: u8,
        harvested: u8,
    },
    Mine {
        item_id: ItemId,
        quantity: u8,
        collected: u8,
    },
    Talk {
        npc_name: String,
        talked: bool,
    },
    Slay {
        monster_kind: String,
        quantity: u8,
        slain: u8,
    },
}

// from hearthfield/ui/transitions.rs
/// Resource that drives fade in/out
#[derive(Resource)]
pub struct ScreenFade {
    /// Current opacity 0.0 (transparent) to 1.0 (opaque black)
    pub alpha: f32,
    /// Target opacity
    pub target_alpha: f32,
    /// Speed of fade (alpha units per second)
    pub speed: f32,
    /// Whether a fade is actively running
    pub active: bool,
    /// Seconds to hold at full black before fading back in
    pub hold_timer: f32,
    /// The color treatment for the current fade.
    pub tint: ScreenFadeTint,
    /// Marks that the next map transition came from a load handoff.
    pub pending_save_load_handoff: bool,
}
