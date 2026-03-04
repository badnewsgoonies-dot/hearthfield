use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::*;
use crate::aircraft::fuel::FuelWarnings;
use crate::aircraft::maintenance::MaintenanceTracker;
use crate::economy::gold::{GoldMilestones, TransactionLog};
use crate::economy::loans::LoanPortfolio;
use crate::economy::insurance::InsuranceState;
use crate::economy::business::AirlineBusiness;
use crate::player::skills::PilotSkills;
use crate::missions::story::StoryProgress;
use crate::crew::relationships::RelationshipDetails;
use crate::player::logbook::Logbook;
use crate::player::reputation::Reputation;
use crate::economy::market::MarketState;
use crate::missions::special::SpecialMissionState;
use crate::airports::city_exploration::CityState;
use crate::airports::services::AirportServiceState;
use crate::airports::facilities::FacilityState;
use crate::world::AirportStatusMap;

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
    #[serde(default)]
    pub maintenance_tracker: MaintenanceTracker,
    #[serde(default)]
    pub transaction_log: TransactionLog,
    #[serde(default)]
    pub gold_milestones: GoldMilestones,
    #[serde(default)]
    pub fuel_warnings: FuelWarnings,
    #[serde(default)]
    pub pilot_skills: PilotSkills,
    #[serde(default)]
    pub story_progress: StoryProgress,
    #[serde(default)]
    pub loan_portfolio: LoanPortfolio,
    #[serde(default)]
    pub insurance_state: InsuranceState,
    #[serde(default)]
    pub airline_business: AirlineBusiness,
    #[serde(default)]
    pub relationship_details: RelationshipDetails,
    #[serde(default)]
    pub logbook: Logbook,
    #[serde(default)]
    pub reputation: Reputation,
    #[serde(default)]
    pub market_state: MarketState,
    #[serde(default)]
    pub special_mission_state: SpecialMissionState,
    #[serde(default)]
    pub city_state: CityState,
    #[serde(default)]
    pub airport_service_state: AirportServiceState,
    #[serde(default)]
    pub facility_state: FacilityState,
    #[serde(default)]
    pub airport_status_map: AirportStatusMap,
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

// ─── SystemParam bundles (stay within Bevy's 16-param limit) ────────────────

/// Read-only bundle of all game-state resources for saving.
#[derive(SystemParam)]
struct SaveResources<'w> {
    pub calendar: Res<'w, Calendar>,
    pub pilot_state: Res<'w, PilotState>,
    pub fleet: Res<'w, Fleet>,
    pub gold: Res<'w, Gold>,
    pub inventory: Res<'w, Inventory>,
    pub player_location: Res<'w, PlayerLocation>,
    pub relationships: Res<'w, Relationships>,
    pub achievements: Res<'w, Achievements>,
    pub play_stats: Res<'w, PlayStats>,
    pub mission_log: Res<'w, MissionLog>,
    pub mission_board: Res<'w, MissionBoard>,
    pub weather_state: Res<'w, WeatherState>,
    pub economy_stats: Res<'w, EconomyStats>,
    pub tutorial_state: Res<'w, TutorialState>,
    pub maintenance_tracker: Res<'w, MaintenanceTracker>,
}

/// Mutable bundle of all game-state resources for loading.
#[derive(SystemParam)]
struct LoadResources<'w> {
    pub calendar: ResMut<'w, Calendar>,
    pub pilot_state: ResMut<'w, PilotState>,
    pub fleet: ResMut<'w, Fleet>,
    pub gold: ResMut<'w, Gold>,
    pub inventory: ResMut<'w, Inventory>,
    pub player_location: ResMut<'w, PlayerLocation>,
    pub relationships: ResMut<'w, Relationships>,
    pub achievements: ResMut<'w, Achievements>,
    pub play_stats: ResMut<'w, PlayStats>,
    pub mission_log: ResMut<'w, MissionLog>,
    pub mission_board: ResMut<'w, MissionBoard>,
    pub weather_state: ResMut<'w, WeatherState>,
    pub economy_stats: ResMut<'w, EconomyStats>,
    pub tutorial_state: ResMut<'w, TutorialState>,
    pub maintenance_tracker: ResMut<'w, MaintenanceTracker>,
}

#[derive(SystemParam)]
struct SaveResources2<'w> {
    pub transaction_log: Res<'w, TransactionLog>,
    pub gold_milestones: Res<'w, GoldMilestones>,
    pub fuel_warnings: Res<'w, FuelWarnings>,
    pub pilot_skills: Res<'w, PilotSkills>,
    pub story_progress: Res<'w, StoryProgress>,
    pub loan_portfolio: Res<'w, LoanPortfolio>,
    pub insurance_state: Res<'w, InsuranceState>,
    pub airline_business: Res<'w, AirlineBusiness>,
    pub relationship_details: Res<'w, RelationshipDetails>,
}

#[derive(SystemParam)]
struct LoadResources2<'w> {
    pub transaction_log: ResMut<'w, TransactionLog>,
    pub gold_milestones: ResMut<'w, GoldMilestones>,
    pub fuel_warnings: ResMut<'w, FuelWarnings>,
    pub pilot_skills: ResMut<'w, PilotSkills>,
    pub story_progress: ResMut<'w, StoryProgress>,
    pub loan_portfolio: ResMut<'w, LoanPortfolio>,
    pub insurance_state: ResMut<'w, InsuranceState>,
    pub airline_business: ResMut<'w, AirlineBusiness>,
    pub relationship_details: ResMut<'w, RelationshipDetails>,
}

