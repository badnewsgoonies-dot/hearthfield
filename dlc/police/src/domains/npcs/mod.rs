use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::shared::{
    CaseBoard, DayOfWeek, DialogueEndEvent, DialogueStartEvent, EvidenceCollectedEvent, GameState,
    GridPosition, InterrogationEndEvent, InterrogationStartEvent, MapId, MapTransitionEvent, Npc,
    NpcDef, NpcId, NpcRegistry, NpcRelationship, NpcRole, NpcTrustChangeEvent, PartnerArc,
    PartnerStage, PlayerInput, PlayerState, ScheduleEntry, ShiftClock, UpdatePhase, Weather,
    XpGainedEvent, MAX_PRESSURE, MAX_TRUST, MIN_TRUST, PIXEL_SCALE, TILE_SIZE,
    XP_PER_INTERROGATION,
};

const INTERACTION_RANGE_TILES: i32 = 2;
const NPC_Z: f32 = 6.0;
const WORLD_TILE_SIZE: f32 = TILE_SIZE * PIXEL_SCALE;
const CONFESSION_QUALITY: f32 = 1.0;
const PARTNER_ID: &str = "det_vasquez";

#[derive(Clone, Copy)]
struct AuthoredScheduleEntry {
    hour: u8,
    map_id: MapId,
    x: i32,
    y: i32,
}

#[derive(Clone, Copy)]
struct AuthoredNpc {
    id: &'static str,
    name: &'static str,
    role: NpcRole,
    default_map: MapId,
    description: &'static str,
    schedule: [AuthoredScheduleEntry; 3],
}

#[derive(Resource, Debug, Default, Clone)]
struct NpcInteractionState {
    active_dialogue_npc: Option<NpcId>,
    active_dialogue_context: Option<String>,
    active_interrogation: Option<(NpcId, String)>,
}

