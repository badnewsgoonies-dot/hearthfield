use bevy::{ecs::system::SystemParam, prelude::*};

use crate::domains::cases::CaseCloseRequestedEvent;
use crate::shared::{
    CaseAssignedEvent, CaseBoard, CaseStatus, EvidenceCollectedEvent, EvidenceLocker,
    EvidenceProcessingState, FatigueChangeEvent, GameState, GridPosition, MapId, PatrolState,
    PlayerInput, PlayerState, ShiftClock, StressChangeEvent, ToastEvent, UpdatePhase,
    COFFEE_FATIGUE_RESTORE, COFFEE_STRESS_RELIEF, COFFEE_TIME_COST_MINUTES, MEAL_FATIGUE_RESTORE,
    MEAL_STRESS_RELIEF, MEAL_TIME_COST_MINUTES, PIXEL_SCALE, TILE_SIZE,
};

const INTERACTION_RANGE_TILES: i32 = 2;
const PRECINCT_OBJECT_Z: f32 = 2.0;
const TOAST_DURATION_SECS: f32 = 2.5;
const WORLD_TILE_SIZE: f32 = TILE_SIZE * PIXEL_SCALE;
const LOCKER_POSITION: GridPosition = GridPosition { x: 27, y: 21 };
const EXTERIOR_EVIDENCE_POSITION: GridPosition = GridPosition { x: 18, y: 14 };

const TRAINING_EVIDENCE_PACKAGE: [&str; 30] = [
    "fingerprint",
    "footprint",
    "weapon",
    "clothing_fiber",
    "tool_mark",
    "receipt",
    "letter",
    "phone_record",
    "security_footage",
    "bank_statement",
    "witness_statement",
    "alibi",
    "confession",
    "tip_off",
    "recording_911",
    "blood_sample",
    "dna_match",
    "ballistic_report",
    "toxicology",
    "digital_forensics",
    "photo_of_scene",
    "weather_log",
    "traffic_cam",
    "broken_lock",
    "tire_track",
    "motive_document",
    "opportunity_timeline",
    "behavioral_pattern",
    "financial_motive",
    "relationship_map",
];

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrecinctInteractable {
    CaseBoard,
    EvidenceTerminal,
    CoffeeMachine,
    MealTable,
    CaptainDoor,
    Locker,
    DispatchRadio,
    ExteriorEvidenceScene,
}

#[derive(Component, Debug)]
pub struct PrecinctObject;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrecinctMapLocation(pub MapId);

#[derive(Clone, Copy)]
struct PrecinctInteractableSpec {
    interactable: PrecinctInteractable,
    map_id: MapId,
    grid: GridPosition,
    color: (u8, u8, u8),
}

const PRECINCT_INTERACTABLE_SPECS: [PrecinctInteractableSpec; 8] = [
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::CaseBoard,
        map_id: MapId::PrecinctInterior,
        grid: GridPosition { x: 4, y: 12 },
        color: (0xe2, 0xc3, 0x3c),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::EvidenceTerminal,
        map_id: MapId::PrecinctInterior,
        grid: GridPosition { x: 26, y: 12 },
        color: (0x45, 0xa6, 0x62),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::CoffeeMachine,
        map_id: MapId::PrecinctInterior,
        grid: GridPosition { x: 26, y: 6 },
        color: (0x74, 0x4e, 0x2b),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::MealTable,
        map_id: MapId::PrecinctInterior,
        grid: GridPosition { x: 24, y: 6 },
        color: (0xcf, 0x82, 0x31),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::CaptainDoor,
        map_id: MapId::PrecinctInterior,
        grid: GridPosition { x: 4, y: 6 },
        color: (0xb7, 0x3a, 0x3a),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::Locker,
        map_id: MapId::PrecinctInterior,
        grid: LOCKER_POSITION,
        color: (0x7d, 0x84, 0x95),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::DispatchRadio,
        map_id: MapId::PrecinctInterior,
        grid: GridPosition { x: 16, y: 16 },
        color: (0x4d, 0xc3, 0xd8),
    },
    PrecinctInteractableSpec {
        interactable: PrecinctInteractable::ExteriorEvidenceScene,
        map_id: MapId::PrecinctExterior,
        grid: EXTERIOR_EVIDENCE_POSITION,
        color: (0xd2, 0x9d, 0x2f),
    },
];

