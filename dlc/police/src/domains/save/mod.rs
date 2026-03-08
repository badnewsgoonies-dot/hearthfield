use std::collections::HashMap;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::shared::{
    CaseBoard, Economy, EvidenceLocker, GameState, Inventory, LoadRequestEvent, NpcId, NpcRegistry,
    NpcRelationship, PartnerArc, PatrolState, PlayerState, SaveRequestEvent, ScheduleEntry,
    ShiftClock, ShiftEndEvent, Skills, UpdatePhase,
};

#[cfg(not(target_arch = "wasm32"))]
const SAVE_DIR_ENV: &str = "PRECINCT_SAVE_DIR";
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_SAVE_DIR: &str = "saves";
const DEFAULT_SAVE_SLOT: u8 = 0;
const SAVE_SLOT_COUNT: u8 = 3;

pub struct SavePlugin;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Resource, Debug, Clone)]
struct SaveConfig {
    directory: PathBuf,
    current_slot: u8,
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for SaveConfig {
    fn default() -> Self {
        let directory = std::env::var_os(SAVE_DIR_ENV)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_SAVE_DIR));

        Self {
            directory,
            current_slot: DEFAULT_SAVE_SLOT,
        }
    }
}

#[cfg(target_arch = "wasm32")]
#[derive(Resource, Debug, Clone)]
struct SaveConfig {
    current_slot: u8,
}

#[cfg(target_arch = "wasm32")]
impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            current_slot: DEFAULT_SAVE_SLOT,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FullSaveData {
    shift_clock: ShiftClock,
    player_state: PlayerState,
    inventory: Inventory,
    case_board: CaseBoard,
    evidence_locker: EvidenceLocker,
    npc_relationships: HashMap<NpcId, NpcRelationship>,
    npc_schedules: HashMap<NpcId, Vec<ScheduleEntry>>,
    partner_arc: PartnerArc,
    economy: Economy,
    skills: Skills,
    patrol_state: PatrolState,
}

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveConfig>()
            .add_systems(Startup, ensure_save_dir)
            .add_systems(
                Update,
                (auto_save, handle_save, handle_load)
                    .chain()
                    .in_set(UpdatePhase::Reactions),
            );
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_save_dir(config: Res<SaveConfig>) {
    if let Err(error) = fs::create_dir_all(&config.directory) {
        bevy::log::error!(
            "Failed to create save directory {}: {error}",
            config.directory.display()
        );
    }
}