const AUTHORED_NPCS: [AuthoredNpc; 12] = [
    AuthoredNpc {
        id: "captain_torres",
        name: "Captain Maria Torres",
        role: NpcRole::Captain,
        default_map: MapId::PrecinctInterior,
        description: "Tough but fair precinct captain. 20 years on the force.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::PrecinctInterior,
                x: 4,
                y: 6,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::PrecinctInterior,
                x: 7,
                y: 4,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::PrecinctInterior,
                x: 5,
                y: 6,
            },
        ],
    },
    AuthoredNpc {
        id: "det_vasquez",
        name: "Detective Alex Vasquez",
        role: NpcRole::Partner,
        default_map: MapId::PrecinctInterior,
        description: "Your assigned partner. Skeptical of rookies but loyal once earned.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::PrecinctInterior,
                x: 18,
                y: 18,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::Downtown,
                x: 22,
                y: 18,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::PrecinctInterior,
                x: 20,
                y: 7,
            },
        ],
    },
    AuthoredNpc {
        id: "officer_chen",
        name: "Officer David Chen",
        role: NpcRole::Colleague,
        default_map: MapId::PrecinctInterior,
        description: "Ambitious rival officer. Competent but cuts corners.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::PrecinctInterior,
                x: 16,
                y: 18,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::Downtown,
                x: 28,
                y: 16,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::PrecinctInterior,
                x: 24,
                y: 18,
            },
        ],
    },
    AuthoredNpc {
        id: "sgt_murphy",
        name: "Desk Sergeant Pat Murphy",
        role: NpcRole::Mentor,
        default_map: MapId::PrecinctInterior,
        description: "Veteran desk sergeant. Knows everyone and everything.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::PrecinctInterior,
                x: 16,
                y: 19,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::PrecinctInterior,
                x: 16,
                y: 17,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::PrecinctInterior,
                x: 16,
                y: 20,
            },
        ],
    },
    AuthoredNpc {
        id: "mayor_aldridge",
        name: "Mayor Victoria Aldridge",
        role: NpcRole::Mayor,
        default_map: MapId::CourtHouse,
        description: "The mayor. Politically savvy, concerned about crime stats.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::CourtHouse,
                x: 10,
                y: 8,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::Downtown,
                x: 24,
                y: 12,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::CourtHouse,
                x: 12,
                y: 8,
            },
        ],
    },
    AuthoredNpc {
        id: "dr_okafor",
        name: "Dr. James Okafor",
        role: NpcRole::MedicalExaminer,
        default_map: MapId::Hospital,
        description: "Medical examiner. Meticulous, dry humor, invaluable for forensics.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::Hospital,
                x: 10,
                y: 8,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::PrecinctInterior,
                x: 27,
                y: 12,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::Hospital,
                x: 12,
                y: 9,
            },
        ],
    },
    AuthoredNpc {
        id: "rita_gomez",
        name: "Rita Gomez",
        role: NpcRole::Informant,
        default_map: MapId::Downtown,
        description: "Diner owner. Hears everything, shares selectively.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::Downtown,
                x: 14,
                y: 24,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::Downtown,
                x: 20,
                y: 18,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::ResidentialSouth,
                x: 8,
                y: 12,
            },
        ],
    },
    AuthoredNpc {
        id: "father_brennan",
        name: "Father Michael Brennan",
        role: NpcRole::Priest,
        default_map: MapId::ResidentialNorth,
        description: "Parish priest. Counselor, mediator, keeper of secrets.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::ResidentialNorth,
                x: 12,
                y: 10,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::Hospital,
                x: 8,
                y: 8,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::ResidentialNorth,
                x: 14,
                y: 12,
            },
        ],
    },
    AuthoredNpc {
        id: "ghost_tipster",
        name: "\"Ghost\"",
        role: NpcRole::Tipster,
        default_map: MapId::IndustrialDistrict,
        description: "Anonymous tipster. Never shows face, communicates by dead drops.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::IndustrialDistrict,
                x: 24,
                y: 18,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::Downtown,
                x: 38,
                y: 30,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::IndustrialDistrict,
                x: 28,
                y: 24,
            },
        ],
    },
    AuthoredNpc {
        id: "nadia_park",
        name: "Nadia Park",
        role: NpcRole::Journalist,
        default_map: MapId::Downtown,
        description: "Investigative journalist. Tenacious, follows the story no matter where.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::Downtown,
                x: 18,
                y: 12,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::CourtHouse,
                x: 8,
                y: 10,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::Downtown,
                x: 26,
                y: 14,
            },
        ],
    },
    AuthoredNpc {
        id: "marcus_cole",
        name: "Marcus Cole",
        role: NpcRole::ExCon,
        default_map: MapId::ResidentialSouth,
        description: "Reformed ex-con. Trying to go straight, knows the criminal world.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::ResidentialSouth,
                x: 12,
                y: 16,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::IndustrialDistrict,
                x: 10,
                y: 22,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::ResidentialSouth,
                x: 18,
                y: 28,
            },
        ],
    },
    AuthoredNpc {
        id: "lucia_vega",
        name: "Lucia Vega",
        role: NpcRole::PublicDefender,
        default_map: MapId::CourtHouse,
        description: "Public defender. Sharp, principled, challenges sloppy police work.",
        schedule: [
            AuthoredScheduleEntry {
                hour: 6,
                map_id: MapId::CourtHouse,
                x: 14,
                y: 10,
            },
            AuthoredScheduleEntry {
                hour: 12,
                map_id: MapId::PrecinctInterior,
                x: 6,
                y: 5,
            },
            AuthoredScheduleEntry {
                hour: 18,
                map_id: MapId::CourtHouse,
                x: 16,
                y: 10,
            },
        ],
    },
];

pub struct NpcsPlugin;

impl Plugin for NpcsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NpcInteractionState>()
            .add_systems(Startup, populate_npc_registry)
            .add_systems(OnEnter(GameState::Playing), spawn_npcs_on_enter)
            .add_systems(OnEnter(GameState::MainMenu), cleanup_npcs)
            .add_systems(
                Update,
                (
                    spawn_npcs_for_map,
                    update_npc_schedules,
                    handle_npc_interaction,
                )
                    .chain()
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (handle_dialogue_events, handle_interrogation_events)
                    .in_set(UpdatePhase::Reactions),
            )
            .add_systems(
                Update,
                (apply_trust_pressure, advance_partner_arc)
                    .chain()
                    .in_set(UpdatePhase::Reactions),
            );
    }
}

fn populate_npc_registry(mut registry: ResMut<NpcRegistry>) {
    registry.definitions = authored_definitions();
    registry.relationships = default_relationships();
    registry.schedules = authored_schedules();
}

