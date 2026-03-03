use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::*;

pub mod autosave;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingSave>()
            .init_resource::<PendingLoad>()
            .init_resource::<autosave::AutosaveConfig>()
            .add_systems(Startup, scan_save_slots)
            .add_systems(Update, save_gather.before(save_write))
            .add_systems(Update, save_write)
            .add_systems(Update, load_read.before(load_apply))
            .add_systems(Update, load_apply)
            .add_systems(
                Update,
                (
                    autosave::autosave_on_day_end.run_if(in_state(GameState::Playing)),
                    autosave::autosave_on_flight_complete,
                ),
            );
    }
}

// ─── SaveFile ────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
pub struct SaveFile {
    #[serde(default)]
    pub calendar: Calendar,
    #[serde(default)]
    pub pilot_state: PilotState,
    #[serde(default)]
    pub fleet: Fleet,
    #[serde(default)]
    pub gold: Gold,
    #[serde(default)]
    pub inventory: Inventory,
    #[serde(default)]
    pub player_location: PlayerLocation,
    #[serde(default)]
    pub relationships: Relationships,
    #[serde(default)]
    pub achievements: Achievements,
    #[serde(default)]
    pub play_stats: PlayStats,
    #[serde(default)]
    pub mission_log: MissionLog,
    #[serde(default)]
    pub mission_board: MissionBoard,
    #[serde(default)]
    pub weather_state: WeatherState,
    #[serde(default)]
    pub economy_stats: EconomyStats,
    #[serde(default)]
    pub tutorial_state: TutorialState,
}

// ─── Staging resources ───────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct PendingSave {
    requests: Vec<(usize, SaveFile)>,
}

#[derive(Resource, Default)]
struct PendingLoad {
    files: Vec<SaveFile>,
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn save_path(slot: usize) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("saves/slot_{slot}.json"))
}

// ─── Save (two-phase) ───────────────────────────────────────────────────────

/// Phase 1: read events + gather all resource data into `PendingSave`.
#[allow(clippy::too_many_arguments)]
fn save_gather(
    mut events: EventReader<SaveRequestEvent>,
    calendar: Res<Calendar>,
    pilot_state: Res<PilotState>,
    fleet: Res<Fleet>,
    gold: Res<Gold>,
    inventory: Res<Inventory>,
    player_location: Res<PlayerLocation>,
    relationships: Res<Relationships>,
    achievements: Res<Achievements>,
    play_stats: Res<PlayStats>,
    mission_log: Res<MissionLog>,
    mission_board: Res<MissionBoard>,
    weather_state: Res<WeatherState>,
    economy_stats: Res<EconomyStats>,
    tutorial_state: Res<TutorialState>,
    mut pending: ResMut<PendingSave>,
) {
    for ev in events.read() {
        pending.requests.push((
            ev.slot,
            SaveFile {
                calendar: calendar.clone(),
                pilot_state: pilot_state.clone(),
                fleet: fleet.clone(),
                gold: gold.clone(),
                inventory: inventory.clone(),
                player_location: player_location.clone(),
                relationships: relationships.clone(),
                achievements: achievements.clone(),
                play_stats: play_stats.clone(),
                mission_log: mission_log.clone(),
                mission_board: mission_board.clone(),
                weather_state: weather_state.clone(),
                economy_stats: economy_stats.clone(),
                tutorial_state: tutorial_state.clone(),
            },
        ));
    }
}

