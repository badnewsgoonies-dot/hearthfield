use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC TYPES
// ═══════════════════════════════════════════════════════════════════════

pub const SAVE_VERSION: u32 = 1;
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
#[derive(Resource, Debug, Clone)]
pub struct ActiveSaveSlot {
    pub slot: u8,
}

impl Default for ActiveSaveSlot {
    fn default() -> Self {
        Self { slot: 0 }
    }
}

/// Cached metadata for all 3 save slots, refreshed on load screen.
#[derive(Resource, Debug, Clone, Default)]
pub struct SaveSlotInfoCache {
    pub slots: Vec<SaveSlotInfo>,
}

/// Statistics accumulated during gameplay. Persisted in SaveData.
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
            // Playing systems
            .add_systems(
                Update,
                (
                    tick_session_timer,
                    track_gold_earned,
                    track_items_shipped,
                    handle_save_request,
                    handle_load_request,
                    autosave_on_day_end,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            // Also allow saving/loading from the Paused state (pause menu)
            .add_systems(
                Update,
                (handle_save_request, handle_load_request)
                    .run_if(in_state(GameState::Paused)),
            )
            // Allow Main Menu to initialize new game and request save-slot load.
            .add_systems(
                Update,
                (handle_load_request, handle_new_game).run_if(in_state(GameState::MainMenu)),
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
    };

    let json =
        serde_json::to_string_pretty(&file).map_err(|e| format!("Serialization failed: {}", e))?;

    let path = slot_path(slot);
    // Write to a temp file first, then rename for atomicity
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, &json)
        .map_err(|e| format!("Write failed for {}: {}", tmp_path.display(), e))?;
    fs::rename(&tmp_path, &path).map_err(|e| format!("Rename failed: {}", e))?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn write_save(
    _slot: u8,
    _calendar: &Calendar,
    _player_state: &PlayerState,
    _inventory: &Inventory,
    _farm_state: &FarmState,
    _animal_state: &AnimalState,
    _relationships: &Relationships,
    _mine_state: &MineState,
    _unlocked_recipes: &UnlockedRecipes,
    _shipping_bin: &ShippingBin,
    _statistics: &GameStatistics,
) -> Result<(), String> {
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

    // Version check — future versions can add migration here
    if file.version != SAVE_VERSION {
        warn!(
            "Save slot {} has version {} but current version is {}. Attempting to load anyway.",
            slot, file.version, SAVE_VERSION
        );
    }

    Ok(file)
}

#[cfg(target_arch = "wasm32")]
fn read_save(_slot: u8) -> Result<FullSaveFile, String> {
    Err("Saves not available in browser".to_string())
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

fn handle_save_request(
    mut save_events: EventReader<SaveRequestEvent>,
    mut complete_events: EventWriter<SaveCompleteEvent>,
    mut cache: ResMut<SaveSlotInfoCache>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    inventory: Res<Inventory>,
    farm_state: Res<FarmState>,
    animal_state: Res<AnimalState>,
    relationships: Res<Relationships>,
    mine_state: Res<MineState>,
    unlocked_recipes: Res<UnlockedRecipes>,
    shipping_bin: Res<ShippingBin>,
    statistics: Res<GameStatistics>,
) {
    for ev in save_events.read() {
        let slot = ev.slot;
        active_slot.slot = slot;

        info!("Saving to slot {}...", slot);

        match write_save(
            slot,
            &calendar,
            &player_state,
            &inventory,
            &farm_state,
            &animal_state,
            &relationships,
            &mine_state,
            &unlocked_recipes,
            &shipping_bin,
            &statistics,
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

fn handle_load_request(
    mut load_events: EventReader<LoadRequestEvent>,
    mut complete_events: EventWriter<LoadCompleteEvent>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut calendar: ResMut<Calendar>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    mut farm_state: ResMut<FarmState>,
    mut animal_state: ResMut<AnimalState>,
    mut relationships: ResMut<Relationships>,
    mut mine_state: ResMut<MineState>,
    mut unlocked_recipes: ResMut<UnlockedRecipes>,
    mut shipping_bin: ResMut<ShippingBin>,
    mut statistics: ResMut<GameStatistics>,
) {
    for ev in load_events.read() {
        let slot = ev.slot;
        info!("Loading from slot {}...", slot);

        match read_save(slot) {
            Ok(file) => {
                active_slot.slot = slot;

                // Apply all loaded state to resources
                *calendar = file.calendar;
                *player_state = file.player_state;
                *inventory = file.inventory;
                *farm_state = file.farm_state;
                *animal_state = file.animal_state;
                *relationships = file.relationships;
                *mine_state = file.mine_state;
                *unlocked_recipes = file.unlocked_recipes;
                *shipping_bin = file.shipping_bin;

                statistics.total_gold_earned = file.total_gold_earned;
                statistics.total_items_shipped = file.total_items_shipped;
                statistics.play_time_seconds = file.play_time_seconds;
                statistics.farm_name = file.farm_name;

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

fn handle_new_game(
    mut new_game_events: EventReader<NewGameEvent>,
    mut active_slot: ResMut<ActiveSaveSlot>,
    mut calendar: ResMut<Calendar>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    mut farm_state: ResMut<FarmState>,
    mut animal_state: ResMut<AnimalState>,
    mut relationships: ResMut<Relationships>,
    mut mine_state: ResMut<MineState>,
    mut unlocked_recipes: ResMut<UnlockedRecipes>,
    mut shipping_bin: ResMut<ShippingBin>,
    mut statistics: ResMut<GameStatistics>,
) {
    for ev in new_game_events.read() {
        info!(
            "Starting new game in slot {} with farm name '{}'",
            ev.active_slot, ev.farm_name
        );

        active_slot.slot = ev.active_slot;

        // Reset all shared resources to default state
        *calendar = Calendar::default();
        *player_state = PlayerState::default();
        *inventory = Inventory::default();
        *farm_state = FarmState::default();
        *animal_state = AnimalState::default();
        *relationships = Relationships::default();
        *mine_state = MineState::default();
        *unlocked_recipes = UnlockedRecipes::default();
        *shipping_bin = ShippingBin::default();

        // Reset statistics with new farm name
        *statistics = GameStatistics::new(ev.farm_name.clone());

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