fn spawn_npcs_on_enter(
    mut commands: Commands,
    registry: Res<NpcRegistry>,
    player_state: Res<PlayerState>,
    clock: Res<ShiftClock>,
    existing_npcs: Query<Entity, With<Npc>>,
) {
    if existing_npcs.iter().next().is_some() {
        return;
    }

    spawn_npcs_for_target_map(
        &mut commands,
        &registry,
        &clock,
        player_state.position_map,
        &HashSet::new(),
    );
}

fn spawn_npcs_for_map(
    mut commands: Commands,
    mut transition_events: EventReader<MapTransitionEvent>,
    registry: Res<NpcRegistry>,
    clock: Res<ShiftClock>,
    existing_npcs: Query<Entity, With<Npc>>,
) {
    let mut target_map = None;

    for event in transition_events.read() {
        target_map = Some(event.to);
    }

    let Some(target_map) = target_map else {
        return;
    };

    for entity in &existing_npcs {
        commands.entity(entity).despawn();
    }

    spawn_npcs_for_target_map(
        &mut commands,
        &registry,
        &clock,
        target_map,
        &HashSet::new(),
    );
}

fn update_npc_schedules(
    mut commands: Commands,
    registry: Res<NpcRegistry>,
    clock: Res<ShiftClock>,
    player_state: Res<PlayerState>,
    mut transition_events: EventReader<MapTransitionEvent>,
    mut npc_query: Query<(Entity, &Npc, &mut GridPosition, &mut Transform)>,
) {
    if transition_events.read().next().is_some() {
        return;
    }

    let mut active_ids = HashSet::new();
    let current_map = player_state.position_map;

    for (entity, npc, mut grid, mut transform) in &mut npc_query {
        let Some(schedule) = registry.schedules.get(&npc.id) else {
            commands.entity(entity).despawn();
            continue;
        };

        let Some(entry) = active_schedule_entry(&npc.id, schedule, &clock) else {
            commands.entity(entity).despawn();
            continue;
        };

        if entry.map_id != current_map {
            commands.entity(entity).despawn();
            continue;
        }

        let grid_position = resolved_grid_position(&npc.id, entry, &clock);
        let world_position = grid_to_world(grid_position);
        grid.x = grid_position.x;
        grid.y = grid_position.y;
        transform.translation.x = world_position.x;
        transform.translation.y = world_position.y;
        active_ids.insert(npc.id.clone());
    }

    spawn_npcs_for_target_map(&mut commands, &registry, &clock, current_map, &active_ids);
}

fn handle_npc_interaction(
    player_input: Res<PlayerInput>,
    player_state: Res<PlayerState>,
    npc_query: Query<(&Npc, &GridPosition)>,
    mut dialogue_events: EventWriter<DialogueStartEvent>,
) {
    if !player_input.interact {
        return;
    }

    let player_grid = player_grid_position(&player_state);
    let Some(npc_id) = nearest_npc(player_grid, &npc_query) else {
        return;
    };

    dialogue_events.send(DialogueStartEvent {
        npc_id,
        context: "npc_interaction".to_string(),
    });
}

fn handle_dialogue_events(
    mut start_events: EventReader<DialogueStartEvent>,
    mut end_events: EventReader<DialogueEndEvent>,
    mut interaction_state: ResMut<NpcInteractionState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut dialogue_started = None;

    for event in start_events.read() {
        dialogue_started = Some((event.npc_id.clone(), event.context.clone()));
    }

    if let Some((npc_id, context)) = dialogue_started {
        interaction_state.active_dialogue_npc = Some(npc_id);
        interaction_state.active_dialogue_context = Some(context);
        next_state.set(GameState::Dialogue);
    }

    if end_events.read().next().is_some() {
        interaction_state.active_dialogue_npc = None;
        interaction_state.active_dialogue_context = None;
        next_state.set(GameState::Playing);
    }
}

