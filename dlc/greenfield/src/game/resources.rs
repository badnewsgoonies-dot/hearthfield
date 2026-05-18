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
    pub active: u32,
    pub completed: u32,
}

pub const MAX_ACTIVE_QUESTS: u32 = 8;

pub const QUEST_XP_REWARD: u32 = 50;

#[derive(Resource, Debug, Default)]
pub struct MusicState {
    pub current_layer: u8,
    pub crossfade_progress: u32,
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


// ═══════════════════════════════════════════════════════════════════
// CROP LIFECYCLE — Greenfield's thematic typestate, generated by
// ironclad::game_lifecycle to match Hearthfield's pattern of
// SoilProgression / ToolProgression / AnimalGrowth.
// ═══════════════════════════════════════════════════════════════════

pub mod crop {
    use ironclad::game_lifecycle;

    /// Crop growth typestate.
    /// Tilled soil + a seed produces a CropGrowth at Seed; the daily
    /// tick advances each stage (Seed -> Sprout -> Sapling -> Mature),
    /// and the farmer harvests Mature crops to score gold/XP.
    #[game_lifecycle(Seed -> Sprout -> Sapling -> Mature -> Harvested)]
    pub struct CropGrowth;
}