/// Phase 2: serialize, write to disk, update slot metadata.
fn save_write(
    mut pending: ResMut<PendingSave>,
    mut save_slots: ResMut<SaveSlots>,
    mut complete: EventWriter<SaveCompleteEvent>,
) {
    for (slot, file) in pending.requests.drain(..) {
        let Ok(json) = serde_json::to_string_pretty(&file) else {
            warn!("Save: failed to serialize save data");
            continue;
        };

        if std::fs::create_dir_all("saves").is_err() {
            warn!("Save: failed to create saves directory");
            continue;
        }

        let path = save_path(slot);
        if std::fs::write(&path, &json).is_err() {
            warn!("Save: failed to write {:?}", path);
            continue;
        }

        if slot < save_slots.slots.len() {
            save_slots.slots[slot] = Some(SaveSlotInfo {
                slot,
                pilot_name: file.pilot_state.name.clone(),
                rank: file.pilot_state.rank,
                day: file.calendar.day,
                season: file.calendar.season,
                year: file.calendar.year,
                play_time_secs: file.play_stats.total_play_time_secs,
                airport: file.player_location.airport,
            });
        }

        info!("Saved slot {} to {:?}", slot, path);
        complete.send(SaveCompleteEvent);
    }
}

// ─── Load (two-phase) ───────────────────────────────────────────────────────

/// Phase 1: read events, deserialize from disk into `PendingLoad`.
fn load_read(
    mut events: EventReader<LoadRequestEvent>,
    mut pending: ResMut<PendingLoad>,
) {
    for ev in events.read() {
        let path = save_path(ev.slot);
        let Ok(json) = std::fs::read_to_string(&path) else {
            warn!("Load: failed to read {:?}", path);
            continue;
        };
        let Ok(file) = serde_json::from_str::<SaveFile>(&json) else {
            warn!("Load: failed to deserialize {:?}", path);
            continue;
        };
        pending.files.push(file);
    }
}

/// Phase 2: overwrite all resources from loaded data, fire transition event.
#[allow(clippy::too_many_arguments)]
fn load_apply(
    mut pending: ResMut<PendingLoad>,
    mut calendar: ResMut<Calendar>,
    mut pilot_state: ResMut<PilotState>,
    mut fleet: ResMut<Fleet>,
    mut gold: ResMut<Gold>,
    mut inventory: ResMut<Inventory>,
    mut player_location: ResMut<PlayerLocation>,
    mut relationships: ResMut<Relationships>,
    mut achievements: ResMut<Achievements>,
    mut play_stats: ResMut<PlayStats>,
    mut mission_log: ResMut<MissionLog>,
    mut mission_board: ResMut<MissionBoard>,
    mut weather_state: ResMut<WeatherState>,
    mut economy_stats: ResMut<EconomyStats>,
    mut tutorial_state: ResMut<TutorialState>,
    mut zone_ev: EventWriter<ZoneTransitionEvent>,
) {
    for file in pending.files.drain(..) {
        let dest_airport = file.player_location.airport;
        let dest_zone = file.player_location.zone;

        *calendar = file.calendar;
        *pilot_state = file.pilot_state;
        *fleet = file.fleet;
        *gold = file.gold;
        *inventory = file.inventory;
        *player_location = file.player_location;
        *relationships = file.relationships;
        *achievements = file.achievements;
        *play_stats = file.play_stats;
        *mission_log = file.mission_log;
        *mission_board = file.mission_board;
        *weather_state = file.weather_state;
        *economy_stats = file.economy_stats;
        *tutorial_state = file.tutorial_state;

        zone_ev.send(ZoneTransitionEvent {
            to_airport: dest_airport,
            to_zone: dest_zone,
            to_x: 5,
            to_y: 5,
        });

        info!("Load complete");
    }
}

// ─── Startup scan ────────────────────────────────────────────────────────────

fn scan_save_slots(mut save_slots: ResMut<SaveSlots>) {
    for slot in 0..save_slots.slots.len() {
        let path = save_path(slot);
        let Ok(json) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(file) = serde_json::from_str::<SaveFile>(&json) else {
            continue;
        };
        save_slots.slots[slot] = Some(SaveSlotInfo {
            slot,
            pilot_name: file.pilot_state.name,
            rank: file.pilot_state.rank,
            day: file.calendar.day,
            season: file.calendar.season,
            year: file.calendar.year,
            play_time_secs: file.play_stats.total_play_time_secs,
            airport: file.player_location.airport,
        });
    }
}