fn handle_interrogation_events(
    mut start_events: EventReader<InterrogationStartEvent>,
    mut end_events: EventReader<InterrogationEndEvent>,
    mut interaction_state: ResMut<NpcInteractionState>,
    mut case_board: ResMut<CaseBoard>,
    mut evidence_events: EventWriter<EvidenceCollectedEvent>,
    mut xp_events: EventWriter<XpGainedEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in start_events.read() {
        interaction_state.active_interrogation =
            Some((event.npc_id.clone(), event.case_id.clone()));
        next_state.set(GameState::Interrogation);
    }

    for event in end_events.read() {
        if let Some(active_case) = case_board
            .active
            .iter_mut()
            .find(|active_case| active_case.case_id == event.case_id)
        {
            active_case
                .suspects_interrogated
                .insert(event.npc_id.clone());
            active_case.notes.push(if event.confession {
                format!("{} confessed during interrogation.", event.npc_id)
            } else {
                format!("{} denied involvement during interrogation.", event.npc_id)
            });
        }

        if event.confession {
            evidence_events.send(EvidenceCollectedEvent {
                evidence_id: format!("confession:{}", event.npc_id),
                case_id: event.case_id.clone(),
                quality: CONFESSION_QUALITY,
            });
            xp_events.send(XpGainedEvent {
                amount: XP_PER_INTERROGATION,
                source: format!("interrogation:{}:{}", event.case_id, event.npc_id),
            });
        }

        interaction_state.active_interrogation = None;
        next_state.set(GameState::Playing);
    }
}

fn apply_trust_pressure(
    mut trust_events: EventReader<NpcTrustChangeEvent>,
    mut registry: ResMut<NpcRegistry>,
) {
    for event in trust_events.read() {
        let relationship = registry
            .relationships
            .entry(event.npc_id.clone())
            .or_insert_with(|| NpcRelationship {
                npc_id: event.npc_id.clone(),
                trust: 0,
                pressure: 0,
                favors_done: 0,
                dialogue_flags: HashSet::new(),
            });

        relationship.trust = (relationship.trust + event.trust_delta).clamp(MIN_TRUST, MAX_TRUST);
        relationship.pressure =
            (relationship.pressure + event.pressure_delta).clamp(0, MAX_PRESSURE);
    }
}

fn advance_partner_arc(registry: Res<NpcRegistry>, mut partner_arc: ResMut<PartnerArc>) {
    let Some(relationship) = registry.relationships.get(PARTNER_ID) else {
        return;
    };

    let target_stage = partner_stage_for_trust(relationship.trust);
    if partner_stage_index(target_stage) <= partner_stage_index(partner_arc.stage) {
        return;
    }

    partner_arc.stage = target_stage;
    partner_arc
        .events_triggered
        .insert(format!("partner_stage:{target_stage:?}"));
}

fn cleanup_npcs(mut commands: Commands, npc_query: Query<Entity, With<Npc>>) {
    for entity in &npc_query {
        commands.entity(entity).despawn();
    }
}

fn authored_definitions() -> HashMap<NpcId, NpcDef> {
    AUTHORED_NPCS
        .iter()
        .map(|npc| {
            (
                npc.id.to_string(),
                NpcDef {
                    id: npc.id.to_string(),
                    name: npc.name.to_string(),
                    role: npc.role,
                    default_map: npc.default_map,
                    description: npc.description.to_string(),
                },
            )
        })
        .collect()
}

fn default_relationships() -> HashMap<NpcId, NpcRelationship> {
    AUTHORED_NPCS
        .iter()
        .map(|npc| {
            (
                npc.id.to_string(),
                NpcRelationship {
                    npc_id: npc.id.to_string(),
                    trust: 0,
                    pressure: 0,
                    favors_done: 0,
                    dialogue_flags: HashSet::new(),
                },
            )
        })
        .collect()
}

fn authored_schedules() -> HashMap<NpcId, Vec<ScheduleEntry>> {
    AUTHORED_NPCS
        .iter()
        .map(|npc| {
            (
                npc.id.to_string(),
                npc.schedule
                    .iter()
                    .map(|entry| ScheduleEntry {
                        hour: entry.hour,
                        map_id: entry.map_id,
                        x: entry.x as f32,
                        y: entry.y as f32,
                    })
                    .collect(),
            )
        })
        .collect()
}

fn spawn_npcs_for_target_map(
    commands: &mut Commands,
    registry: &NpcRegistry,
    clock: &ShiftClock,
    target_map: MapId,
    already_active: &HashSet<NpcId>,
) {
    for authored in AUTHORED_NPCS {
        if already_active.contains(authored.id) {
            continue;
        }

        let Some(definition) = registry.definitions.get(authored.id) else {
            continue;
        };
        let Some(schedule) = registry.schedules.get(authored.id) else {
            continue;
        };
        let Some(entry) = active_schedule_entry(authored.id, schedule, clock) else {
            continue;
        };

        if entry.map_id != target_map {
            continue;
        }

        spawn_npc_entity(commands, authored.id, definition, entry, clock);
    }
}

