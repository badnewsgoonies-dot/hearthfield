use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::calendar::festivals::FestivalState;
use crate::crafting::machines::{ProcessingMachine, ProcessingMachineRegistry, SavedMachine};
use crate::economy::blacksmith::ToolUpgradeQueue;
use crate::economy::buildings::BuildingLevels;
use crate::economy::shipping::ShippingBinQuality;
use crate::npcs::schedules::FarmVisitTracker;
use crate::shared::ShippingLog;
use crate::shared::*;
use crate::world::chests::ChestMarker;
use crate::world::CurrentMapId;

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC TYPES
// ═══════════════════════════════════════════════════════════════════════

pub const SAVE_VERSION: u32 = 2;
pub const NUM_SAVE_SLOTS: usize = 3;

/// Info about a save slot shown on the load/save screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlotInfo {
    pub slot: u8,
    pub exists: bool,
    pub day: u8,
    pub season: Season,
    pub year: u32,
    pub gold: u32,
    pub farm_name: String,
    pub play_time_seconds: u64,
    pub save_timestamp: u64,
}

impl Default for SaveSlotInfo {
    fn default() -> Self {
        Self {
            slot: 0,
            exists: false,
            day: 1,
            season: Season::Spring,
            year: 1,
            gold: 0,
            farm_name: String::from("Hearthfield Farm"),
            play_time_seconds: 0,
            save_timestamp: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// EVENTS
// ═══════════════════════════════════════════════════════════════════════

/// Sent by UI (pause menu) to trigger a manual save.
#[derive(Event, Debug, Clone)]
pub struct SaveRequestEvent {
    pub slot: u8,
}

/// Sent by UI to trigger loading a specific slot.
#[derive(Event, Debug, Clone)]
pub struct LoadRequestEvent {
    pub slot: u8,
}

/// Sent by SavePlugin after a save completes (success or failure).
#[derive(Event, Debug, Clone)]
pub struct SaveCompleteEvent {
    pub slot: u8,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Sent by SavePlugin after a load completes.
#[derive(Event, Debug, Clone)]
pub struct LoadCompleteEvent {
    pub slot: u8,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Sent to initialize a new game (clears all state to defaults).
#[derive(Event, Debug, Clone)]
pub struct NewGameEvent {
    pub farm_name: String,
    pub active_slot: u8,
}

// ═══════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Tracks which save slot is currently active.
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveSaveSlot {
    pub slot: u8,
}

/// Cached metadata for all 3 save slots, refreshed on load screen.
#[derive(Resource, Debug, Clone, Default)]
pub struct SaveSlotInfoCache {
    pub slots: Vec<SaveSlotInfo>,
}

/// Save-file metadata accumulated during gameplay.
///
/// NOTE: `total_gold_earned` and `total_items_shipped` overlap with the same
/// fields on `PlayStats` (shared) and `EconomyStats` (economy). All three
/// resources track independently from separate event readers because they serve
/// distinct consumers:
///
/// - `GameStatistics` feeds save-file header fields read by the slot picker.
/// - `PlayStats` feeds achievement condition checks (`check_achievements`).
/// - `EconomyStats` feeds the year-end evaluation scorer (`handle_evaluation`).
///
/// Consolidating would require cross-domain coupling, so the duplication is
/// intentional.
#[derive(Resource, Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStatistics {
    pub total_gold_earned: u64,
    pub total_items_shipped: u64,
    pub play_time_seconds: u64,
    pub farm_name: String,
}

impl GameStatistics {
    pub fn new(farm_name: impl Into<String>) -> Self {
        Self {
            total_gold_earned: 0,
            total_items_shipped: 0,
            play_time_seconds: 0,
            farm_name: farm_name.into(),
        }
    }
}

/// Accumulated play time from the current session start.
#[derive(Resource, Debug, Clone)]
pub struct SessionTimer {
    pub elapsed: Duration,
}

impl Default for SessionTimer {
    fn default() -> Self {
        Self {
            elapsed: Duration::ZERO,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEM PARAM BUNDLES (to stay within Bevy's 16-param limit)
// ═══════════════════════════════════════════════════════════════════════

/// Read-only bundle of core game-state resources (for saving).
#[derive(SystemParam)]
struct CoreSaveResources<'w> {
    pub calendar: Res<'w, Calendar>,
    pub inventory: Res<'w, Inventory>,
    pub farm_state: Res<'w, FarmState>,
    pub animal_state: Res<'w, AnimalState>,
    pub relationships: Res<'w, Relationships>,
    pub mine_state: Res<'w, MineState>,
    pub unlocked_recipes: Res<'w, UnlockedRecipes>,
    pub shipping_bin: Res<'w, ShippingBin>,
    pub statistics: Res<'w, GameStatistics>,
}

/// Mutable bundle of core game-state resources (for loading).
#[derive(SystemParam)]
struct CoreLoadResources<'w> {
    pub calendar: ResMut<'w, Calendar>,
    pub inventory: ResMut<'w, Inventory>,
    pub farm_state: ResMut<'w, FarmState>,
    pub animal_state: ResMut<'w, AnimalState>,
    pub relationships: ResMut<'w, Relationships>,
    pub mine_state: ResMut<'w, MineState>,
    pub unlocked_recipes: ResMut<'w, UnlockedRecipes>,
    pub shipping_bin: ResMut<'w, ShippingBin>,
    pub statistics: ResMut<'w, GameStatistics>,
}

/// Read-only bundle of extended resources (for saving).
#[derive(SystemParam)]
struct ExtendedResources<'w> {
    pub house_state: Res<'w, HouseState>,
    pub marriage_state: Res<'w, MarriageState>,
    pub quest_log: Res<'w, QuestLog>,
    pub sprinkler_state: Res<'w, SprinklerState>,
    pub active_buffs: Res<'w, ActiveBuffs>,
    pub evaluation_score: Res<'w, EvaluationScore>,
    pub relationship_stages: Res<'w, RelationshipStages>,
    pub achievements: Res<'w, Achievements>,
    pub tutorial_state: Res<'w, TutorialState>,
    pub play_stats: Res<'w, PlayStats>,
    pub building_levels: Res<'w, BuildingLevels>,
    pub shipping_log: Res<'w, ShippingLog>,
    pub fish_encyclopedia: Res<'w, crate::fishing::FishEncyclopedia>,
    pub fishing_skill: Res<'w, crate::fishing::skill::FishingSkill>,
    pub harvest_stats: Res<'w, crate::economy::stats::HarvestStats>,
    pub animal_product_stats: Res<'w, crate::economy::stats::AnimalProductStats>,
    pub economy_stats: Res<'w, crate::economy::gold::EconomyStats>,
    pub daily_talk_tracker: Res<'w, crate::npcs::dialogue::DailyTalkTracker>,
    pub gift_decay_tracker: Res<'w, crate::npcs::map_events::GiftDecayTracker>,
    pub tool_upgrade_queue: Res<'w, ToolUpgradeQueue>,
    pub shipping_bin_quality: Res<'w, ShippingBinQuality>,
    pub festival_state: Res<'w, FestivalState>,
    pub farm_visit_tracker: Res<'w, FarmVisitTracker>,
}

/// Mutable bundle of the extended resources (for loading / new game).
#[derive(SystemParam)]
struct ExtendedResourcesMut<'w> {
    pub house_state: ResMut<'w, HouseState>,
    pub marriage_state: ResMut<'w, MarriageState>,
    pub quest_log: ResMut<'w, QuestLog>,
    pub sprinkler_state: ResMut<'w, SprinklerState>,
    pub active_buffs: ResMut<'w, ActiveBuffs>,
    pub evaluation_score: ResMut<'w, EvaluationScore>,
    pub relationship_stages: ResMut<'w, RelationshipStages>,
    pub achievements: ResMut<'w, Achievements>,
    pub tutorial_state: ResMut<'w, TutorialState>,
    pub play_stats: ResMut<'w, PlayStats>,
    pub building_levels: ResMut<'w, BuildingLevels>,
    pub shipping_log: ResMut<'w, ShippingLog>,
    pub fish_encyclopedia: ResMut<'w, crate::fishing::FishEncyclopedia>,
    pub fishing_skill: ResMut<'w, crate::fishing::skill::FishingSkill>,
    pub harvest_stats: ResMut<'w, crate::economy::stats::HarvestStats>,
    pub animal_product_stats: ResMut<'w, crate::economy::stats::AnimalProductStats>,
    pub economy_stats: ResMut<'w, crate::economy::gold::EconomyStats>,
    pub daily_talk_tracker: ResMut<'w, crate::npcs::dialogue::DailyTalkTracker>,
    pub gift_decay_tracker: ResMut<'w, crate::npcs::map_events::GiftDecayTracker>,
    pub tool_upgrade_queue: ResMut<'w, ToolUpgradeQueue>,
    pub shipping_bin_quality: ResMut<'w, ShippingBinQuality>,
    pub festival_state: ResMut<'w, FestivalState>,
    pub farm_visit_tracker: ResMut<'w, FarmVisitTracker>,
}

/// Chest-related resources needed during load (for restoring chest entities).
#[derive(SystemParam)]
struct ChestLoadResources<'w, 's> {
    pub current_map_id: ResMut<'w, CurrentMapId>,
    pub existing_chests: Query<'w, 's, Entity, With<ChestMarker>>,
    pub chest_sprites: Res<'w, crate::world::chests::ChestSpriteData>,
}

/// Machine-related resources needed during load (for restoring placed machines).
#[derive(SystemParam)]
struct MachineLoadResources<'w, 's> {
    pub machine_registry: ResMut<'w, ProcessingMachineRegistry>,
    pub existing_machines: Query<'w, 's, Entity, With<ProcessingMachine>>,
    pub furniture: Res<'w, crate::world::objects::FurnitureAtlases>,
}

// ═══════════════════════════════════════════════════════════════════════
// PLUGIN
// ═══════════════════════════════════════════════════════════════════════

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ActiveSaveSlot>()
            .init_resource::<SaveSlotInfoCache>()
            .init_resource::<GameStatistics>()
            .init_resource::<SessionTimer>()
            // Events emitted/received by this plugin
            .add_event::<SaveRequestEvent>()
            .add_event::<LoadRequestEvent>()
            .add_event::<SaveCompleteEvent>()
            .add_event::<LoadCompleteEvent>()
            .add_event::<NewGameEvent>()
            // Startup: scan existing save files for the slot cache
            .add_systems(Startup, scan_save_slots)
            // Playing systems — registered individually to stay within Bevy's
            // system-tuple trait bounds (each system has many Res/ResMut params).
            .add_systems(
                Update,
                tick_session_timer.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                track_gold_earned.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                track_items_shipped.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                handle_save_request.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                handle_load_request.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                autosave_on_day_end.run_if(in_state(GameState::Playing)),
            )
            // Also allow saving/loading from the Paused state (pause menu)
            .add_systems(
                Update,
                handle_save_request.run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                Update,
                handle_load_request.run_if(in_state(GameState::Paused)),
            )
            // Allow Main Menu to initialize new game and request save-slot load.
            .add_systems(
                Update,
                handle_load_request.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                Update,
                handle_new_game.run_if(in_state(GameState::MainMenu)),
            )
            // Refresh slot metadata whenever menu is entered.
            .add_systems(OnEnter(GameState::MainMenu), scan_save_slots)
            // Quick-save keybind: F5 in Playing or Paused
            .add_systems(
                Update,
                quicksave_keybind
                    .run_if(in_state(GameState::Playing).or(in_state(GameState::Paused))),
            );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FILESYSTEM HELPERS
// ═══════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
fn saves_directory() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("saves")
}

#[cfg(not(target_arch = "wasm32"))]
fn slot_path(slot: u8) -> PathBuf {
    saves_directory().join(format!("slot_{}.json", slot))
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_saves_dir() -> Result<(), std::io::Error> {
    let dir = saves_directory();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
fn current_timestamp() -> u64 {
    0
}

// ═══════════════════════════════════════════════════════════════════════
// FULL SAVE DATA WITH EXTENDED FIELDS
// ═══════════════════════════════════════════════════════════════════════

/// Wrapper that adds save-metadata around the shared SaveData.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FullSaveFile {
    pub version: u32,
    pub slot: u8,
    pub save_timestamp: u64,
    pub play_time_seconds: u64,
    pub farm_name: String,
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
    #[serde(default)]
    pub house_state: HouseState,
    #[serde(default)]
    pub marriage_state: MarriageState,
    #[serde(default)]
    pub quest_log: QuestLog,
    #[serde(default)]
    pub sprinkler_state: SprinklerState,
    #[serde(default)]
    pub active_buffs: ActiveBuffs,
    #[serde(default)]
    pub evaluation_score: EvaluationScore,
    #[serde(default)]
    pub relationship_stages: RelationshipStages,
    #[serde(default)]
    pub achievements: Achievements,
    #[serde(default)]
    pub tutorial_state: TutorialState,
    #[serde(default)]
    pub play_stats: PlayStats,
    #[serde(default)]
    pub building_levels: BuildingLevels,
    #[serde(default)]
    pub shipping_log: ShippingLog,
    #[serde(default)]
    pub fish_encyclopedia: crate::fishing::FishEncyclopedia,
    #[serde(default)]
    pub fishing_skill: crate::fishing::skill::FishingSkill,
    #[serde(default)]
    pub harvest_stats: crate::economy::stats::HarvestStats,
    #[serde(default)]
    pub animal_product_stats: crate::economy::stats::AnimalProductStats,
    #[serde(default)]
    pub economy_stats: crate::economy::gold::EconomyStats,
    #[serde(default)]
    pub daily_talk_tracker: crate::npcs::dialogue::DailyTalkTracker,
    #[serde(default)]
    pub gift_decay_tracker: crate::npcs::map_events::GiftDecayTracker,
    #[serde(default)]
    pub tool_upgrade_queue: ToolUpgradeQueue,
    #[serde(default)]
    pub shipping_bin_quality: ShippingBinQuality,
    #[serde(default)]
    pub festival_state: FestivalState,
    #[serde(default)]
    pub farm_visit_tracker: FarmVisitTracker,
    /// Storage chest contents placed by the player.
    #[serde(default)]
    pub chests: Vec<StorageChest>,
    /// Processing machines placed by the player.
    #[serde(default)]
    pub placed_machines: Vec<SavedMachine>,
}

impl FullSaveFile {
    fn to_save_slot_info(&self) -> SaveSlotInfo {
        SaveSlotInfo {
            slot: self.slot,
            exists: true,
            day: self.calendar.day,
            season: self.calendar.season,
            year: self.calendar.year,
            gold: self.player_state.gold,
            farm_name: self.farm_name.clone(),
            play_time_seconds: self.play_time_seconds,
            save_timestamp: self.save_timestamp,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SAVE / LOAD LOGIC
// ═══════════════════════════════════════════════════════════════════════

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn write_save(
    slot: u8,
    calendar: &Calendar,
    player_state: &PlayerState,
    inventory: &Inventory,
    farm_state: &FarmState,
    animal_state: &AnimalState,
    relationships: &Relationships,
    mine_state: &MineState,
    unlocked_recipes: &UnlockedRecipes,
    shipping_bin: &ShippingBin,
    statistics: &GameStatistics,
    house_state: &HouseState,
    marriage_state: &MarriageState,
    quest_log: &QuestLog,
    sprinkler_state: &SprinklerState,
    active_buffs: &ActiveBuffs,
    evaluation_score: &EvaluationScore,
    relationship_stages: &RelationshipStages,
    achievements: &Achievements,
    tutorial_state: &TutorialState,
    play_stats: &PlayStats,
    building_levels: &BuildingLevels,
    shipping_log: &ShippingLog,
    fish_encyclopedia: &crate::fishing::FishEncyclopedia,
    fishing_skill: &crate::fishing::skill::FishingSkill,
    harvest_stats: &crate::economy::stats::HarvestStats,
    animal_product_stats: &crate::economy::stats::AnimalProductStats,
    economy_stats: &crate::economy::gold::EconomyStats,
    daily_talk_tracker: &crate::npcs::dialogue::DailyTalkTracker,
    gift_decay_tracker: &crate::npcs::map_events::GiftDecayTracker,
    tool_upgrade_queue: &ToolUpgradeQueue,
    shipping_bin_quality: &ShippingBinQuality,
    festival_state: &FestivalState,
    farm_visit_tracker: &FarmVisitTracker,
    chests: &[StorageChest],
    placed_machines: &[SavedMachine],
) -> Result<(), String> {
    ensure_saves_dir().map_err(|e| format!("Could not create saves directory: {}", e))?;

    let file = FullSaveFile {
        version: SAVE_VERSION,
        slot,
        save_timestamp: current_timestamp(),
        play_time_seconds: statistics.play_time_seconds,
        farm_name: statistics.farm_name.clone(),
        calendar: calendar.clone(),
        player_state: player_state.clone(),
        inventory: inventory.clone(),
        farm_state: farm_state.clone(),
        animal_state: animal_state.clone(),
        relationships: relationships.clone(),
        mine_state: mine_state.clone(),
        unlocked_recipes: unlocked_recipes.clone(),
        shipping_bin: shipping_bin.clone(),
        total_gold_earned: statistics.total_gold_earned,
        total_items_shipped: statistics.total_items_shipped,
        house_state: house_state.clone(),
        marriage_state: marriage_state.clone(),
        quest_log: quest_log.clone(),
        sprinkler_state: sprinkler_state.clone(),
        active_buffs: active_buffs.clone(),
        evaluation_score: evaluation_score.clone(),
        relationship_stages: relationship_stages.clone(),
        achievements: achievements.clone(),
        tutorial_state: tutorial_state.clone(),
        play_stats: play_stats.clone(),
        building_levels: building_levels.clone(),
        shipping_log: shipping_log.clone(),
        fish_encyclopedia: fish_encyclopedia.clone(),
        fishing_skill: fishing_skill.clone(),
        harvest_stats: harvest_stats.clone(),
        animal_product_stats: animal_product_stats.clone(),
        economy_stats: economy_stats.clone(),
        daily_talk_tracker: daily_talk_tracker.clone(),
        gift_decay_tracker: gift_decay_tracker.clone(),
        tool_upgrade_queue: tool_upgrade_queue.clone(),
        shipping_bin_quality: shipping_bin_quality.clone(),
        festival_state: festival_state.clone(),
        farm_visit_tracker: farm_visit_tracker.clone(),
        chests: chests.to_vec(),
        placed_machines: placed_machines.to_vec(),
    };

    let json =
        serde_json::to_string_pretty(&file).map_err(|e| format!("Serialization failed: {}", e))?;

    let path = slot_path(slot);
    // Write to a temp file first, then rename for atomicity
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json)
        .map_err(|e| format!("Write failed for {}: {}", tmp_path.display(), e))?;
    // On Windows, fs::rename fails if the destination already exists.
    // Remove the old file first so the rename succeeds on all platforms.
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Could not remove old save {}: {}", path.display(), e))?;
    }
    fs::rename(&tmp_path, &path).map_err(|e| format!("Rename failed: {}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
fn write_save(
    slot: u8,
    calendar: &Calendar,
    player_state: &PlayerState,
    inventory: &Inventory,
    farm_state: &FarmState,
    animal_state: &AnimalState,
    relationships: &Relationships,
    mine_state: &MineState,
    unlocked_recipes: &UnlockedRecipes,
    shipping_bin: &ShippingBin,
    statistics: &GameStatistics,
    house_state: &HouseState,
    marriage_state: &MarriageState,
    quest_log: &QuestLog,
    sprinkler_state: &SprinklerState,
    active_buffs: &ActiveBuffs,
    evaluation_score: &EvaluationScore,
    relationship_stages: &RelationshipStages,
    achievements: &Achievements,
    tutorial_state: &TutorialState,
    play_stats: &PlayStats,
    building_levels: &BuildingLevels,
    shipping_log: &ShippingLog,
    fish_encyclopedia: &crate::fishing::FishEncyclopedia,
    fishing_skill: &crate::fishing::skill::FishingSkill,
    harvest_stats: &crate::economy::stats::HarvestStats,
    animal_product_stats: &crate::economy::stats::AnimalProductStats,
    economy_stats: &crate::economy::gold::EconomyStats,
    daily_talk_tracker: &crate::npcs::dialogue::DailyTalkTracker,
    gift_decay_tracker: &crate::npcs::map_events::GiftDecayTracker,
    tool_upgrade_queue: &ToolUpgradeQueue,
    shipping_bin_quality: &ShippingBinQuality,
    festival_state: &FestivalState,
    farm_visit_tracker: &FarmVisitTracker,
    chests: &[StorageChest],
    placed_machines: &[SavedMachine],
) -> Result<(), String> {
    let file = FullSaveFile {
        version: SAVE_VERSION,
        slot,
        save_timestamp: current_timestamp(),
        play_time_seconds: statistics.play_time_seconds,
        farm_name: statistics.farm_name.clone(),
        calendar: calendar.clone(),
        player_state: player_state.clone(),
        inventory: inventory.clone(),
        farm_state: farm_state.clone(),
        animal_state: animal_state.clone(),
        relationships: relationships.clone(),
        mine_state: mine_state.clone(),
        unlocked_recipes: unlocked_recipes.clone(),
        shipping_bin: shipping_bin.clone(),
        total_gold_earned: statistics.total_gold_earned,
        total_items_shipped: statistics.total_items_shipped,
        house_state: house_state.clone(),
        marriage_state: marriage_state.clone(),
        quest_log: quest_log.clone(),
        sprinkler_state: sprinkler_state.clone(),
        active_buffs: active_buffs.clone(),
        evaluation_score: evaluation_score.clone(),
        relationship_stages: relationship_stages.clone(),
        achievements: achievements.clone(),
        tutorial_state: tutorial_state.clone(),
        play_stats: play_stats.clone(),
        building_levels: building_levels.clone(),
        shipping_log: shipping_log.clone(),
        fish_encyclopedia: fish_encyclopedia.clone(),
        fishing_skill: fishing_skill.clone(),
        harvest_stats: harvest_stats.clone(),
        animal_product_stats: animal_product_stats.clone(),
        economy_stats: economy_stats.clone(),
        daily_talk_tracker: daily_talk_tracker.clone(),
        gift_decay_tracker: gift_decay_tracker.clone(),
        tool_upgrade_queue: tool_upgrade_queue.clone(),
        shipping_bin_quality: shipping_bin_quality.clone(),
        festival_state: festival_state.clone(),
        farm_visit_tracker: farm_visit_tracker.clone(),
        chests: chests.to_vec(),
        placed_machines: placed_machines.to_vec(),
    };

    let json = serde_json::to_string(&file).map_err(|e| format!("Serialization failed: {}", e))?;

    let key = format!("hearthfield_save_{}", slot);
    let storage = web_sys::window()
        .ok_or_else(|| "No browser window".to_string())?
        .local_storage()
        .map_err(|_| "Failed to access localStorage".to_string())?
        .ok_or_else(|| "localStorage not available".to_string())?;
    storage
        .set_item(&key, &json)
        .map_err(|_| "Failed to write to localStorage".to_string())?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_save(slot: u8) -> Result<FullSaveFile, String> {
    let path = slot_path(slot);
    if !path.exists() {
        return Err(format!("Save slot {} does not exist", slot));
    }
    let json = fs::read_to_string(&path)
        .map_err(|e| format!("Read failed for {}: {}", path.display(), e))?;
    let file: FullSaveFile =
        serde_json::from_str(&json).map_err(|e| format!("Deserialization failed: {}", e))?;

    // Reject saves from future versions (unknown format); allow older saves
    // (serde(default) fills in missing fields).
    if file.version > SAVE_VERSION {
        return Err(format!(
            "Save slot {} uses version {} but this game only supports up to version {}. \
             Please update the game.",
            slot, file.version, SAVE_VERSION
        ));
    }
    if file.version < SAVE_VERSION {
        warn!(
            "Save slot {} has older version {} (current: {}). Loading with defaults for new fields.",
            slot, file.version, SAVE_VERSION
        );
    }

    Ok(file)
}

#[cfg(target_arch = "wasm32")]
fn read_save(slot: u8) -> Result<FullSaveFile, String> {
    let key = format!("hearthfield_save_{}", slot);
    let storage = web_sys::window()
        .ok_or_else(|| "No browser window".to_string())?
        .local_storage()
        .map_err(|_| "Failed to access localStorage".to_string())?
        .ok_or_else(|| "localStorage not available".to_string())?;
    let json = storage
        .get_item(&key)
        .map_err(|_| "Failed to read from localStorage".to_string())?
        .ok_or_else(|| format!("Save slot {} does not exist", slot))?;
    let file: FullSaveFile =
        serde_json::from_str(&json).map_err(|e| format!("Deserialization failed: {}", e))?;

    if file.version > SAVE_VERSION {
        return Err(format!(
            "Save slot {} uses version {} but this game only supports up to version {}. \
             Please update the game.",
            slot, file.version, SAVE_VERSION
        ));
    }
    if file.version < SAVE_VERSION {
        warn!(
            "Save slot {} has older version {} (current: {}). Loading with defaults for new fields.",
            slot, file.version, SAVE_VERSION
        );
    }

    Ok(file)
}

fn peek_save(slot: u8) -> Option<SaveSlotInfo> {
    match read_save(slot) {
        Ok(file) => Some(file.to_save_slot_info()),
        Err(_) => Some(SaveSlotInfo {
            slot,
            exists: false,
            ..Default::default()
        }),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

fn scan_save_slots(mut cache: ResMut<SaveSlotInfoCache>) {
    cache.slots.clear();
    for slot in 0..NUM_SAVE_SLOTS as u8 {
        let info = peek_save(slot).unwrap_or(SaveSlotInfo {
            slot,
            exists: false,
            ..Default::default()
        });
        cache.slots.push(info);
    }
    info!("Save slot scan complete. Found {} slots.", NUM_SAVE_SLOTS);
}

fn tick_session_timer(
    time: Res<Time>,
    mut session: ResMut<SessionTimer>,
    mut stats: ResMut<GameStatistics>,
) {
    session.elapsed += time.delta();
    // Accumulate into statistics every second to keep stats reasonable
    let elapsed_secs = session.elapsed.as_secs();
    if elapsed_secs > 0 {
        stats.play_time_seconds = stats.play_time_seconds.saturating_add(elapsed_secs);
        session.elapsed -= Duration::from_secs(elapsed_secs);
    }
}

fn track_gold_earned(
    mut gold_events: EventReader<GoldChangeEvent>,
    mut stats: ResMut<GameStatistics>,
) {
    for ev in gold_events.read() {
        if ev.amount > 0 {
            stats.total_gold_earned = stats.total_gold_earned.saturating_add(ev.amount as u64);
        }
    }
}

fn track_items_shipped(
    mut shop_events: EventReader<ShopTransactionEvent>,
    mut stats: ResMut<GameStatistics>,
) {
    for ev in shop_events.read() {
        // Count items placed in the shipping bin (is_purchase = false means selling)
        if !ev.is_purchase {
            stats.total_items_shipped =
                stats.total_items_shipped.saturating_add(ev.quantity as u64);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_save_request(
    mut save_events: EventReader<SaveRequestEvent>,
    mut complete_events: EventWriter<SaveCompleteEvent>,
    mut cache: ResMut<SaveSlotInfoCache>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut player_state: ResMut<PlayerState>,
    core: CoreSaveResources,
    ext: ExtendedResources,
    player_grid_q: Query<&GridPosition, With<Player>>,
    chest_query: Query<&StorageChest, With<ChestMarker>>,
    machine_query: Query<(&ProcessingMachine, &GridPosition)>,
) {
    for ev in save_events.read() {
        let slot = ev.slot;
        active_slot.slot = slot;

        // Sync player grid position into PlayerState before serializing
        if let Ok(gp) = player_grid_q.get_single() {
            player_state.save_grid_x = gp.x;
            player_state.save_grid_y = gp.y;
        }

        // Collect all chest contents from ECS entities
        let chests: Vec<StorageChest> = chest_query.iter().cloned().collect();

        // Collect all placed processing machines from ECS entities
        let placed_machines: Vec<SavedMachine> = machine_query
            .iter()
            .map(|(machine, gp)| SavedMachine {
                grid_x: gp.x,
                grid_y: gp.y,
                machine_type: machine.machine_type,
                input_item: machine.input_item.clone(),
                output_item: machine.output_item.clone(),
                processing_time_remaining: machine.processing_time_remaining,
                is_ready: machine.is_ready,
            })
            .collect();

        info!("Saving to slot {}...", slot);

        match write_save(
            slot,
            &core.calendar,
            &player_state,
            &core.inventory,
            &core.farm_state,
            &core.animal_state,
            &core.relationships,
            &core.mine_state,
            &core.unlocked_recipes,
            &core.shipping_bin,
            &core.statistics,
            &ext.house_state,
            &ext.marriage_state,
            &ext.quest_log,
            &ext.sprinkler_state,
            &ext.active_buffs,
            &ext.evaluation_score,
            &ext.relationship_stages,
            &ext.achievements,
            &ext.tutorial_state,
            &ext.play_stats,
            &ext.building_levels,
            &ext.shipping_log,
            &ext.fish_encyclopedia,
            &ext.fishing_skill,
            &ext.harvest_stats,
            &ext.animal_product_stats,
            &ext.economy_stats,
            &ext.daily_talk_tracker,
            &ext.gift_decay_tracker,
            &ext.tool_upgrade_queue,
            &ext.shipping_bin_quality,
            &ext.festival_state,
            &ext.farm_visit_tracker,
            &chests,
            &placed_machines,
        ) {
            Ok(()) => {
                info!("Save to slot {} succeeded.", slot);
                // Refresh the slot info in the cache
                if let Some(info) = peek_save(slot) {
                    if let Some(cached) = cache.slots.get_mut(slot as usize) {
                        *cached = info;
                    }
                }
                complete_events.send(SaveCompleteEvent {
                    slot,
                    success: true,
                    error_message: None,
                });
            }
            Err(e) => {
                warn!("Save to slot {} FAILED: {}", slot, e);
                complete_events.send(SaveCompleteEvent {
                    slot,
                    success: false,
                    error_message: Some(e),
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_load_request(
    mut commands: Commands,
    mut load_events: EventReader<LoadRequestEvent>,
    mut complete_events: EventWriter<LoadCompleteEvent>,
    mut map_events: EventWriter<MapTransitionEvent>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut player_state: ResMut<PlayerState>,
    mut core: CoreLoadResources,
    mut ext: ExtendedResourcesMut,
    mut chests: ChestLoadResources,
    mut machines: MachineLoadResources,
) {
    for ev in load_events.read() {
        let slot = ev.slot;
        info!("Loading from slot {}...", slot);

        match read_save(slot) {
            Ok(file) => {
                active_slot.slot = slot;

                // Apply all loaded state to resources
                *core.calendar = file.calendar;
                // Reset runtime-only flag — time_paused should not persist
                // across sessions.
                core.calendar.time_paused = false;
                *player_state = file.player_state;
                *core.inventory = file.inventory;
                // Clamp selected_slot to valid bounds in case save data is
                // malformed or from a version with a different slot count.
                if core.inventory.selected_slot >= core.inventory.slots.len() {
                    core.inventory.selected_slot = 0;
                }
                *core.farm_state = file.farm_state;
                *core.animal_state = file.animal_state;
                *core.relationships = file.relationships;
                *core.mine_state = file.mine_state;
                *core.unlocked_recipes = file.unlocked_recipes;
                *core.shipping_bin = file.shipping_bin;

                core.statistics.total_gold_earned = file.total_gold_earned;
                core.statistics.total_items_shipped = file.total_items_shipped;
                core.statistics.play_time_seconds = file.play_time_seconds;
                core.statistics.farm_name = file.farm_name;

                *ext.house_state = file.house_state;
                *ext.marriage_state = file.marriage_state;
                *ext.quest_log = file.quest_log;
                *ext.sprinkler_state = file.sprinkler_state;
                *ext.active_buffs = file.active_buffs;
                *ext.evaluation_score = file.evaluation_score;
                *ext.relationship_stages = file.relationship_stages;
                *ext.achievements = file.achievements;
                *ext.tutorial_state = file.tutorial_state;
                *ext.play_stats = file.play_stats;
                *ext.building_levels = file.building_levels;
                *ext.shipping_log = file.shipping_log;
                *ext.fish_encyclopedia = file.fish_encyclopedia;
                *ext.fishing_skill = file.fishing_skill;
                *ext.harvest_stats = file.harvest_stats;
                *ext.animal_product_stats = file.animal_product_stats;
                *ext.economy_stats = file.economy_stats;
                *ext.daily_talk_tracker = file.daily_talk_tracker;
                *ext.gift_decay_tracker = file.gift_decay_tracker;
                *ext.tool_upgrade_queue = file.tool_upgrade_queue;
                *ext.shipping_bin_quality = file.shipping_bin_quality;
                *ext.festival_state = file.festival_state;
                ext.festival_state.restore_runtime_state();
                *ext.farm_visit_tracker = file.farm_visit_tracker;

                // Restore storage chests: despawn any existing chest entities
                // and spawn saved ones.
                for entity in chests.existing_chests.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                for chest in file.chests {
                    let (gx, gy) = chest.grid_pos;
                    let world_x = gx as f32 * TILE_SIZE + TILE_SIZE * 0.5;
                    let world_y = gy as f32 * TILE_SIZE + TILE_SIZE * 0.5;

                    let chest_sprite = if chests.chest_sprites.loaded {
                        let mut s = Sprite::from_atlas_image(
                            chests.chest_sprites.image.clone(),
                            TextureAtlas {
                                layout: chests.chest_sprites.layout.clone(),
                                index: 0,
                            },
                        );
                        s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
                        s
                    } else {
                        Sprite {
                            color: Color::srgb(0.55, 0.35, 0.15),
                            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                            ..default()
                        }
                    };

                    commands.spawn((
                        ChestMarker,
                        chest,
                        chest_sprite,
                        Transform::from_translation(Vec3::new(world_x, world_y, Z_ENTITY_BASE)),
                        LogicalPosition(Vec2::new(world_x, world_y)),
                        YSorted,
                    ));
                }

                // Restore processing machines: despawn existing, spawn from save, rebuild registry.
                for entity in machines.existing_machines.iter() {
                    commands.entity(entity).despawn_recursive();
                }
                machines.machine_registry.machines.clear();
                for saved in file.placed_machines {
                    use crate::crafting::machines::item_to_machine_type;
                    let world_x = saved.grid_x as f32 * TILE_SIZE;
                    let world_y = saved.grid_y as f32 * TILE_SIZE;
                    let machine_sprite = if machines.furniture.machine_anim_layout
                        != Handle::default()
                    {
                        let mut s = Sprite::from_atlas_image(
                            machines.furniture.machine_anim_image.clone(),
                            TextureAtlas {
                                layout: machines.furniture.machine_anim_layout.clone(),
                                index: 0,
                            },
                        );
                        s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
                        s
                    } else if machines.furniture.loaded {
                        let mut s =
                            Sprite::from_image(machines.furniture.processing_machine_image.clone());
                        s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
                        s
                    } else {
                        Sprite::from_color(
                            Color::srgb(0.6, 0.4, 0.2),
                            Vec2::new(TILE_SIZE, TILE_SIZE),
                        )
                    };
                    let display_label = saved.machine_type.display_name().to_string();
                    let item_id = {
                        // Derive the item id from the machine type (reverse of item_to_machine_type).
                        match saved.machine_type {
                            crate::crafting::machines::MachineType::Furnace => "furnace",
                            crate::crafting::machines::MachineType::PreservesJar => "preserves_jar",
                            crate::crafting::machines::MachineType::CheesePress => "cheese_press",
                            crate::crafting::machines::MachineType::Loom => "loom",
                            crate::crafting::machines::MachineType::Keg => "keg",
                            crate::crafting::machines::MachineType::OilMaker => "oil_maker",
                            crate::crafting::machines::MachineType::MayonnaiseMachine => {
                                "mayonnaise_machine"
                            }
                            crate::crafting::machines::MachineType::Tapper => "tapper",
                            crate::crafting::machines::MachineType::BeeHouse => "bee_house",
                            crate::crafting::machines::MachineType::RecyclingMachine => {
                                "recycling_machine"
                            }
                            crate::crafting::machines::MachineType::CrabPot => "crab_pot",
                        }
                    };
                    let mut restored = ProcessingMachine::new(saved.machine_type);
                    restored.input_item = saved.input_item;
                    restored.output_item = saved.output_item;
                    restored.processing_time_remaining = saved.processing_time_remaining;
                    restored.is_ready = saved.is_ready;
                    let entity = commands
                        .spawn((
                            restored,
                            GridPosition::new(saved.grid_x, saved.grid_y),
                            machine_sprite,
                            Transform::from_xyz(world_x, world_y, Z_ENTITY_BASE),
                            LogicalPosition(Vec2::new(world_x, world_y)),
                            YSorted,
                            crate::crafting::machines::MachineAnimTimer::default(),
                            Interactable {
                                kind: InteractionKind::Machine,
                                label: display_label,
                            },
                        ))
                        .id();
                    machines
                        .machine_registry
                        .machines
                        .insert((saved.grid_x, saved.grid_y), entity);
                    let _ = item_id; // item_id available for future use
                    let _ = item_to_machine_type; // suppress unused import warning
                }

                // Force the world to reload the correct map after restoring state.
                // Invalidate CurrentMapId so handle_map_transition doesn't skip
                // the reload when the player was already on this map.
                // Pick a dummy that differs from the saved map.
                chests.current_map_id.map_id = if player_state.current_map == MapId::Mine {
                    MapId::Farm
                } else {
                    MapId::Mine
                };
                let spawn_x = player_state.save_grid_x;
                let spawn_y = player_state.save_grid_y;
                map_events.send(MapTransitionEvent {
                    to_map: player_state.current_map,
                    to_x: spawn_x,
                    to_y: spawn_y,
                });

                info!("Load from slot {} succeeded.", slot);
                complete_events.send(LoadCompleteEvent {
                    slot,
                    success: true,
                    error_message: None,
                });
            }
            Err(e) => {
                warn!("Load from slot {} FAILED: {}", slot, e);
                complete_events.send(LoadCompleteEvent {
                    slot,
                    success: false,
                    error_message: Some(e),
                });
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_new_game(
    mut commands: Commands,
    mut new_game_events: EventReader<NewGameEvent>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut player_state: ResMut<PlayerState>,
    mut core: CoreLoadResources,
    mut ext: ExtendedResourcesMut,
    mut machine_registry: ResMut<ProcessingMachineRegistry>,
    existing_chests: Query<Entity, With<ChestMarker>>,
    existing_machines: Query<Entity, With<ProcessingMachine>>,
) {
    for ev in new_game_events.read() {
        info!(
            "Starting new game in slot {} with farm name '{}'",
            ev.active_slot, ev.farm_name
        );

        active_slot.slot = ev.active_slot;

        // Despawn any player-placed chest entities from a previous session
        for entity in existing_chests.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Despawn any placed machine entities from a previous session
        for entity in existing_machines.iter() {
            commands.entity(entity).despawn_recursive();
        }
        machine_registry.machines.clear();

        // Reset all shared resources to default state
        *core.calendar = Calendar::default();
        *player_state = PlayerState::default();
        *core.inventory = Inventory::default();
        *core.farm_state = FarmState::default();
        *core.animal_state = AnimalState::default();
        *core.relationships = Relationships::default();
        *core.mine_state = MineState::default();
        *core.unlocked_recipes = UnlockedRecipes::default();
        *core.shipping_bin = ShippingBin::default();

        // Reset statistics with new farm name
        *core.statistics = GameStatistics::new(ev.farm_name.clone());

        // Reset extended resources to default state
        *ext.house_state = HouseState::default();
        *ext.marriage_state = MarriageState::default();
        *ext.quest_log = QuestLog::default();
        *ext.sprinkler_state = SprinklerState::default();
        *ext.active_buffs = ActiveBuffs::default();
        *ext.evaluation_score = EvaluationScore::default();
        *ext.relationship_stages = RelationshipStages::default();
        *ext.achievements = Achievements::default();
        *ext.tutorial_state = TutorialState::default();
        *ext.play_stats = PlayStats::default();
        *ext.building_levels = BuildingLevels::default();
        *ext.shipping_log = ShippingLog::default();
        *ext.fish_encyclopedia = crate::fishing::FishEncyclopedia::default();
        *ext.fishing_skill = crate::fishing::skill::FishingSkill::default();
        *ext.harvest_stats = crate::economy::stats::HarvestStats::default();
        *ext.animal_product_stats = crate::economy::stats::AnimalProductStats::default();
        *ext.economy_stats = crate::economy::gold::EconomyStats::default();
        *ext.daily_talk_tracker = crate::npcs::dialogue::DailyTalkTracker::default();
        *ext.gift_decay_tracker = crate::npcs::map_events::GiftDecayTracker::default();
        *ext.tool_upgrade_queue = ToolUpgradeQueue::default();
        *ext.shipping_bin_quality = ShippingBinQuality::default();
        *ext.festival_state = FestivalState::default();
        *ext.farm_visit_tracker = FarmVisitTracker::default();

        // Starter items are granted by grant_starter_items in player/interaction.rs
        // (runs on first frame of Playing state when inventory is empty).

        info!("New game initialized.");
    }
}

/// Listen for DayEndEvent and autosave to the active slot.
fn autosave_on_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut save_writer: EventWriter<SaveRequestEvent>,
    active_slot: Res<ActiveSaveSlot>,
) {
    for ev in day_end_events.read() {
        info!(
            "Autosaving at end of day {} {:?} year {}",
            ev.day, ev.season, ev.year
        );
        save_writer.send(SaveRequestEvent {
            slot: active_slot.slot,
        });
    }
}

/// F5 = quicksave to active slot, F9 = quickload from active slot.
fn quicksave_keybind(
    player_input: Res<PlayerInput>,
    active_slot: Res<ActiveSaveSlot>,
    mut save_writer: EventWriter<SaveRequestEvent>,
    mut load_writer: EventWriter<LoadRequestEvent>,
) {
    if player_input.quicksave {
        info!("F5 quicksave to slot {}", active_slot.slot);
        save_writer.send(SaveRequestEvent {
            slot: active_slot.slot,
        });
    }
    if player_input.quickload {
        info!("F9 quickload from slot {}", active_slot.slot);
        load_writer.send(LoadRequestEvent {
            slot: active_slot.slot,
        });
    }
}