#[derive(SystemParam)]
struct SaveResources3<'w> {
    pub logbook: Res<'w, Logbook>,
    pub reputation: Res<'w, Reputation>,
    pub market_state: Res<'w, MarketState>,
    pub special_mission_state: Res<'w, SpecialMissionState>,
    pub city_state: Res<'w, CityState>,
    pub airport_service_state: Res<'w, AirportServiceState>,
    pub facility_state: Res<'w, FacilityState>,
    pub airport_status_map: Res<'w, AirportStatusMap>,
}

#[derive(SystemParam)]
struct LoadResources3<'w> {
    pub logbook: ResMut<'w, Logbook>,
    pub reputation: ResMut<'w, Reputation>,
    pub market_state: ResMut<'w, MarketState>,
    pub special_mission_state: ResMut<'w, SpecialMissionState>,
    pub city_state: ResMut<'w, CityState>,
    pub airport_service_state: ResMut<'w, AirportServiceState>,
    pub facility_state: ResMut<'w, FacilityState>,
    pub airport_status_map: ResMut<'w, AirportStatusMap>,
}

// ─── Save (two-phase) ───────────────────────────────────────────────────────

/// Phase 1: read events + gather all resource data into `PendingSave`.
fn save_gather(
    mut events: EventReader<SaveRequestEvent>,
    res: SaveResources,
    res2: SaveResources2,
    res3: SaveResources3,
    mut pending: ResMut<PendingSave>,
) {
    for ev in events.read() {
        pending.requests.push((
            ev.slot,
            SaveFile {
                calendar: res.calendar.clone(),
                pilot_state: res.pilot_state.clone(),
                fleet: res.fleet.clone(),
                gold: res.gold.clone(),
                inventory: res.inventory.clone(),
                player_location: res.player_location.clone(),
                relationships: res.relationships.clone(),
                achievements: res.achievements.clone(),
                play_stats: res.play_stats.clone(),
                mission_log: res.mission_log.clone(),
                mission_board: res.mission_board.clone(),
                weather_state: res.weather_state.clone(),
                economy_stats: res.economy_stats.clone(),
                tutorial_state: res.tutorial_state.clone(),
                maintenance_tracker: res.maintenance_tracker.clone(),
                transaction_log: res2.transaction_log.clone(),
                gold_milestones: res2.gold_milestones.clone(),
                fuel_warnings: res2.fuel_warnings.clone(),
                pilot_skills: res2.pilot_skills.clone(),
                story_progress: res2.story_progress.clone(),
                loan_portfolio: res2.loan_portfolio.clone(),
                insurance_state: res2.insurance_state.clone(),
                airline_business: res2.airline_business.clone(),
                relationship_details: res2.relationship_details.clone(),
                logbook: res3.logbook.clone(),
                reputation: res3.reputation.clone(),
                market_state: res3.market_state.clone(),
                special_mission_state: res3.special_mission_state.clone(),
                city_state: res3.city_state.clone(),
                airport_service_state: res3.airport_service_state.clone(),
                facility_state: res3.facility_state.clone(),
                airport_status_map: res3.airport_status_map.clone(),
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
fn load_apply(
    mut pending: ResMut<PendingLoad>,
    mut res: LoadResources,
    mut res2: LoadResources2,
    mut res3: LoadResources3,
    mut zone_ev: EventWriter<ZoneTransitionEvent>,
) {
    for file in pending.files.drain(..) {
        let dest_airport = file.player_location.airport;
        let dest_zone = file.player_location.zone;

        *res.calendar = file.calendar;
        *res.pilot_state = file.pilot_state;
        *res.fleet = file.fleet;
        *res.gold = file.gold;
        *res.inventory = file.inventory;
        *res.player_location = file.player_location;
        *res.relationships = file.relationships;
        *res.achievements = file.achievements;
        *res.play_stats = file.play_stats;
        *res.mission_log = file.mission_log;
        *res.mission_board = file.mission_board;
        *res.weather_state = file.weather_state;
        *res.economy_stats = file.economy_stats;
        *res.tutorial_state = file.tutorial_state;
        *res.maintenance_tracker = file.maintenance_tracker;
        *res2.transaction_log = file.transaction_log;
        *res2.gold_milestones = file.gold_milestones;
        *res2.fuel_warnings = file.fuel_warnings;
        *res2.pilot_skills = file.pilot_skills;
        *res2.story_progress = file.story_progress;
        *res2.loan_portfolio = file.loan_portfolio;
        *res2.insurance_state = file.insurance_state;
        *res2.airline_business = file.airline_business;
        *res2.relationship_details = file.relationship_details;
        *res3.logbook = file.logbook;
        *res3.reputation = file.reputation;
        *res3.market_state = file.market_state;
        *res3.special_mission_state = file.special_mission_state;
        *res3.city_state = file.city_state;
        *res3.airport_service_state = file.airport_service_state;
        *res3.facility_state = file.facility_state;
        *res3.airport_status_map = file.airport_status_map;

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