fn spawn_npc_entity(
    commands: &mut Commands,
    npc_id: &str,
    definition: &NpcDef,
    schedule_entry: &ScheduleEntry,
    clock: &ShiftClock,
) {
    let grid_position = resolved_grid_position(npc_id, schedule_entry, clock);
    let world_position = grid_to_world(grid_position);

    commands.spawn((
        Npc {
            id: definition.id.clone(),
        },
        grid_position,
        Sprite::from_color(role_color(definition.role), Vec2::splat(TILE_SIZE)),
        Transform::from_xyz(world_position.x, world_position.y, NPC_Z),
    ));
}

fn nearest_npc(
    player_grid: GridPosition,
    npc_query: &Query<(&Npc, &GridPosition)>,
) -> Option<NpcId> {
    npc_query
        .iter()
        .filter_map(|(npc, grid)| {
            let distance = distance_squared(player_grid, *grid);
            (distance <= INTERACTION_RANGE_TILES * INTERACTION_RANGE_TILES)
                .then_some((npc.id.clone(), distance))
        })
        .min_by_key(|(_, distance)| *distance)
        .map(|(npc_id, _)| npc_id)
}

fn active_schedule_entry<'a>(
    _npc_id: &str,
    schedule: &'a [ScheduleEntry],
    clock: &ShiftClock,
) -> Option<&'a ScheduleEntry> {
    let schedule_index = if clock.hour >= 18 || clock.hour < 6 {
        2
    } else if clock.hour >= 12 {
        1
    } else {
        0
    };

    schedule.get(schedule_index)
}

fn resolved_grid_position(npc_id: &str, entry: &ScheduleEntry, clock: &ShiftClock) -> GridPosition {
    let grid = GridPosition {
        x: entry.x.round() as i32,
        y: entry.y.round() as i32,
    };
    let weather = weather_offset(entry.map_id, clock.weather);
    let weekend = weekend_offset(npc_id, clock.day_of_week, clock.hour);

    GridPosition {
        x: (grid.x + weather.x + weekend.x).max(0),
        y: (grid.y + weather.y + weekend.y).max(0),
    }
}

fn weather_offset(map_id: MapId, weather: Weather) -> IVec2 {
    if !is_exterior_map(map_id) {
        return IVec2::ZERO;
    }

    match weather {
        Weather::Clear => IVec2::ZERO,
        Weather::Rainy => IVec2::new(0, -1),
        Weather::Foggy => IVec2::new(1, 0),
        Weather::Snowy => IVec2::new(-1, -1),
    }
}

fn weekend_offset(npc_id: &str, day_of_week: DayOfWeek, hour: u8) -> IVec2 {
    if !matches!(day_of_week, DayOfWeek::Saturday | DayOfWeek::Sunday) || !(12..18).contains(&hour)
    {
        return IVec2::ZERO;
    }

    match npc_id {
        "rita_gomez" | "nadia_park" => IVec2::new(1, 0),
        "mayor_aldridge" => IVec2::new(-1, 0),
        _ => IVec2::ZERO,
    }
}

fn is_exterior_map(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PrecinctExterior
            | MapId::Downtown
            | MapId::ResidentialNorth
            | MapId::ResidentialSouth
            | MapId::IndustrialDistrict
            | MapId::Highway
            | MapId::ForestPark
            | MapId::CrimeSceneTemplate
    )
}

fn partner_stage_for_trust(trust: i32) -> PartnerStage {
    if trust >= 90 {
        PartnerStage::BestFriends
    } else if trust >= 60 {
        PartnerStage::TrustedPartners
    } else if trust >= 30 {
        PartnerStage::WorkingRapport
    } else if trust >= 10 {
        PartnerStage::UneasyPartners
    } else {
        PartnerStage::Stranger
    }
}

fn partner_stage_index(stage: PartnerStage) -> u8 {
    match stage {
        PartnerStage::Stranger => 0,
        PartnerStage::UneasyPartners => 1,
        PartnerStage::WorkingRapport => 2,
        PartnerStage::TrustedPartners => 3,
        PartnerStage::BestFriends => 4,
    }
}