#[cfg(target_arch = "wasm32")]
fn ensure_save_dir(_config: Res<SaveConfig>) {}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn handle_save(
    mut save_requests: EventReader<SaveRequestEvent>,
    config: Res<SaveConfig>,
    shift_clock: Res<ShiftClock>,
    player_state: Res<PlayerState>,
    inventory: Res<Inventory>,
    case_board: Res<CaseBoard>,
    evidence_locker: Res<EvidenceLocker>,
    npc_registry: Res<NpcRegistry>,
    partner_arc: Res<PartnerArc>,
    economy: Res<Economy>,
    skills: Res<Skills>,
    patrol_state: Res<PatrolState>,
) {
    if save_requests.read().next().is_none() {
        return;
    }

    let save_data = FullSaveData {
        shift_clock: shift_clock.clone(),
        player_state: player_state.clone(),
        inventory: inventory.clone(),
        case_board: case_board.clone(),
        evidence_locker: evidence_locker.clone(),
        npc_relationships: npc_registry.relationships.clone(),
        npc_schedules: npc_registry.schedules.clone(),
        partner_arc: partner_arc.clone(),
        economy: economy.clone(),
        skills: skills.clone(),
        patrol_state: patrol_state.clone(),
    };

    if let Err(error) = write_save_data(&config, &save_data) {
        bevy::log::error!(
            "Failed to write save slot {} in {}: {error}",
            normalized_slot(config.current_slot),
            config.directory.display()
        );
    }
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
fn handle_save(
    mut save_requests: EventReader<SaveRequestEvent>,
    _config: Res<SaveConfig>,
    _shift_clock: Res<ShiftClock>,
    _player_state: Res<PlayerState>,
    _inventory: Res<Inventory>,
    _case_board: Res<CaseBoard>,
    _evidence_locker: Res<EvidenceLocker>,
    _npc_registry: Res<NpcRegistry>,
    _partner_arc: Res<PartnerArc>,
    _economy: Res<Economy>,
    _skills: Res<Skills>,
    _patrol_state: Res<PatrolState>,
) {
    if save_requests.read().next().is_some() {
        bevy::log::warn!("save not supported in browser");
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::too_many_arguments)]
fn handle_load(
    mut load_requests: EventReader<LoadRequestEvent>,
    mut config: ResMut<SaveConfig>,
    mut next_state: ResMut<NextState<GameState>>,
    mut shift_clock: ResMut<ShiftClock>,
    mut player_state: ResMut<PlayerState>,
    mut inventory: ResMut<Inventory>,
    mut case_board: ResMut<CaseBoard>,
    mut evidence_locker: ResMut<EvidenceLocker>,
    mut npc_registry: ResMut<NpcRegistry>,
    mut partner_arc: ResMut<PartnerArc>,
    mut economy: ResMut<Economy>,
    mut skills: ResMut<Skills>,
    mut patrol_state: ResMut<PatrolState>,
) {
    let Some(requested_slot) = load_requests.read().last().map(|event| event.slot) else {
        return;
    };

    config.current_slot = normalized_slot(requested_slot);

    let save_data = match read_save_data(&config) {
        Ok(data) => data,
        Err(error) => {
            bevy::log::error!(
                "Failed to load save slot {} from {}: {error}",
                config.current_slot,
                config.directory.display()
            );
            return;
        }
    };

    *shift_clock = save_data.shift_clock;
    *player_state = save_data.player_state;
    *inventory = save_data.inventory;
    *case_board = save_data.case_board;
    *evidence_locker = save_data.evidence_locker;
    npc_registry.relationships = save_data.npc_relationships;
    npc_registry.schedules = save_data.npc_schedules;
    *partner_arc = save_data.partner_arc;
    *economy = save_data.economy;
    *skills = save_data.skills;
    *patrol_state = save_data.patrol_state;
    next_state.set(GameState::Playing);
}

#[cfg(target_arch = "wasm32")]
#[allow(clippy::too_many_arguments)]
fn handle_load(
    mut load_requests: EventReader<LoadRequestEvent>,
    mut config: ResMut<SaveConfig>,
    _next_state: ResMut<NextState<GameState>>,
    _shift_clock: ResMut<ShiftClock>,
    _player_state: ResMut<PlayerState>,
    _inventory: ResMut<Inventory>,
    _case_board: ResMut<CaseBoard>,
    _evidence_locker: ResMut<EvidenceLocker>,
    _npc_registry: ResMut<NpcRegistry>,
    _partner_arc: ResMut<PartnerArc>,
    _economy: ResMut<Economy>,
    _skills: ResMut<Skills>,
    _patrol_state: ResMut<PatrolState>,
) {
    if let Some(requested_slot) = load_requests.read().last().map(|event| event.slot) {
        config.current_slot = normalized_slot(requested_slot);
        bevy::log::warn!("save not supported in browser");
    }
}

fn auto_save(
    mut shift_end_events: EventReader<ShiftEndEvent>,
    mut save_requests: EventWriter<SaveRequestEvent>,
) {
    if shift_end_events.read().next().is_some() {
        save_requests.send_default();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn write_save_data(config: &SaveConfig, save_data: &FullSaveData) -> Result<(), String> {
    fs::create_dir_all(&config.directory).map_err(|error| error.to_string())?;

    let payload =
        serde_json::to_string_pretty(save_data).map_err(|error| format!("serialize: {error}"))?;
    fs::write(save_path(&config.directory, config.current_slot), payload)
        .map_err(|error| error.to_string())
}

#[cfg(not(target_arch = "wasm32"))]
fn read_save_data(config: &SaveConfig) -> Result<FullSaveData, String> {
    let payload = fs::read_to_string(save_path(&config.directory, config.current_slot))
        .map_err(|error| error.to_string())?;

    serde_json::from_str(&payload).map_err(|error| format!("deserialize: {error}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn save_path(directory: &Path, slot: u8) -> PathBuf {
    directory.join(format!("save_{}.json", normalized_slot(slot)))
}

fn normalized_slot(slot: u8) -> u8 {
    slot % SAVE_SLOT_COUNT
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::{HashMap, HashSet};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use bevy::state::app::StatesPlugin;

    use crate::shared::{
        ActiveCase, CaseSolvedEvent, DayOfWeek, DispatchCall, DispatchEventKind, Equipment,
        EvidenceCategory, EvidencePiece, EvidenceProcessingState, InventorySlot, PartnerStage,
        Rank, ShiftType, Weather,
    };

    struct TestSaveDir {
        path: PathBuf,
    }

    impl TestSaveDir {
        fn new() -> Self {
            let unique_suffix = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "precinct-save-tests-{}-{unique_suffix}",
                std::process::id()
            ));

            Self { path }
        }
    }

    impl Drop for TestSaveDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn build_test_app(save_dir: &Path) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        );
        app.insert_resource(SaveConfig {
            directory: save_dir.to_path_buf(),
            current_slot: DEFAULT_SAVE_SLOT,
        });
        app.init_resource::<ShiftClock>()
            .init_resource::<PlayerState>()
            .init_resource::<Inventory>()
            .init_resource::<CaseBoard>()
            .init_resource::<EvidenceLocker>()
            .init_resource::<NpcRegistry>()
            .init_resource::<PartnerArc>()
            .init_resource::<Economy>()
            .init_resource::<Skills>()
            .init_resource::<PatrolState>()
            .add_event::<SaveRequestEvent>()
            .add_event::<LoadRequestEvent>()
            .add_event::<ShiftEndEvent>()
            .add_event::<CaseSolvedEvent>()
            .add_plugins(SavePlugin);
        app
    }

    fn seeded_save_data() -> FullSaveData {
        let mut witness_flags = HashSet::new();
        witness_flags.insert("shared_secret".to_string());

        let mut partner_flags = HashSet::new();
        partner_flags.insert("stakeout_complete".to_string());

        let mut active_notes = Vec::new();
        active_notes.push("Dusting for prints in the lot.".to_string());

        let mut witnesses = HashSet::new();
        witnesses.insert("rita_gomez".to_string());

        let mut suspects = HashSet::new();
        suspects.insert("ghost_tipster".to_string());

        FullSaveData {
            shift_clock: ShiftClock {
                shift_number: 7,
                day: 3,
                day_of_week: DayOfWeek::Wednesday,
                hour: 21,
                minute: 45,
                shift_type: ShiftType::Afternoon,
                on_duty: false,
                weather: Weather::Foggy,
                rank: Rank::Detective,
                time_scale: 4.0,
                time_paused: true,
                elapsed_real_seconds: 12.5,
            },
            player_state: PlayerState {
                fatigue: 48.0,
                stress: 31.0,
                gold: 512,
                equipped: vec![Equipment::Badge, Equipment::Notebook, Equipment::Flashlight],
                position_map: crate::shared::MapId::PrecinctExterior,
                position_x: 96.0,
                position_y: 144.0,
            },
            inventory: Inventory {
                evidence_slots: vec![
                    Some(InventorySlot {
                        item_id: "fingerprint_kit".to_string(),
                        quantity: 1,
                    }),
                    None,
                ],
                personal_slots: vec![Some(InventorySlot {
                    item_id: "coffee".to_string(),
                    quantity: 2,
                })],
            },
            case_board: CaseBoard {
                available: vec!["patrol_001_petty_theft".to_string()],
                active: vec![ActiveCase {
                    case_id: "patrol_006_car_breakin".to_string(),
                    status: crate::shared::CaseStatus::EvidenceComplete,
                    evidence_collected: vec![
                        "fingerprint".to_string(),
                        "broken_lock".to_string(),
                        "photo_of_scene".to_string(),
                    ],
                    witnesses_interviewed: witnesses,
                    suspects_interrogated: suspects,
                    shifts_elapsed: 2,
                    notes: active_notes,
                }],
                solved: vec!["patrol_002_vandalism".to_string()],
                cold: vec!["patrol_004_lost_pet".to_string()],
                failed: vec!["patrol_005_shoplifting".to_string()],
                total_cases_solved: 4,
            },
            evidence_locker: EvidenceLocker {
                pieces: vec![
                    EvidencePiece {
                        id: "fingerprint".to_string(),
                        name: "Fingerprint".to_string(),
                        category: EvidenceCategory::Physical,
                        description: "Lifted from the door handle".to_string(),
                        quality: 0.72,
                        linked_case: Some("patrol_006_car_breakin".to_string()),
                        processing_state: EvidenceProcessingState::Analyzed,
                        collected_shift: 6,
                        collected_map: crate::shared::MapId::PrecinctExterior,
                    },
                    EvidencePiece {
                        id: "photo_of_scene".to_string(),
                        name: "Photo of Scene".to_string(),
                        category: EvidenceCategory::Environmental,
                        description: "Parking lot overview".to_string(),
                        quality: 0.64,
                        linked_case: Some("patrol_006_car_breakin".to_string()),
                        processing_state: EvidenceProcessingState::Processing,
                        collected_shift: 6,
                        collected_map: crate::shared::MapId::PrecinctExterior,
                    },
                ],
            },
            npc_relationships: HashMap::from([(
                "det_vasquez".to_string(),
                NpcRelationship {
                    npc_id: "det_vasquez".to_string(),
                    trust: 18,
                    pressure: 4,
                    favors_done: 2,
                    dialogue_flags: witness_flags,
                },
            )]),
            npc_schedules: HashMap::from([(
                "det_vasquez".to_string(),
                vec![
                    ScheduleEntry {
                        hour: 6,
                        map_id: crate::shared::MapId::PrecinctInterior,
                        x: 18.0,
                        y: 18.0,
                    },
                    ScheduleEntry {
                        hour: 12,
                        map_id: crate::shared::MapId::Downtown,
                        x: 22.0,
                        y: 18.0,
                    },
                ],
            )]),
            partner_arc: PartnerArc {
                stage: PartnerStage::WorkingRapport,
                events_triggered: partner_flags,
            },
            economy: Economy {
                reputation: 12,
                department_budget: 680,
                weekly_expenses: 140,
                total_earned: 920,
            },
            skills: Skills {
                total_xp: 245,
                available_points: 2,
                investigation_level: 3,
                interrogation_level: 2,
                patrol_level: 1,
                leadership_level: 0,
            },
            patrol_state: PatrolState {
                fuel: 58.0,
                calls_responded: 5,
                calls_ignored: 1,
                current_dispatch: Some(DispatchCall {
                    kind: DispatchEventKind::TrafficStop,
                    map_id: crate::shared::MapId::PrecinctExterior,
                    description: "Suspicious sedan circling the lot".to_string(),
                    fatigue_cost: 4.0,
                    stress_cost: 2.0,
                    xp_reward: 12,
                    may_generate_evidence: true,
                }),
            },
        }
    }

    fn snapshot_world(app: &mut App) -> serde_json::Value {
        let world = app.world_mut();
        let save_data = FullSaveData {
            shift_clock: world.resource::<ShiftClock>().clone(),
            player_state: world.resource::<PlayerState>().clone(),
            inventory: world.resource::<Inventory>().clone(),
            case_board: world.resource::<CaseBoard>().clone(),
            evidence_locker: world.resource::<EvidenceLocker>().clone(),
            npc_relationships: world.resource::<NpcRegistry>().relationships.clone(),
            npc_schedules: world.resource::<NpcRegistry>().schedules.clone(),
            partner_arc: world.resource::<PartnerArc>().clone(),
            economy: world.resource::<Economy>().clone(),
            skills: world.resource::<Skills>().clone(),
            patrol_state: world.resource::<PatrolState>().clone(),
        };

        serde_json::to_value(save_data).unwrap()
    }

    fn load_seed_data_into_world(app: &mut App, save_data: &FullSaveData) {
        *app.world_mut().resource_mut::<ShiftClock>() = save_data.shift_clock.clone();
        *app.world_mut().resource_mut::<PlayerState>() = save_data.player_state.clone();
        *app.world_mut().resource_mut::<Inventory>() = save_data.inventory.clone();
        *app.world_mut().resource_mut::<CaseBoard>() = save_data.case_board.clone();
        *app.world_mut().resource_mut::<EvidenceLocker>() = save_data.evidence_locker.clone();
        let mut npc_registry = app.world_mut().resource_mut::<NpcRegistry>();
        npc_registry.relationships = save_data.npc_relationships.clone();
        npc_registry.schedules = save_data.npc_schedules.clone();
        *app.world_mut().resource_mut::<PartnerArc>() = save_data.partner_arc.clone();
        *app.world_mut().resource_mut::<Economy>() = save_data.economy.clone();
        *app.world_mut().resource_mut::<Skills>() = save_data.skills.clone();
        *app.world_mut().resource_mut::<PatrolState>() = save_data.patrol_state.clone();
    }

    fn set_current_slot(app: &mut App, slot: u8) {
        app.world_mut().resource_mut::<SaveConfig>().current_slot = slot;
    }

    #[test]
    fn ensure_save_dir_creates_directory() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);

        assert!(!save_dir.path.exists());
        app.update();
        assert!(save_dir.path.exists());
    }

    #[test]
    fn save_request_creates_json_file_on_disk() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        let save_data = seeded_save_data();

        load_seed_data_into_world(&mut app, &save_data);
        set_current_slot(&mut app, 2);

        app.world_mut().send_event(SaveRequestEvent);
        app.update();

        let save_file = save_path(&save_dir.path, 2);
        assert!(save_file.exists());

        let payload = fs::read_to_string(save_file).unwrap();
        assert!(payload.contains("\"shift_clock\""));
        assert!(payload.contains("\"npc_relationships\""));
    }

    #[test]
    fn load_restores_shift_clock_and_targets_playing_state() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        let save_data = seeded_save_data();

        fs::create_dir_all(&save_dir.path).unwrap();
        fs::write(
            save_path(&save_dir.path, 1),
            serde_json::to_string_pretty(&save_data).unwrap(),
        )
        .unwrap();

        app.world_mut().resource_mut::<ShiftClock>().hour = 6;
        app.world_mut().send_event(LoadRequestEvent { slot: 1 });
        app.update();

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.shift_number, 7);
        assert_eq!(clock.hour, 21);
        assert_eq!(clock.minute, 45);
        assert_eq!(app.world().resource::<SaveConfig>().current_slot, 1);
    }

    #[test]
    fn load_restores_player_state() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        let save_data = seeded_save_data();

        fs::create_dir_all(&save_dir.path).unwrap();
        fs::write(
            save_path(&save_dir.path, 0),
            serde_json::to_string_pretty(&save_data).unwrap(),
        )
        .unwrap();

        app.world_mut().resource_mut::<PlayerState>().gold = 0;
        app.world_mut().send_event(LoadRequestEvent { slot: 0 });
        app.update();

        let player_state = app.world().resource::<PlayerState>();
        assert_eq!(player_state.gold, 512);
        assert_eq!(
            player_state.position_map,
            crate::shared::MapId::PrecinctExterior
        );
        assert_eq!(player_state.equipped.len(), 3);
    }

    #[test]
    fn load_restores_case_board() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        let save_data = seeded_save_data();

        fs::create_dir_all(&save_dir.path).unwrap();
        fs::write(
            save_path(&save_dir.path, 0),
            serde_json::to_string_pretty(&save_data).unwrap(),
        )
        .unwrap();

        app.world_mut().resource_mut::<CaseBoard>().active.clear();
        app.world_mut().send_event(LoadRequestEvent { slot: 0 });
        app.update();

        let case_board = app.world().resource::<CaseBoard>();
        assert_eq!(case_board.active.len(), 1);
        assert_eq!(case_board.active[0].case_id, "patrol_006_car_breakin");
        assert_eq!(case_board.total_cases_solved, 4);
    }

    #[test]
    fn round_trip_restores_all_serialized_resources() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        let original = seeded_save_data();

        load_seed_data_into_world(&mut app, &original);
        let expected_snapshot = snapshot_world(&mut app);

        app.world_mut().send_event(SaveRequestEvent);
        app.update();

        *app.world_mut().resource_mut::<ShiftClock>() = ShiftClock::default();
        *app.world_mut().resource_mut::<PlayerState>() = PlayerState::default();
        *app.world_mut().resource_mut::<Inventory>() = Inventory::default();
        *app.world_mut().resource_mut::<CaseBoard>() = CaseBoard::default();
        *app.world_mut().resource_mut::<EvidenceLocker>() = EvidenceLocker::default();
        *app.world_mut().resource_mut::<NpcRegistry>() = NpcRegistry::default();
        *app.world_mut().resource_mut::<PartnerArc>() = PartnerArc::default();
        *app.world_mut().resource_mut::<Economy>() = Economy::default();
        *app.world_mut().resource_mut::<Skills>() = Skills::default();
        *app.world_mut().resource_mut::<PatrolState>() = PatrolState::default();

        app.world_mut().send_event(LoadRequestEvent { slot: 0 });
        app.update();

        assert_eq!(snapshot_world(&mut app), expected_snapshot);
    }

    #[test]
    fn auto_save_writes_current_slot_when_shift_ends() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        load_seed_data_into_world(&mut app, &seeded_save_data());
        set_current_slot(&mut app, 2);

        app.world_mut().send_event(ShiftEndEvent {
            shift_number: 9,
            cases_progressed: 1,
            evidence_collected: 2,
            xp_earned: 15,
        });
        app.update();

        assert!(save_path(&save_dir.path, 2).exists());
    }

    #[test]
    fn full_save_data_serializes_all_expected_fields() {
        let json = serde_json::to_value(seeded_save_data()).unwrap();
        let object = json.as_object().unwrap();

        for key in [
            "shift_clock",
            "player_state",
            "inventory",
            "case_board",
            "evidence_locker",
            "npc_relationships",
            "npc_schedules",
            "partner_arc",
            "economy",
            "skills",
            "patrol_state",
        ] {
            assert!(object.contains_key(key), "missing key {key}");
        }
    }

    #[test]
    fn load_replaces_npc_registry_mutable_slices_but_preserves_definitions() {
        let save_dir = TestSaveDir::new();
        let mut app = build_test_app(&save_dir.path);
        let save_data = seeded_save_data();

        fs::create_dir_all(&save_dir.path).unwrap();
        fs::write(
            save_path(&save_dir.path, 0),
            serde_json::to_string_pretty(&save_data).unwrap(),
        )
        .unwrap();

        app.world_mut().resource_mut::<NpcRegistry>().definitions = HashMap::from([(
            "captain_torres".to_string(),
            crate::shared::NpcDef {
                id: "captain_torres".to_string(),
                name: "Captain Maria Torres".to_string(),
                role: crate::shared::NpcRole::Captain,
                default_map: crate::shared::MapId::PrecinctInterior,
                description: "Precinct captain".to_string(),
            },
        )]);
        app.world_mut().send_event(LoadRequestEvent { slot: 0 });
        app.update();

        let registry = app.world().resource::<NpcRegistry>();
        assert!(registry.definitions.contains_key("captain_torres"));
        assert!(registry.relationships.contains_key("det_vasquez"));
        assert!(registry.schedules.contains_key("det_vasquez"));
    }
}