pub struct PrecinctPlugin;

#[derive(SystemParam)]
pub struct PrecinctInteractionContext<'w, 's> {
    case_board: Res<'w, CaseBoard>,
    patrol_state: Option<Res<'w, PatrolState>>,
    evidence_locker: ResMut<'w, EvidenceLocker>,
    shift_clock: ResMut<'w, ShiftClock>,
    case_assigned_events: EventWriter<'w, CaseAssignedEvent>,
    case_close_requests: EventWriter<'w, CaseCloseRequestedEvent>,
    evidence_collected_events: EventWriter<'w, EvidenceCollectedEvent>,
    fatigue_events: EventWriter<'w, FatigueChangeEvent>,
    stress_events: EventWriter<'w, StressChangeEvent>,
    toast_events: EventWriter<'w, ToastEvent>,
    #[system_param(ignore)]
    _marker: std::marker::PhantomData<&'s ()>,
}

impl Plugin for PrecinctPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_precinct_objects_on_enter)
            .add_systems(
                OnEnter(GameState::MainMenu),
                cleanup_precinct_objects_on_exit,
            )
            .add_systems(
                Update,
                (sync_precinct_object_visibility, handle_precinct_interaction)
                    .in_set(UpdatePhase::Reactions)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn spawn_precinct_objects_on_enter(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    existing_objects: Query<Entity, With<PrecinctObject>>,
) {
    spawn_objects_if_needed(&mut commands, &existing_objects, player_state.position_map);
}

pub fn handle_precinct_interaction(
    player_input: Res<PlayerInput>,
    player_state: Res<PlayerState>,
    interactables: Query<
        (&PrecinctInteractable, &GridPosition, &PrecinctMapLocation),
        With<PrecinctObject>,
    >,
    mut interaction_context: PrecinctInteractionContext,
) {
    if !player_input.interact {
        return;
    }

    let player_grid = player_grid_position(&player_state);
    let Some(interactable) =
        nearest_interactable(player_grid, player_state.position_map, &interactables)
    else {
        return;
    };

    match interactable {
        PrecinctInteractable::CaseBoard => {
            if let Some(case_id) = interaction_context
                .case_board
                .active
                .iter()
                .find(|active_case| active_case.status == CaseStatus::EvidenceComplete)
                .map(|active_case| active_case.case_id.clone())
            {
                interaction_context
                    .case_close_requests
                    .send(CaseCloseRequestedEvent {
                        case_id: case_id.clone(),
                    });
                interaction_context.toast_events.send(ToastEvent {
                    message: format!("Filed report for case: {case_id}"),
                    duration_secs: TOAST_DURATION_SECS,
                });
            } else if let Some(case_id) = interaction_context.case_board.available.first() {
                interaction_context
                    .case_assigned_events
                    .send(CaseAssignedEvent {
                        case_id: case_id.clone(),
                    });
                interaction_context.toast_events.send(ToastEvent {
                    message: format!("Assigned case: {case_id}"),
                    duration_secs: TOAST_DURATION_SECS,
                });
            } else {
                interaction_context.toast_events.send(ToastEvent {
                    message: "No cases available right now.".to_string(),
                    duration_secs: TOAST_DURATION_SECS,
                });
            }
        }
        PrecinctInteractable::EvidenceTerminal => {
            let started = start_processing_raw_evidence(&mut interaction_context.evidence_locker);
            let message = if started > 0 {
                format!("Evidence lab queued {started} item(s) for processing.")
            } else {
                "No raw evidence is waiting for the lab.".to_string()
            };
            interaction_context.toast_events.send(ToastEvent {
                message,
                duration_secs: TOAST_DURATION_SECS,
            });
        }
        PrecinctInteractable::CoffeeMachine => {
            interaction_context.fatigue_events.send(FatigueChangeEvent {
                delta: COFFEE_FATIGUE_RESTORE,
            });
            interaction_context.stress_events.send(StressChangeEvent {
                delta: -COFFEE_STRESS_RELIEF,
            });
            advance_clock_minutes(
                &mut interaction_context.shift_clock,
                COFFEE_TIME_COST_MINUTES,
            );
            interaction_context.toast_events.send(ToastEvent {
                message: "Coffee break taken.".to_string(),
                duration_secs: TOAST_DURATION_SECS,
            });
        }
        PrecinctInteractable::MealTable => {
            interaction_context.fatigue_events.send(FatigueChangeEvent {
                delta: MEAL_FATIGUE_RESTORE,
            });
            interaction_context.stress_events.send(StressChangeEvent {
                delta: -MEAL_STRESS_RELIEF,
            });
            advance_clock_minutes(&mut interaction_context.shift_clock, MEAL_TIME_COST_MINUTES);
            interaction_context.toast_events.send(ToastEvent {
                message: "Meal break taken.".to_string(),
                duration_secs: TOAST_DURATION_SECS,
            });
        }
        PrecinctInteractable::CaptainDoor => {
            interaction_context.toast_events.send(ToastEvent {
                message: "Captain Torres is busy right now.".to_string(),
                duration_secs: TOAST_DURATION_SECS,
            });
        }
        PrecinctInteractable::Locker => {
            interaction_context.toast_events.send(ToastEvent {
                message: "Locker management is coming in a future wave.".to_string(),
                duration_secs: TOAST_DURATION_SECS,
            });
        }
        PrecinctInteractable::DispatchRadio => {
            let message = interaction_context
                .patrol_state
                .and_then(|state| {
                    state
                        .current_dispatch
                        .as_ref()
                        .map(|call| call.description.clone())
                })
                .unwrap_or_else(|| "Dispatch is quiet right now.".to_string());
            interaction_context.toast_events.send(ToastEvent {
                message,
                duration_secs: TOAST_DURATION_SECS,
            });
        }
        PrecinctInteractable::ExteriorEvidenceScene => {
            let Some(case_id) = interaction_context
                .case_board
                .active
                .first()
                .map(|active_case| active_case.case_id.clone())
            else {
                interaction_context.toast_events.send(ToastEvent {
                    message: "Take a case from the board before tagging the scene.".to_string(),
                    duration_secs: TOAST_DURATION_SECS,
                });
                return;
            };

            for evidence_id in TRAINING_EVIDENCE_PACKAGE {
                interaction_context
                    .evidence_collected_events
                    .send(EvidenceCollectedEvent {
                        evidence_id: evidence_id.to_string(),
                        case_id: case_id.clone(),
                        quality: 1.0,
                    });
            }

            interaction_context.toast_events.send(ToastEvent {
                message: format!("Training scene tagged evidence for case: {case_id}"),
                duration_secs: TOAST_DURATION_SECS,
            });
        }
    }
}

pub fn sync_precinct_object_visibility(
    player_state: Res<PlayerState>,
    mut precinct_objects: Query<(&PrecinctMapLocation, &mut Visibility), With<PrecinctObject>>,
) {
    for (map_location, mut object_visibility) in &mut precinct_objects {
        *object_visibility = if map_location.0 == player_state.position_map {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn cleanup_precinct_objects_on_exit(
    mut commands: Commands,
    precinct_objects: Query<Entity, With<PrecinctObject>>,
) {
    despawn_precinct_objects(&mut commands, &precinct_objects);
}

fn spawn_objects_if_needed(
    commands: &mut Commands,
    existing_objects: &Query<Entity, With<PrecinctObject>>,
    current_map: MapId,
) {
    if existing_objects.iter().next().is_some() {
        return;
    }

    for spec in PRECINCT_INTERACTABLE_SPECS {
        commands.spawn((
            PrecinctObject,
            spec.interactable,
            PrecinctMapLocation(spec.map_id),
            spec.grid,
            if spec.map_id == current_map {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
            Sprite::from_color(
                Color::srgb_u8(spec.color.0, spec.color.1, spec.color.2),
                Vec2::splat(WORLD_TILE_SIZE),
            ),
            Transform::from_xyz(
                spec.grid.x as f32 * WORLD_TILE_SIZE,
                spec.grid.y as f32 * WORLD_TILE_SIZE,
                PRECINCT_OBJECT_Z,
            ),
        ));
    }
}

fn despawn_precinct_objects(
    commands: &mut Commands,
    precinct_objects: &Query<Entity, With<PrecinctObject>>,
) {
    for entity in precinct_objects.iter() {
        commands.entity(entity).despawn();
    }
}

fn nearest_interactable(
    player_grid: GridPosition,
    current_map: MapId,
    interactables: &Query<
        (&PrecinctInteractable, &GridPosition, &PrecinctMapLocation),
        With<PrecinctObject>,
    >,
) -> Option<PrecinctInteractable> {
    let mut best_match = None;
    let mut best_distance_sq = i32::MAX;

    for (interactable, grid, map_location) in interactables.iter() {
        if map_location.0 != current_map {
            continue;
        }

        let distance_sq = distance_squared(player_grid, *grid);
        if distance_sq <= INTERACTION_RANGE_TILES * INTERACTION_RANGE_TILES
            && distance_sq < best_distance_sq
        {
            best_match = Some(*interactable);
            best_distance_sq = distance_sq;
        }
    }

    best_match
}

fn distance_squared(a: GridPosition, b: GridPosition) -> i32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx) + (dy * dy)
}

fn player_grid_position(player_state: &PlayerState) -> GridPosition {
    GridPosition {
        x: (player_state.position_x / WORLD_TILE_SIZE).round() as i32,
        y: (player_state.position_y / WORLD_TILE_SIZE).round() as i32,
    }
}

fn advance_clock_minutes(clock: &mut ShiftClock, minutes: u32) {
    let current_minutes = u32::from(clock.hour) * 60 + u32::from(clock.minute);
    let updated_minutes = (current_minutes + minutes) % (24 * 60);

    clock.hour = (updated_minutes / 60) as u8;
    clock.minute = (updated_minutes % 60) as u8;
}

fn start_processing_raw_evidence(locker: &mut EvidenceLocker) -> usize {
    let mut started = 0;

    for piece in &mut locker.pieces {
        if piece.processing_state == EvidenceProcessingState::Raw {
            piece.processing_state = EvidenceProcessingState::Processing;
            started += 1;
        }
    }

    started
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::domains::cases::CaseCloseRequestedEvent;
    use crate::shared::{
        ActiveCase, EvidenceCategory, EvidenceCollectedEvent, EvidencePiece, Rank, ShiftType,
        Weather,
    };
    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;
    use std::collections::{HashMap, HashSet};

    fn build_test_app() -> App {
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
        app.init_resource::<CaseBoard>()
            .init_resource::<EvidenceLocker>()
            .init_resource::<PlayerInput>()
            .init_resource::<PlayerState>()
            .init_resource::<ShiftClock>()
            .init_resource::<PatrolState>()
            .add_event::<CaseAssignedEvent>()
            .add_event::<CaseCloseRequestedEvent>()
            .add_event::<EvidenceCollectedEvent>()
            .add_event::<FatigueChangeEvent>()
            .add_event::<StressChangeEvent>()
            .add_event::<ToastEvent>()
            .add_plugins(PrecinctPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
    }

    fn set_player_near(app: &mut App, grid: GridPosition) {
        set_player_near_on_map(app, MapId::PrecinctInterior, grid);
    }

    fn set_player_near_on_map(app: &mut App, map_id: MapId, grid: GridPosition) {
        let mut player_state = app.world_mut().resource_mut::<PlayerState>();
        player_state.position_map = map_id;
        player_state.position_x = grid.x as f32 * WORLD_TILE_SIZE;
        player_state.position_y = grid.y as f32 * WORLD_TILE_SIZE;
    }

    fn interact_once(app: &mut App) {
        app.world_mut().resource_mut::<PlayerInput>().interact = true;
        app.update();
        app.world_mut().resource_mut::<PlayerInput>().interact = false;
    }

    #[test]
    fn precinct_objects_spawn_at_specified_grid_positions() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let mut query = app
            .world_mut()
            .query::<(&PrecinctInteractable, &GridPosition, &Sprite)>();

        let objects = query
            .iter(app.world())
            .map(|(interactable, grid, sprite)| (*interactable, *grid, sprite.custom_size))
            .collect::<Vec<_>>();

        assert_eq!(objects.len(), PRECINCT_INTERACTABLE_SPECS.len());

        let positions = objects
            .iter()
            .map(|(interactable, grid, _)| (*interactable, *grid))
            .collect::<HashMap<_, _>>();

        assert_eq!(
            positions
                .get(&PrecinctInteractable::CaseBoard)
                .map(|grid| (grid.x, grid.y)),
            Some((4, 12))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::EvidenceTerminal)
                .map(|grid| (grid.x, grid.y)),
            Some((26, 12))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::CoffeeMachine)
                .map(|grid| (grid.x, grid.y)),
            Some((26, 6))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::MealTable)
                .map(|grid| (grid.x, grid.y)),
            Some((24, 6))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::CaptainDoor)
                .map(|grid| (grid.x, grid.y)),
            Some((4, 6))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::Locker)
                .map(|grid| (grid.x, grid.y)),
            Some((LOCKER_POSITION.x, LOCKER_POSITION.y))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::DispatchRadio)
                .map(|grid| (grid.x, grid.y)),
            Some((16, 16))
        );
        assert_eq!(
            positions
                .get(&PrecinctInteractable::ExteriorEvidenceScene)
                .map(|grid| (grid.x, grid.y)),
            Some((EXTERIOR_EVIDENCE_POSITION.x, EXTERIOR_EVIDENCE_POSITION.y))
        );

        for (_, _, custom_size) in objects {
            assert_eq!(custom_size, Some(Vec2::splat(WORLD_TILE_SIZE)));
        }
    }

    #[test]
    fn coffee_machine_emits_recovery_events_and_advances_clock() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.hour = 8;
            clock.minute = 5;
            clock.shift_number = 1;
            clock.shift_type = ShiftType::Morning;
            clock.rank = Rank::PatrolOfficer;
            clock.weather = Weather::Clear;
        }

        set_player_near(&mut app, GridPosition { x: 26, y: 6 });
        interact_once(&mut app);

        let fatigue_events = app
            .world_mut()
            .resource_mut::<Events<FatigueChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let stress_events = app
            .world_mut()
            .resource_mut::<Events<StressChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(fatigue_events.len(), 1);
        assert_eq!(fatigue_events[0].delta, COFFEE_FATIGUE_RESTORE);
        assert_eq!(stress_events.len(), 1);
        assert_eq!(stress_events[0].delta, -COFFEE_STRESS_RELIEF);

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.hour, 8);
        assert_eq!(clock.minute, 20);
    }

    #[test]
    fn meal_table_emits_recovery_events_and_advances_clock() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.hour = 12;
            clock.minute = 40;
        }

        set_player_near(&mut app, GridPosition { x: 24, y: 6 });
        interact_once(&mut app);

        let fatigue_events = app
            .world_mut()
            .resource_mut::<Events<FatigueChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let stress_events = app
            .world_mut()
            .resource_mut::<Events<StressChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(fatigue_events.len(), 1);
        assert_eq!(fatigue_events[0].delta, MEAL_FATIGUE_RESTORE);
        assert_eq!(stress_events.len(), 1);
        assert_eq!(stress_events[0].delta, -MEAL_STRESS_RELIEF);

        let clock = app.world().resource::<ShiftClock>();
        assert_eq!(clock.hour, 13);
        assert_eq!(clock.minute, 10);
    }

    #[test]
    fn case_board_interaction_emits_first_available_case_assignment() {
        let mut app = build_test_app();
        app.world_mut().resource_mut::<CaseBoard>().available = vec![
            "patrol_001_petty_theft".to_string(),
            "patrol_002_noise_complaint".to_string(),
        ];
        enter_playing(&mut app);

        set_player_near(&mut app, GridPosition { x: 4, y: 12 });
        interact_once(&mut app);

        let case_events = app
            .world_mut()
            .resource_mut::<Events<CaseAssignedEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(case_events.len(), 1);
        assert_eq!(case_events[0].case_id, "patrol_001_petty_theft");
    }

    #[test]
    fn case_board_interaction_requests_case_close_when_case_is_ready() {
        let mut app = build_test_app();
        app.world_mut().resource_mut::<CaseBoard>().available =
            vec!["patrol_002_noise_complaint".to_string()];
        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_001_petty_theft".to_string(),
                status: CaseStatus::EvidenceComplete,
                evidence_collected: vec![
                    "fingerprint".to_string(),
                    "witness_statement".to_string(),
                ],
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });
        enter_playing(&mut app);

        set_player_near(&mut app, GridPosition { x: 4, y: 12 });
        interact_once(&mut app);

        let case_events = app
            .world_mut()
            .resource_mut::<Events<CaseAssignedEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let close_requests = app
            .world_mut()
            .resource_mut::<Events<CaseCloseRequestedEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert!(case_events.is_empty());
        assert_eq!(close_requests.len(), 1);
        assert_eq!(close_requests[0].case_id, "patrol_001_petty_theft");
    }

    #[test]
    fn evidence_terminal_starts_processing_all_raw_evidence() {
        let mut app = build_test_app();
        app.world_mut().resource_mut::<EvidenceLocker>().pieces = vec![
            raw_evidence("fingerprint", MapId::CrimeSceneTemplate),
            raw_evidence("witness_statement", MapId::Downtown),
            analyzed_evidence("dna_match", MapId::Hospital),
        ];
        enter_playing(&mut app);

        set_player_near(&mut app, GridPosition { x: 26, y: 12 });
        interact_once(&mut app);

        let states = app
            .world()
            .resource::<EvidenceLocker>()
            .pieces
            .iter()
            .map(|piece| (piece.id.as_str(), piece.processing_state))
            .collect::<HashMap<_, _>>();

        assert_eq!(
            states.get("fingerprint"),
            Some(&EvidenceProcessingState::Processing)
        );
        assert_eq!(
            states.get("witness_statement"),
            Some(&EvidenceProcessingState::Processing)
        );
        assert_eq!(
            states.get("dna_match"),
            Some(&EvidenceProcessingState::Analyzed)
        );
    }

    #[test]
    fn exterior_evidence_scene_emits_training_package_for_active_case() {
        let mut app = build_test_app();
        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_006_car_breakin".to_string(),
                status: CaseStatus::Active,
                evidence_collected: Vec::new(),
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });
        enter_playing(&mut app);

        set_player_near_on_map(
            &mut app,
            MapId::PrecinctExterior,
            EXTERIOR_EVIDENCE_POSITION,
        );
        interact_once(&mut app);

        let evidence_events = app
            .world_mut()
            .resource_mut::<Events<EvidenceCollectedEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(evidence_events.len(), TRAINING_EVIDENCE_PACKAGE.len());
        assert!(evidence_events
            .iter()
            .all(|event| event.case_id == "patrol_006_car_breakin"));
        assert!(evidence_events
            .iter()
            .any(|event| event.evidence_id == "broken_lock"));
        assert!(evidence_events
            .iter()
            .any(|event| event.evidence_id == "photo_of_scene"));
    }

    #[test]
    fn precinct_objects_hide_outside_interior_and_cleanup_on_main_menu() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let mut query = app
            .world_mut()
            .query_filtered::<(Entity, &PrecinctMapLocation, &Visibility), With<PrecinctObject>>();
        assert_eq!(
            query.iter(app.world()).count(),
            PRECINCT_INTERACTABLE_SPECS.len()
        );

        set_player_near_on_map(
            &mut app,
            MapId::PrecinctExterior,
            GridPosition { x: 12, y: 1 },
        );

        app.update();

        let visible_maps = query
            .iter(app.world())
            .filter(|(_, _, visibility)| **visibility == Visibility::Visible)
            .map(|(_, map_location, _)| map_location.0)
            .collect::<Vec<_>>();
        assert_eq!(visible_maps, vec![MapId::PrecinctExterior]);

        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::MainMenu);
        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<PrecinctObject>>();
        assert_eq!(query.iter(app.world()).count(), 0);
    }

    fn raw_evidence(id: &str, collected_map: MapId) -> EvidencePiece {
        EvidencePiece {
            id: id.to_string(),
            name: id.replace('_', " "),
            category: EvidenceCategory::Physical,
            description: format!("{id} description"),
            quality: 0.5,
            linked_case: Some("test_case".to_string()),
            processing_state: EvidenceProcessingState::Raw,
            collected_shift: 1,
            collected_map,
        }
    }

    fn analyzed_evidence(id: &str, collected_map: MapId) -> EvidencePiece {
        EvidencePiece {
            processing_state: EvidenceProcessingState::Analyzed,
            ..raw_evidence(id, collected_map)
        }
    }
}