fn player_grid_position(player_state: &PlayerState) -> GridPosition {
    GridPosition {
        x: (player_state.position_x / WORLD_TILE_SIZE).round() as i32,
        y: (player_state.position_y / WORLD_TILE_SIZE).round() as i32,
    }
}

fn grid_to_world(grid_position: GridPosition) -> Vec2 {
    Vec2::new(
        grid_position.x as f32 * WORLD_TILE_SIZE,
        grid_position.y as f32 * WORLD_TILE_SIZE,
    )
}

fn distance_squared(a: GridPosition, b: GridPosition) -> i32 {
    let delta_x = a.x - b.x;
    let delta_y = a.y - b.y;
    delta_x * delta_x + delta_y * delta_y
}

fn role_color(role: NpcRole) -> Color {
    match role {
        NpcRole::Captain => Color::srgb(0.74, 0.29, 0.26),
        NpcRole::Partner => Color::srgb(0.24, 0.46, 0.82),
        NpcRole::Colleague => Color::srgb(0.28, 0.68, 0.70),
        NpcRole::Mentor => Color::srgb(0.67, 0.56, 0.31),
        NpcRole::Mayor => Color::srgb(0.70, 0.48, 0.18),
        NpcRole::MedicalExaminer => Color::srgb(0.70, 0.70, 0.78),
        NpcRole::Informant => Color::srgb(0.40, 0.66, 0.30),
        NpcRole::Priest => Color::srgb(0.56, 0.56, 0.63),
        NpcRole::Tipster => Color::srgb(0.20, 0.20, 0.26),
        NpcRole::Journalist => Color::srgb(0.86, 0.69, 0.28),
        NpcRole::ExCon => Color::srgb(0.49, 0.34, 0.22),
        NpcRole::PublicDefender => Color::srgb(0.55, 0.30, 0.63),
        NpcRole::Witness => Color::srgb(0.80, 0.80, 0.80),
        NpcRole::Suspect => Color::srgb(0.58, 0.18, 0.18),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;

    use crate::shared::{ActiveCase, CaseStatus};

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
        app.init_resource::<NpcRegistry>();
        app.init_resource::<PartnerArc>();
        app.init_resource::<PlayerState>();
        app.init_resource::<PlayerInput>();
        app.init_resource::<ShiftClock>();
        app.init_resource::<CaseBoard>();
        app.add_event::<DialogueStartEvent>();
        app.add_event::<DialogueEndEvent>();
        app.add_event::<InterrogationStartEvent>();
        app.add_event::<InterrogationEndEvent>();
        app.add_event::<NpcTrustChangeEvent>();
        app.add_event::<EvidenceCollectedEvent>();
        app.add_event::<MapTransitionEvent>();
        app.add_event::<XpGainedEvent>();
        app.add_plugins(NpcsPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    fn set_player_grid(app: &mut App, map_id: MapId, grid_position: GridPosition) {
        let world_position = grid_to_world(grid_position);
        let mut player_state = app.world_mut().resource_mut::<PlayerState>();
        player_state.position_map = map_id;
        player_state.position_x = world_position.x;
        player_state.position_y = world_position.y;
    }

    #[test]
    fn all_twelve_npcs_populate_registry_on_startup() {
        let mut app = build_test_app();
        app.update();

        let registry = app.world().resource::<NpcRegistry>();
        assert_eq!(registry.definitions.len(), 12);
        assert_eq!(registry.relationships.len(), 12);
        assert_eq!(registry.schedules.len(), 12);
        assert_eq!(
            registry
                .schedules
                .values()
                .filter(|entries| entries.len() == 3)
                .count(),
            12
        );
        assert_eq!(
            registry
                .definitions
                .get("ghost_tipster")
                .map(|npc| npc.name.as_str()),
            Some("\"Ghost\"")
        );
    }

    #[test]
    fn trust_change_applies_and_clamps_correctly() {
        let mut app = build_test_app();
        app.update();

        app.world_mut()
            .resource_mut::<Events<NpcTrustChangeEvent>>()
            .send(NpcTrustChangeEvent {
                npc_id: "rita_gomez".to_string(),
                trust_delta: 150,
                pressure_delta: 0,
            });

        app.update();

        let registry = app.world().resource::<NpcRegistry>();
        assert_eq!(registry.relationships["rita_gomez"].trust, MAX_TRUST);
    }

    #[test]
    fn pressure_change_applies_and_clamps_correctly() {
        let mut app = build_test_app();
        app.update();

        app.world_mut()
            .resource_mut::<Events<NpcTrustChangeEvent>>()
            .send(NpcTrustChangeEvent {
                npc_id: "marcus_cole".to_string(),
                trust_delta: 0,
                pressure_delta: 140,
            });
        app.update();
        app.world_mut()
            .resource_mut::<Events<NpcTrustChangeEvent>>()
            .send(NpcTrustChangeEvent {
                npc_id: "marcus_cole".to_string(),
                trust_delta: 0,
                pressure_delta: -200,
            });

        app.update();

        let registry = app.world().resource::<NpcRegistry>();
        assert_eq!(registry.relationships["marcus_cole"].pressure, 0);
    }

    #[test]
    fn partner_arc_advances_at_correct_trust_thresholds() {
        let mut app = build_test_app();
        app.update();

        for (trust, expected_stage) in [
            (10, PartnerStage::UneasyPartners),
            (30, PartnerStage::WorkingRapport),
            (60, PartnerStage::TrustedPartners),
            (90, PartnerStage::BestFriends),
        ] {
            app.world_mut()
                .resource_mut::<NpcRegistry>()
                .relationships
                .get_mut(PARTNER_ID)
                .unwrap()
                .trust = trust;
            app.update();
            assert_eq!(app.world().resource::<PartnerArc>().stage, expected_stage);
        }
    }

    #[test]
    fn npcs_spawn_on_correct_map_based_on_schedule() {
        let mut app = build_test_app();
        app.update();
        set_player_grid(
            &mut app,
            MapId::PrecinctInterior,
            GridPosition { x: 16, y: 20 },
        );

        enter_playing(&mut app);

        let mut npc_query = app.world_mut().query::<&Npc>();
        let spawned: HashSet<_> = npc_query
            .iter(app.world())
            .map(|npc| npc.id.clone())
            .collect();

        assert!(spawned.contains("captain_torres"));
        assert!(spawned.contains("det_vasquez"));
        assert!(spawned.contains("officer_chen"));
        assert!(spawned.contains("sgt_murphy"));
        assert_eq!(spawned.len(), 4);
    }

    #[test]
    fn interrogation_end_with_confession_emits_evidence_collected_event() {
        let mut app = build_test_app();
        app.update();

        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_001_petty_theft".to_string(),
                status: CaseStatus::Interrogating,
                evidence_collected: Vec::new(),
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });

        app.world_mut()
            .resource_mut::<Events<InterrogationEndEvent>>()
            .send(InterrogationEndEvent {
                npc_id: "marcus_cole".to_string(),
                case_id: "patrol_001_petty_theft".to_string(),
                confession: true,
            });

        app.update();

        let case_board = app.world().resource::<CaseBoard>();
        let active_case = case_board
            .active
            .iter()
            .find(|active_case| active_case.case_id == "patrol_001_petty_theft")
            .unwrap();
        assert!(active_case.suspects_interrogated.contains("marcus_cole"));

        let events = app.world().resource::<Events<EvidenceCollectedEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader
            .read(events)
            .map(|event| (event.evidence_id.clone(), event.case_id.clone()))
            .collect();

        assert!(emitted.contains(&(
            "confession:marcus_cole".to_string(),
            "patrol_001_petty_theft".to_string()
        )));
    }

    #[test]
    fn npc_cleanup_removes_entities_on_map_transition() {
        let mut app = build_test_app();
        app.update();
        set_player_grid(
            &mut app,
            MapId::PrecinctInterior,
            GridPosition { x: 16, y: 20 },
        );
        enter_playing(&mut app);

        let mut npc_query = app.world_mut().query_filtered::<Entity, With<Npc>>();
        let existing_entities: Vec<_> = npc_query.iter(app.world()).collect();
        assert!(!existing_entities.is_empty());

        set_player_grid(&mut app, MapId::Downtown, GridPosition { x: 20, y: 18 });
        app.world_mut()
            .resource_mut::<Events<MapTransitionEvent>>()
            .send(MapTransitionEvent {
                from: MapId::PrecinctInterior,
                to: MapId::Downtown,
            });

        app.update();

        for entity in existing_entities {
            assert!(!app.world().entities().contains(entity));
        }
    }
}
