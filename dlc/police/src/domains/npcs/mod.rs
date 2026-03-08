use std::collections::{HashMap, HashSet};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::domains::cases::{case_definition, witness_lines, CaseRegistry};
use crate::shared::{
    CaseAssignedEvent, CaseBoard, DayOfWeek, DialogueEndEvent, DialogueStartEvent,
    EvidenceCollectedEvent, Facing, GameState, GridPosition, InterrogationEndEvent,
    InterrogationStartEvent, MapId, MapTransitionEvent, Npc, NpcDef, NpcId, NpcRegistry,
    NpcRelationship, NpcRole, NpcTrustChangeEvent, PartnerArc, PartnerStage, PlayerInput,
    PlayerState, ScheduleEntry, ShiftClock, ToastEvent, UpdatePhase, Weather, XpGainedEvent,
    MAX_PRESSURE, MAX_TRUST, MIN_TRUST, PIXEL_SCALE, TILE_SIZE, XP_PER_INTERROGATION,
};

const INTERACTION_RANGE_TILES: i32 = 2;
const NPC_Z: f32 = 6.0;
const WORLD_TILE_SIZE: f32 = TILE_SIZE * PIXEL_SCALE;
const CONFESSION_QUALITY: f32 = 1.0;
const PARTNER_ID: &str = "det_vasquez";
const GHOST_ID: &str = "ghost_tipster";
const HIGH_TRUST_THRESHOLD: i32 = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrustBand {
    Low,
    Mid,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TimeBand {
    Morning,
    Afternoon,
    Night,
}

#[derive(Clone, Copy)]
struct DialogueProfile {
    low_trust: &'static str,
    mid_trust: &'static str,
    high_trust: &'static str,
    morning: &'static str,
    afternoon: &'static str,
    night: &'static str,
    casework: &'static str,
    volunteered: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InterrogationBeat {
    BuildTrust,
    ApplyPressure,
    AskEvidence,
    Endgame,
}

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

#[derive(Component, Debug, Clone, Copy)]
struct NpcFacing(Facing);

#[derive(Resource, Debug, Default, Clone)]
struct InvestigationCommentaryState {
    emitted_keys: HashSet<String>,
}

#[derive(Clone, Copy)]
struct CharacterSpriteSheetSpec {
    path: &'static str,
    columns: u32,
    rows: u32,
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
            .init_resource::<InvestigationCommentaryState>()
            .add_systems(Startup, populate_npc_registry)
            .add_systems(OnEnter(GameState::Playing), spawn_npcs_on_enter)
            .add_systems(OnEnter(GameState::MainMenu), cleanup_npcs)
            .add_systems(
                Update,
                handle_dialogue_cancel_input
                    .in_set(UpdatePhase::Intent)
                    .run_if(in_state(GameState::Dialogue)),
            )
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
                (
                    apply_trust_pressure,
                    advance_partner_arc,
                    emit_investigation_commentary,
                    emit_patrol_commentary,
                )
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
    asset_server: Option<Res<AssetServer>>,
    mut atlas_layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
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
        asset_server.as_deref(),
        &mut atlas_layouts,
    );
}

fn spawn_npcs_for_map(
    mut commands: Commands,
    mut transition_events: EventReader<MapTransitionEvent>,
    registry: Res<NpcRegistry>,
    clock: Res<ShiftClock>,
    asset_server: Option<Res<AssetServer>>,
    mut atlas_layouts: Option<ResMut<Assets<TextureAtlasLayout>>>,
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
        asset_server.as_deref(),
        &mut atlas_layouts,
    );
}

fn update_npc_schedules(
    mut commands: Commands,
    registry: Res<NpcRegistry>,
    clock: Res<ShiftClock>,
    player_state: Res<PlayerState>,
    sprite_assets: (
        Option<Res<AssetServer>>,
        Option<ResMut<Assets<TextureAtlasLayout>>>,
    ),
    mut transition_events: EventReader<MapTransitionEvent>,
    mut npc_query: Query<(
        Entity,
        &Npc,
        &mut GridPosition,
        &mut Transform,
        &mut NpcFacing,
        &mut Sprite,
    )>,
) {
    let (asset_server, mut atlas_layouts) = sprite_assets;
    if transition_events.read().next().is_some() {
        return;
    }

    let mut active_ids = HashSet::new();
    let current_map = player_state.position_map;

    for (entity, npc, mut grid, mut transform, mut facing, mut sprite) in &mut npc_query {
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

        let previous_grid = *grid;
        let grid_position = resolved_grid_position(&npc.id, entry, &clock);
        let world_position = grid_to_world(grid_position);
        grid.x = grid_position.x;
        grid.y = grid_position.y;
        transform.translation.x = world_position.x;
        transform.translation.y = world_position.y;
        facing.0 = facing_from_grid_delta(previous_grid, grid_position, facing.0);
        sync_character_sprite(&mut sprite, facing.0);
        active_ids.insert(npc.id.clone());
    }

    spawn_npcs_for_target_map(
        &mut commands,
        &registry,
        &clock,
        current_map,
        &active_ids,
        asset_server.as_deref(),
        &mut atlas_layouts,
    );
}

fn handle_npc_interaction(
    mut interaction: NpcInteractionInput,
    mut interaction_output: NpcInteractionOutput,
) {
    if !interaction.player_input.interact {
        return;
    }

    let player_grid = player_grid_position(&interaction.player_state);
    let Some(npc_id) = nearest_npc(player_grid, &interaction.npc_query) else {
        return;
    };

    let witness_case_id = active_case_for_witness(
        &interaction.case_board,
        interaction.case_registry.as_deref(),
        &npc_id,
    );

    if npc_id == GHOST_ID {
        if let Some(case_id) = witness_case_id.as_deref() {
            record_witness_interview(&mut interaction.case_board, case_id, &npc_id);
        }

        interaction_output.toast_events.send(ToastEvent {
            message: ghost_dialogue_toast(
                &interaction.npc_registry,
                &interaction.case_board,
                interaction.case_registry.as_deref(),
                &interaction.clock,
            ),
            duration_secs: 3.8,
        });
        return;
    }

    if let Some(case_id) = active_case_for_suspect(
        &interaction.case_board,
        interaction.case_registry.as_deref(),
        &npc_id,
    ) {
        interaction_output
            .interrogation_events
            .send(InterrogationStartEvent { npc_id, case_id });
        return;
    }

    let context = if npc_id == "captain_torres" {
        "captain_briefing".to_string()
    } else if npc_id == PARTNER_ID {
        if interaction.case_board.active.is_empty() {
            "partner_patrol".to_string()
        } else {
            "partner_casework".to_string()
        }
    } else if let Some(case_id) = witness_case_id {
        record_witness_interview(&mut interaction.case_board, &case_id, &npc_id);
        format!("case_interview:{case_id}")
    } else {
        "npc_interaction".to_string()
    };

    interaction_output
        .dialogue_events
        .send(DialogueStartEvent { npc_id, context });
}

#[derive(SystemParam)]
struct NpcInteractionInput<'w, 's> {
    player_input: Res<'w, PlayerInput>,
    player_state: Res<'w, PlayerState>,
    clock: Res<'w, ShiftClock>,
    npc_registry: Res<'w, NpcRegistry>,
    case_board: ResMut<'w, CaseBoard>,
    case_registry: Option<Res<'w, CaseRegistry>>,
    npc_query: Query<'w, 's, (&'static Npc, &'static GridPosition)>,
}

#[derive(SystemParam)]
struct NpcInteractionOutput<'w, 's> {
    next_state: ResMut<'w, NextState<GameState>>,
    dialogue_events: EventWriter<'w, DialogueStartEvent>,
    interrogation_events: EventWriter<'w, InterrogationStartEvent>,
    toast_events: EventWriter<'w, ToastEvent>,
    marker: std::marker::PhantomData<&'s ()>,
}

fn handle_dialogue_cancel_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dialogue_end_events: EventWriter<DialogueEndEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        dialogue_end_events.send(DialogueEndEvent);
    }
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
        if let Some(active_case) = case_board
            .active
            .iter_mut()
            .find(|active_case| active_case.case_id == event.case_id)
        {
            active_case.status = crate::shared::CaseStatus::Interrogating;
        }
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
                evidence_id: "confession".to_string(),
                case_id: event.case_id.clone(),
                quality: CONFESSION_QUALITY,
            });
            xp_events.send(XpGainedEvent {
                amount: XP_PER_INTERROGATION,
                source: format!("interrogation:{}:{}", event.case_id, event.npc_id),
            });
        }

        if let Some(active_case) = case_board
            .active
            .iter_mut()
            .find(|active_case| active_case.case_id == event.case_id)
        {
            active_case.status = if event.confession {
                active_case.status
            } else {
                crate::shared::CaseStatus::Investigating
            };
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

fn emit_investigation_commentary(
    mut assignment_events: EventReader<CaseAssignedEvent>,
    mut evidence_events: EventReader<EvidenceCollectedEvent>,
    partner_arc: Res<PartnerArc>,
    clock: Res<ShiftClock>,
    mut commentary_state: ResMut<InvestigationCommentaryState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for event in assignment_events.read() {
        let partner_key = format!("vasquez:assignment:{}", event.case_id);
        if commentary_state.emitted_keys.insert(partner_key) {
            toast_events.send(ToastEvent {
                message: vasquez_assignment_comment(&event.case_id, partner_arc.stage),
                duration_secs: 3.2,
            });
        }

        if case_has_ghost_tip(&event.case_id) {
            let ghost_key = format!("ghost:assignment:{}", event.case_id);
            if commentary_state.emitted_keys.insert(ghost_key) {
                toast_events.send(ToastEvent {
                    message: ghost_case_tip(&event.case_id, clock.hour),
                    duration_secs: 3.6,
                });
            }
        }
    }

    for event in evidence_events.read() {
        let partner_key = format!("vasquez:evidence:{}", event.case_id);
        if commentary_state.emitted_keys.insert(partner_key) {
            toast_events.send(ToastEvent {
                message: vasquez_evidence_comment(
                    &event.case_id,
                    partner_arc.stage,
                    &event.evidence_id,
                ),
                duration_secs: 3.0,
            });
        }

        if case_has_ghost_tip(&event.case_id) {
            let ghost_key = format!("ghost:evidence:{}", event.case_id);
            if commentary_state.emitted_keys.insert(ghost_key) {
                toast_events.send(ToastEvent {
                    message: ghost_case_tip(&event.case_id, clock.hour),
                    duration_secs: 3.6,
                });
            }
        }
    }
}

fn emit_patrol_commentary(
    mut transition_events: EventReader<MapTransitionEvent>,
    case_board: Res<CaseBoard>,
    partner_arc: Res<PartnerArc>,
    clock: Res<ShiftClock>,
    mut commentary_state: ResMut<InvestigationCommentaryState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let Some(active_case_id) = case_board
        .active
        .first()
        .map(|active_case| active_case.case_id.as_str())
    else {
        return;
    };

    for event in transition_events.read() {
        if !is_exterior_map(event.to) || event.to == MapId::PrecinctExterior {
            continue;
        }

        let key = format!(
            "vasquez:patrol:{}:{:?}:{}",
            active_case_id, event.to, clock.shift_number
        );
        if !commentary_state.emitted_keys.insert(key) {
            continue;
        }

        toast_events.send(ToastEvent {
            message: vasquez_patrol_comment(
                active_case_id,
                partner_arc.stage,
                event.to.display_name(),
            ),
            duration_secs: 3.2,
        });
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
    asset_server: Option<&AssetServer>,
    atlas_layouts: &mut Option<ResMut<Assets<TextureAtlasLayout>>>,
) {
    for authored in AUTHORED_NPCS {
        if authored.id == GHOST_ID {
            continue;
        }

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

        spawn_npc_entity(
            commands,
            authored.id,
            definition,
            entry,
            clock,
            asset_server,
            atlas_layouts,
        );
    }
}

fn spawn_npc_entity(
    commands: &mut Commands,
    npc_id: &str,
    definition: &NpcDef,
    schedule_entry: &ScheduleEntry,
    clock: &ShiftClock,
    asset_server: Option<&AssetServer>,
    atlas_layouts: &mut Option<ResMut<Assets<TextureAtlasLayout>>>,
) {
    let grid_position = resolved_grid_position(npc_id, schedule_entry, clock);
    let world_position = grid_to_world(grid_position);
    let facing = default_npc_facing(npc_id);
    let sprite = npc_sprite(npc_id, facing, asset_server, atlas_layouts.as_deref_mut())
        .unwrap_or_else(|| {
            Sprite::from_color(role_color(definition.role), Vec2::splat(WORLD_TILE_SIZE))
        });

    commands.spawn((
        Npc {
            id: definition.id.clone(),
        },
        NpcFacing(facing),
        grid_position,
        sprite,
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

fn active_case_for_suspect(
    case_board: &CaseBoard,
    registry: Option<&CaseRegistry>,
    npc_id: &str,
) -> Option<String> {
    case_board.active.iter().find_map(|active_case| {
        if active_case.suspects_interrogated.contains(npc_id) {
            return None;
        }

        let is_suspect = registry
            .and_then(|registry| registry.get(&active_case.case_id))
            .map(|case_def| {
                case_def
                    .suspects
                    .iter()
                    .any(|suspect_id| suspect_id == npc_id)
            })
            .or_else(|| {
                case_definition(&active_case.case_id).map(|case_def| {
                    case_def
                        .suspects
                        .iter()
                        .any(|suspect_id| *suspect_id == npc_id)
                })
            })
            .unwrap_or(false);

        is_suspect.then(|| active_case.case_id.clone())
    })
}

fn active_case_for_witness(
    case_board: &CaseBoard,
    registry: Option<&CaseRegistry>,
    npc_id: &str,
) -> Option<String> {
    case_board.active.iter().find_map(|active_case| {
        if active_case.witnesses_interviewed.contains(npc_id) {
            return None;
        }

        let is_witness = registry
            .and_then(|registry| registry.get(&active_case.case_id))
            .map(|case_def| {
                case_def
                    .witnesses
                    .iter()
                    .any(|witness_id| witness_id == npc_id)
            })
            .or_else(|| {
                case_definition(&active_case.case_id).map(|case_def| {
                    case_def
                        .witnesses
                        .iter()
                        .any(|witness_id| witness_id == npc_id)
                })
            })
            .unwrap_or(false);

        is_witness.then(|| active_case.case_id.clone())
    })
}

fn record_witness_interview(case_board: &mut CaseBoard, case_id: &str, npc_id: &str) {
    let Some(active_case) = case_board
        .active
        .iter_mut()
        .find(|active_case| active_case.case_id == case_id)
    else {
        return;
    };

    if active_case.witnesses_interviewed.insert(npc_id.to_string()) {
        active_case
            .notes
            .push(format!("Interviewed witness {npc_id} about the case."));
    }
}

fn case_has_ghost_tip(case_id: &str) -> bool {
    witness_lines(case_id, GHOST_ID).is_some()
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

pub(crate) fn dialogue_text(
    npc_id: &str,
    context: Option<&str>,
    registry: &NpcRegistry,
    case_board: &CaseBoard,
    case_registry: Option<&CaseRegistry>,
    clock: &ShiftClock,
    partner_arc: &PartnerArc,
) -> String {
    let npc_name = registry
        .definitions
        .get(npc_id)
        .map(|npc| npc.name.as_str())
        .unwrap_or("Unknown contact");
    let relationship = registry.relationships.get(npc_id);
    let trust = relationship.map(|rel| rel.trust).unwrap_or_default();
    let trust_band = trust_band(trust);
    let time_band = time_band(clock.hour);
    let case_context = case_context_for_dialogue(case_board, case_registry, npc_id, context);

    if npc_id == "captain_torres" {
        return format!(
            "{npc_name}\n\n{}\n{}\n{}\n{}",
            captain_trust_line(trust_band),
            captain_time_line(time_band),
            captain_assignment_line(case_board, case_registry),
            captain_promotion_line(case_board),
        );
    }

    if npc_id == PARTNER_ID {
        let mut lines = vec![
            partner_stage_line(partner_arc.stage).to_string(),
            time_line_for_partner(time_band).to_string(),
        ];

        if let Some((case_id, case_name, is_witness, _)) = case_context.as_ref() {
            lines.push(format!(
                "Vasquez folds his arms. \"{case_name} is where somebody's version of the truth gets thin. Let's find which seam rips first.\""
            ));
            if *is_witness {
                if let Some(snippet) = witness_quote(case_id, npc_id) {
                    lines.push(format!("He adds, \"{snippet}\""));
                }
            }
        } else {
            lines.push(
                "Vasquez scans the block before answering. \"Quiet patrols are where the city hides the setup for noisier days.\""
                    .to_string(),
            );
        }

        if trust >= HIGH_TRUST_THRESHOLD {
            lines.push(
                "He lowers his voice. \"Take the second look, not the first one. First looks are what suspects rehearse for.\""
                    .to_string(),
            );
        }

        return format!("{npc_name}\n\n{}", lines.join("\n\n"));
    }

    let profile = dialogue_profile(npc_id);
    let mut lines = vec![
        trust_line(profile, trust_band).to_string(),
        time_line(profile, time_band).to_string(),
    ];

    if let Some((case_id, case_name, is_witness, is_suspect)) = case_context.as_ref() {
        lines.push(format_casework_line(
            npc_id,
            profile,
            case_name,
            *is_witness,
            *is_suspect,
        ));
        if *is_witness {
            if let Some(snippet) = witness_quote(case_id, npc_id) {
                lines.push(format!("\"{snippet}\""));
            }
        }
    } else {
        lines.push(profile.casework.to_string());
    }

    if trust >= HIGH_TRUST_THRESHOLD {
        lines.push(profile.volunteered.to_string());
    }

    format!("{npc_name}\n\n{}", lines.join("\n\n"))
}

pub(crate) fn ghost_dialogue_toast(
    registry: &NpcRegistry,
    case_board: &CaseBoard,
    case_registry: Option<&CaseRegistry>,
    clock: &ShiftClock,
) -> String {
    let trust = registry
        .relationships
        .get(GHOST_ID)
        .map(|rel| rel.trust)
        .unwrap_or_default();
    let mut lines = vec![match trust_band(trust) {
        TrustBand::Low => "A burner phone buzzes once: \"You keep arriving after the useful lies are gone.\"".to_string(),
        TrustBand::Mid => "A folded receipt appears under your boot: \"Better. Now stop announcing yourself to every frightened witness.\"".to_string(),
        TrustBand::High => "A text lands with no number attached: \"You are learning which silence matters. Do not waste it.\"".to_string(),
    }];

    lines.push(
        match time_band(clock.hour) {
            TimeBand::Morning => "Second line: \"Morning light flatters bad crime scenes. Look for what still seems ugly.\"",
            TimeBand::Afternoon => "Second line: \"By afternoon, the honest people are tired and the liars are polished. Interview accordingly.\"",
            TimeBand::Night => "Second line: \"Night keeps two ledgers: what happened, and what people swear happened once the dark got involved.\"",
        }
        .to_string(),
    );

    if let Some((case_id, case_name, is_witness, _)) =
        case_context_for_dialogue(case_board, case_registry, GHOST_ID, Some("ghost_toast"))
    {
        if is_witness {
            if let Some(snippet) = witness_quote(&case_id, GHOST_ID) {
                lines.push(format!("Case drop on {case_name}: \"{snippet}\""));
            }
        } else {
            lines.push(format!("Case drop on {case_name}: \"Somebody in this file is counting on routine to protect them. Break the routine.\""));
        }
    } else {
        lines.push(
            "No signature, just one last line: \"If you ever meet me properly, something already went wrong.\""
                .to_string(),
        );
    }

    lines.join("\n")
}

pub(crate) fn interrogation_summary_text(
    npc_id: &str,
    case_id: &str,
    registry: &NpcRegistry,
    case_registry: Option<&CaseRegistry>,
) -> String {
    let relationship = registry.relationships.get(npc_id);
    let trust = relationship.map(|rel| rel.trust).unwrap_or_default();
    let pressure = relationship.map(|rel| rel.pressure).unwrap_or_default();
    let case_name = case_name_for(case_registry, case_id);

    format!(
        "Trust: {trust} | Pressure: {pressure}\n{}\nThe room keeps circling {case_name}.",
        interrogation_feedback(npc_id, case_id, registry, InterrogationBeat::Endgame),
    )
}

pub(crate) fn interrogation_feedback(
    npc_id: &str,
    case_id: &str,
    registry: &NpcRegistry,
    beat: InterrogationBeat,
) -> String {
    let relationship = registry.relationships.get(npc_id);
    let trust = relationship.map(|rel| rel.trust).unwrap_or_default();
    let pressure = relationship.map(|rel| rel.pressure).unwrap_or_default();
    let trust_band = trust_band(trust);

    match beat {
        InterrogationBeat::BuildTrust => match npc_id {
            "officer_chen" => "Chen straightens his tie. \"Fine. Ask the question like you already know the answer and maybe we both leave with our pride.\"",
            "mayor_aldridge" => "Aldridge exhales carefully. \"Respect is rarer in this building than leverage. Keep going.\"",
            "marcus_cole" => "Marcus eases back a fraction. \"Talk to me like a man trying not to go back, not like a headline.\"",
            _ => "The suspect gives you a little more room to work with. Rapport is finally buying silence instead of resistance.",
        }
        .to_string(),
        InterrogationBeat::ApplyPressure => match npc_id {
            "officer_chen" => "Chen's jaw locks. \"You come at me sloppy and I make you prove every syllable.\"",
            "mayor_aldridge" => "Aldridge's eyes harden. \"Pressure is just theater unless you brought records with you.\"",
            "marcus_cole" => "Marcus leans forward. \"Push me harder than the facts can hold and I stop helping you remember anything.\"",
            _ => "The room tightens. Pressure is moving the suspect, but not in a direction you can control for free.",
        }
        .to_string(),
        InterrogationBeat::AskEvidence => match npc_id {
            "officer_chen" => format!("Chen glances at the file. \"If {case_id} hangs on one bad assumption, your whole ladder goes with it.\""),
            "mayor_aldridge" => format!("Aldridge folds her hands. \"Evidence is the only language that survives {case_id}. Speak that or go home.\""),
            "marcus_cole" => format!("Marcus taps the table once. \"On {case_id}, you already know which detail keeps getting skipped.\""),
            _ => "Mentioning the evidence shifts the room from posturing toward specifics, which is usually where lies lose their balance.".to_string(),
        }
        .to_string(),
        InterrogationBeat::Endgame => {
            let tone = if pressure >= 85 {
                "They are close to cracking, but only if your last move sounds inevitable."
            } else if pressure >= 45 {
                "The pressure is working unevenly. One more bad push and they will harden instead of fold."
            } else if trust_band == TrustBand::High {
                "Trust is doing more work than fear right now. A careful question may open them faster than a threat."
            } else {
                "You still do not own the room. Build leverage before you gamble on a confession."
            };

            match npc_id {
                "officer_chen" => format!("Chen keeps pretending this is a career conversation, not an interrogation. {tone}"),
                "mayor_aldridge" => format!("Aldridge treats every pause like a press conference beat. {tone}"),
                "marcus_cole" => format!("Marcus is measuring whether you want truth or just a body to pin it on. {tone}"),
                _ => tone.to_string(),
            }
        }
    }
}

fn dialogue_profile(npc_id: &str) -> DialogueProfile {
    match npc_id {
        "captain_torres" => DialogueProfile {
            low_trust: "Captain Torres studies you over the file stack. \"You do not earn easy assignments by wanting them.\"",
            mid_trust: "Captain Torres nods once. \"You are starting to sound like someone who listens before talking.\"",
            high_trust: "Captain Torres lets the sternness soften a notch. \"You are giving me fewer surprises and better outcomes. Keep that ratio.\"",
            morning: "She taps the duty board. \"Morning is for clean priorities. Decide yours before the city decides them for you.\"",
            afternoon: "She closes a folder with two fingers. \"By afternoon, every weak report starts to smell like an excuse.\"",
            night: "Her office light is still on. \"Night shift is where lazy thinking asks for cover. Do not give it any.\"",
            casework: "She glances at the board. \"Whatever is live, keep it factual enough that nobody upstairs can strangle it with politics.\"",
            volunteered: "\"Watch the person who looks relieved at bad news,\" she adds. \"Relief in a crisis usually means preparation.\"",
        },
        "officer_chen" => DialogueProfile {
            low_trust: "Chen barely looks up. \"If you need a map of the beat, ask someone who still walks slow enough to enjoy it.\"",
            mid_trust: "Chen shrugs. \"You are not dead weight, which already puts you above half the station.\"",
            high_trust: "Chen cracks a quick grin. \"You keep pace. That makes you useful and mildly irritating.\"",
            morning: "He flicks ash from a stale coffee lid. \"Morning calls tell you who panicked overnight and who planned ahead.\"",
            afternoon: "Chen watches the hallway traffic. \"Afternoons are for speed. Hesitation is how witnesses remember the wrong cop.\"",
            night: "He lowers his voice. \"Night patrol is when shortcuts start looking clever right before they ruin you.\"",
            casework: "Chen leans closer. \"If there is a live case, beat the rumor mill to the first useful detail or you are already late.\"",
            volunteered: "\"Check who filed last and talked first,\" Chen says. \"People do that when they need the story in the air before the facts arrive.\"",
        },
        "sgt_murphy" => DialogueProfile {
            low_trust: "Murphy rubs the bridge of his nose. \"Every rookie thinks the city gets stranger after midnight. It does not. You just get less patient.\"",
            mid_trust: "Murphy smiles like he has seen this lesson before. \"You are finally learning that the loudest person at a scene is usually the least useful.\"",
            high_trust: "Murphy shifts into mentor mode. \"You want my secret? Keep your notes clean enough that future-you feels respected.\"",
            morning: "\"Back in '09 we lost a burglary because someone trusted breakfast gossip over a timestamp,\" Murphy says. \"Morning lies always sound wholesome.\"",
            afternoon: "\"Afternoon desks are dangerous,\" Murphy warns. \"That is when tired cops decide memory counts as evidence.\"",
            night: "Murphy chuckles without humor. \"Night shift teaches humility. Every alley looks manageable until it starts talking back.\"",
            casework: "\"If a live case feels obvious,\" Murphy says, \"that means the liar had time to decorate it.\"",
            volunteered: "\"Free hint,\" he adds. \"The first person asking whether you solved it yet usually knows how close you are.\"",
        },
        "mayor_aldridge" => DialogueProfile {
            low_trust: "Mayor Aldridge folds her hands precisely. \"I appreciate initiative more when it arrives with discretion.\"",
            mid_trust: "Aldridge gives you a measured nod. \"You ask direct questions without performing them. That is rarer than it should be.\"",
            high_trust: "Her tone warms by one degree. \"If I share a concern with you, it is because I expect you to act like a professional and not a tourist.\"",
            morning: "She checks the day's briefing cards. \"Morning headlines are written by noon. Solve problems before they become language.\"",
            afternoon: "Aldridge watches the street from the courthouse window. \"By afternoon, every rumor has found a microphone.\"",
            night: "She exhales at the empty hall. \"Night is when city business pretends it is private.\"",
            casework: "Aldridge lowers her voice. \"If the current case touches an office downtown, assume two more people know than admit it.\"",
            volunteered: "\"Follow contracts and favors together,\" she says. \"Scandal breeds where those two stop pretending to be separate.\"",
        },
        "dr_okafor" => DialogueProfile {
            low_trust: "Dr. Okafor adjusts his gloves. \"Speculation is a fine hobby. Bring me samples if you would like science instead.\"",
            mid_trust: "He nods toward the tray. \"You have started asking questions in an order the evidence can tolerate.\"",
            high_trust: "Okafor's dry smile appears. \"I set aside my good microscope time for detectives who do not store evidence beside coffee.\"",
            morning: "\"Bodies are most honest before everyone else wakes up and starts narrating them,\" Okafor says.",
            afternoon: "\"By afternoon the living have edited the story three times,\" he murmurs. \"The tissue usually has not.\"",
            night: "\"Night work is simple,\" Okafor says. \"The dead remain punctual. The living become dramatic.\"",
            casework: "\"On the live file, the evidence will wait,\" he says. \"Human memory will not. Prioritize accordingly.\"",
            volunteered: "\"If you want the case to hold up,\" he adds, \"bring me context with the sample, not after I have disproved your theory.\"",
        },
        "rita_gomez" => DialogueProfile {
            low_trust: "Rita wipes a glass without looking up. \"Coffee is cheap. My confidence is not.\"",
            mid_trust: "Rita leans on the counter. \"You have learned how to ask a question without making the whole room leave first.\"",
            high_trust: "She slides a fresh cup your way. \"You keep my name out of ugly paperwork. That buys you real answers.\"",
            morning: "\"Breakfast crowd lies with their faces,\" Rita says. \"Lunch crowd lies with their wallets.\"",
            afternoon: "\"Afternoons are good for rumors,\" she says. \"People get bold once they think the hard part of the day already happened.\"",
            night: "\"Night crowd talks like shadows have attorneys,\" Rita mutters. \"Listen anyway.\"",
            casework: "Rita lowers her voice. \"If there is a case running, somebody already used this diner to test which story sounds safest out loud.\"",
            volunteered: "\"Here is the free part,\" Rita says. \"Ask who suddenly paid cash today and you usually find the real panic.\"",
        },
        "father_brennan" => DialogueProfile {
            low_trust: "Father Brennan clasps his hands. \"Authority without patience sounds a great deal like fear, officer.\"",
            mid_trust: "He studies you gently. \"You are beginning to leave enough silence for truth to enter the room on its own.\"",
            high_trust: "Brennan inclines his head. \"I do not offer trust lightly, but you have stopped treating pain like an interruption.\"",
            morning: "\"Morning confessions are rarely complete,\" he says. \"People still believe they can outrun themselves by lunch.\"",
            afternoon: "\"By afternoon remorse gets practical,\" Brennan notes. \"That is when the details start arriving.\"",
            night: "\"Night is when guilt grows theatrical,\" he says. \"Useful, but not the same as honest.\"",
            casework: "\"Whatever the current case did to the neighborhood,\" Brennan says, \"someone is waiting for permission to tell the difficult version.\"",
            volunteered: "\"Watch who seeks absolution before accusation,\" he adds. \"That path is rarely random.\"",
        },
        "ghost_tipster" => DialogueProfile {
            low_trust: "\"You are still chasing surface noise.\"",
            mid_trust: "\"Better. You finally know a clue can whisper.\"",
            high_trust: "\"Now you are listening like somebody who intends to survive the answer.\"",
            morning: "\"Morning makes bad staging look tidy.\"",
            afternoon: "\"By afternoon, liars have polished the obvious.\"",
            night: "\"Night keeps the version people fear and the version they caused.\"",
            casework: "\"Every live case has one person praying routine holds for one more hour.\"",
            volunteered: "\"Break the schedule and the truth starts limping.\"",
        },
        "nadia_park" => DialogueProfile {
            low_trust: "Nadia keeps writing while you speak. \"If you want me off the record, offer me something worth not printing.\"",
            mid_trust: "She pockets the notepad. \"You have developed the useful habit of asking what happened before asking who benefits.\"",
            high_trust: "Nadia tilts her head. \"I trust you more than most uniforms, which is either progress or a sign of civic decline.\"",
            morning: "\"Morning sources are cautious,\" Nadia says. \"They still think the day can be controlled.\"",
            afternoon: "\"By afternoon everyone has chosen their angle,\" she says. \"That is when pattern beats quote.\"",
            night: "\"Night reporting is easy,\" Nadia says. \"Everyone mistakes exhaustion for honesty.\"",
            casework: "\"If there is a live file, assume someone has already started rewriting it for tomorrow's audience,\" Nadia says.",
            volunteered: "\"When witnesses all reuse the same adjective,\" she adds, \"someone handed them language before they handed you facts.\"",
        },
        "marcus_cole" => DialogueProfile {
            low_trust: "Marcus keeps his hands visible on purpose. \"Every cop says they only want the truth right up until the truth looks inconvenient.\"",
            mid_trust: "He eyes the street, not you. \"You are learning the difference between a nervous man and a guilty one.\"",
            high_trust: "Marcus gives a short nod. \"I know when somebody is trying to clear a case and when they are trying to clear a person. You have gotten better at the second part.\"",
            morning: "\"Morning crews brag too early,\" Marcus says. \"That is useful if you know where they buy coffee.\"",
            afternoon: "\"By afternoon the smart ones go quiet,\" he mutters. \"Only the desperate stay visible.\"",
            night: "\"Night makes everybody feel ten feet tall,\" Marcus says. \"That is why so many bad plans leave footprints.\"",
            casework: "\"If the current case brushes the old crews,\" Marcus says, \"I can tell you what kind of mistake they think nobody notices anymore.\"",
            volunteered: "\"Street rule,\" he adds. \"After a messy job, the guilty stop boasting and start asking who talked.\"",
        },
        "lucia_vega" => DialogueProfile {
            low_trust: "Lucia folds a case file shut. \"If you came looking for praise, you took a wrong turn at the courthouse.\"",
            mid_trust: "She studies you carefully. \"You have become slightly less allergic to due process. I notice these things.\"",
            high_trust: "Lucia's posture softens without surrendering. \"I still challenge your work. I simply no longer assume it deserves to fail.\"",
            morning: "\"Morning arraignments are where sloppy police work sobers up,\" Lucia says.",
            afternoon: "\"By afternoon everyone starts pretending shortcuts were strategy,\" she says. \"I dislike that hour.\"",
            night: "\"Night is when bad cases hope no lawyer is still awake,\" Lucia says. \"Unfortunate for them.\"",
            casework: "\"If your live case depends on a shortcut,\" Lucia says, \"fix it now. Court punishes arrogance more reliably than crime does.\"",
            volunteered: "\"The innocent ramble,\" she adds. \"The coached answer in tidy bricks. Listen for the masonry.\"",
        },
        _ => DialogueProfile {
            low_trust: "They study you warily, weighing badge against motive.",
            mid_trust: "They answer with a little less caution than before.",
            high_trust: "They seem willing to risk giving you the useful version.",
            morning: "Morning has them alert and guarded.",
            afternoon: "Afternoon leaves them practical, not patient.",
            night: "Night makes every answer feel more expensive.",
            casework: "They keep circling back to the live work on your board.",
            volunteered: "At high trust, they give you one extra detail for free.",
        },
    }
}

fn trust_line(profile: DialogueProfile, band: TrustBand) -> &'static str {
    match band {
        TrustBand::Low => profile.low_trust,
        TrustBand::Mid => profile.mid_trust,
        TrustBand::High => profile.high_trust,
    }
}

fn time_line(profile: DialogueProfile, band: TimeBand) -> &'static str {
    match band {
        TimeBand::Morning => profile.morning,
        TimeBand::Afternoon => profile.afternoon,
        TimeBand::Night => profile.night,
    }
}

fn trust_band(trust: i32) -> TrustBand {
    if trust >= HIGH_TRUST_THRESHOLD {
        TrustBand::High
    } else if trust >= 20 {
        TrustBand::Mid
    } else {
        TrustBand::Low
    }
}

fn time_band(hour: u8) -> TimeBand {
    if !(6..18).contains(&hour) {
        TimeBand::Night
    } else if hour >= 12 {
        TimeBand::Afternoon
    } else {
        TimeBand::Morning
    }
}

fn case_context_for_dialogue(
    case_board: &CaseBoard,
    case_registry: Option<&CaseRegistry>,
    npc_id: &str,
    context: Option<&str>,
) -> Option<(String, String, bool, bool)> {
    let preferred_case_id = context
        .and_then(|context| context.strip_prefix("case_interview:"))
        .map(str::to_string);

    if let Some(case_id) = preferred_case_id {
        let case_name = case_name_for(case_registry, &case_id);
        let is_witness = case_has_witness(case_registry, &case_id, npc_id);
        let is_suspect = case_has_suspect(case_registry, &case_id, npc_id);

        return Some((case_id, case_name, is_witness, is_suspect));
    }

    for active_case in &case_board.active {
        let case_name = case_name_for(case_registry, &active_case.case_id);
        let is_witness = case_has_witness(case_registry, &active_case.case_id, npc_id);
        let is_suspect = case_has_suspect(case_registry, &active_case.case_id, npc_id);

        if is_witness || is_suspect || npc_id == PARTNER_ID || npc_id == "captain_torres" {
            return Some((
                active_case.case_id.clone(),
                case_name,
                is_witness,
                is_suspect,
            ));
        }
    }

    case_board.active.first().map(|active_case| {
        (
            active_case.case_id.clone(),
            case_name_for(case_registry, &active_case.case_id),
            false,
            false,
        )
    })
}

fn case_name_for(case_registry: Option<&CaseRegistry>, case_id: &str) -> String {
    case_registry
        .and_then(|registry| registry.get(case_id))
        .map(|case_def| case_def.name.clone())
        .or_else(|| case_definition(case_id).map(|case_def| case_def.name))
        .unwrap_or_else(|| case_id.to_string())
}

fn case_has_witness(case_registry: Option<&CaseRegistry>, case_id: &str, npc_id: &str) -> bool {
    case_registry
        .and_then(|registry| registry.get(case_id))
        .map(|case_def| {
            case_def
                .witnesses
                .iter()
                .any(|witness_id| witness_id == npc_id)
        })
        .or_else(|| {
            case_definition(case_id).map(|case_def| {
                case_def
                    .witnesses
                    .iter()
                    .any(|witness_id| witness_id == npc_id)
            })
        })
        .unwrap_or(false)
}

fn case_has_suspect(case_registry: Option<&CaseRegistry>, case_id: &str, npc_id: &str) -> bool {
    case_registry
        .and_then(|registry| registry.get(case_id))
        .map(|case_def| {
            case_def
                .suspects
                .iter()
                .any(|suspect_id| suspect_id == npc_id)
        })
        .or_else(|| {
            case_definition(case_id).map(|case_def| {
                case_def
                    .suspects
                    .iter()
                    .any(|suspect_id| suspect_id == npc_id)
            })
        })
        .unwrap_or(false)
}

fn witness_quote(case_id: &str, npc_id: &str) -> Option<&'static str> {
    witness_lines(case_id, npc_id).and_then(|lines| lines.first().copied())
}

fn format_casework_line(
    npc_id: &str,
    profile: DialogueProfile,
    case_name: &str,
    is_witness: bool,
    is_suspect: bool,
) -> String {
    if is_suspect {
        return format!("They stiffen at the mention of {case_name}, already treating the conversation like a closing door.");
    }

    match npc_id {
        "sgt_murphy" => {
            format!("Murphy taps the blotter. \"{case_name} is either simpler than it looks or proud of looking simple. I distrust both.\"")
        }
        "rita_gomez" => {
            format!("Rita keeps her voice low. \"Word on {case_name} spread faster than the sirens. That usually means someone wanted the neighborhood rehearsed.\"")
        }
        "dr_okafor" => {
            format!("Okafor glances at your notebook. \"On {case_name}, bring me timing with your evidence and I can save you two bad theories.\"")
        }
        "father_brennan" => {
            format!("Brennan looks past you toward the street. \"{case_name} has people deciding whether truth will cost them more than silence.\"")
        }
        "mayor_aldridge" => {
            format!("Aldridge lowers her voice. \"If {case_name} spills into public panic, the facts will need to arrive before the speeches do.\"")
        }
        "nadia_park" => {
            format!("Nadia taps her pen against the pad. \"{case_name} already has three competing narratives, and only one of them belongs to reality.\"")
        }
        "marcus_cole" => {
            format!("Marcus folds his arms. \"If {case_name} smells like a crew job, somebody on the edge is already wondering who sold them cheap.\"")
        }
        "lucia_vega" => {
            format!("Lucia raises an eyebrow. \"If {case_name} reaches court with holes in it, I will find them before the jury does.\"")
        }
        "officer_chen" => {
            format!("Chen flashes a tight grin. \"{case_name} rewards whoever moves first and documents second. Try not to be that cop.\"")
        }
        _ if is_witness => {
            format!("They keep circling back to {case_name}, treating every answer like testimony.")
        }
        _ => profile.casework.to_string(),
    }
}

fn captain_trust_line(band: TrustBand) -> &'static str {
    match band {
        TrustBand::Low => "Captain Torres keeps the file closed. \"I do not grade effort. I grade outcomes.\"",
        TrustBand::Mid => "Captain Torres nods once. \"You are starting to look steadier under pressure. Do not confuse that with finished.\"",
        TrustBand::High => "Captain Torres lets herself sound almost proud. \"You have become the kind of cop I can brief in one sentence and trust with the other nine I leave unsaid.\"",
    }
}

fn captain_time_line(band: TimeBand) -> &'static str {
    match band {
        TimeBand::Morning => "\"Morning briefing matters because panic has not had time to write over the facts yet,\" she says.",
        TimeBand::Afternoon => "\"By afternoon the city has opinions. I still want evidence,\" Torres says.",
        TimeBand::Night => "\"Night work punishes vanity faster than daylight does,\" Torres says.",
    }
}

fn captain_assignment_line(case_board: &CaseBoard, case_registry: Option<&CaseRegistry>) -> String {
    if let Some(active_case) = case_board.active.first() {
        let case_name = case_name_for(case_registry, &active_case.case_id);
        format!("She points to the board. \"Your assignment is {case_name}. Keep the witnesses talking and the paperwork cleaner than the scene.\"")
    } else if let Some(case_id) = case_board.available.first() {
        let case_name = case_name_for(case_registry, case_id);
        format!("She taps the next card on the board. \"Take {case_name}. Bring me facts, not atmosphere.\"")
    } else {
        "She glances toward the empty board. \"No fresh assignment right now. That means you clean up the old messes before they turn into new ones.\"".to_string()
    }
}

fn captain_promotion_line(case_board: &CaseBoard) -> &'static str {
    match case_board.total_cases_solved {
        0..=4 => "\"You want promotion talk? Stack clean closes until command stops asking who you are.\"",
        5..=11 => "\"Your file is starting to look like detective material. Keep it disciplined.\"",
        12..=19 => "\"Sergeant work means carrying other people's mistakes without adding your own. Start practicing now.\"",
        _ => "\"At this point promotion is not about ambition. It is about whether you can make the room steadier when everyone else wobbles.\"",
    }
}

fn vasquez_assignment_comment(case_id: &str, stage: PartnerStage) -> String {
    let case_name = case_name_for(None, case_id);
    let line = match stage {
        PartnerStage::Stranger => {
            "Torres gave us a live file, so keep your eyes open and your ego quiet."
        }
        PartnerStage::UneasyPartners => {
            "We'll split the ground and compare notes before the city lies to us twice."
        }
        PartnerStage::WorkingRapport => {
            "Good. A real case. Enough pressure to matter, not enough to rush the fundamentals."
        }
        PartnerStage::TrustedPartners => {
            "This one fits us. You work the people, I'll work the seams around them."
        }
        PartnerStage::BestFriends => {
            "We know the dance now. Let's close it clean before the town writes its own ending."
        }
    };

    format!("Vasquez: {line} ({case_name})")
}

fn vasquez_evidence_comment(case_id: &str, stage: PartnerStage, evidence_id: &str) -> String {
    let evidence_name = evidence_id.replace('_', " ");
    let line = match stage {
        PartnerStage::Stranger => "One hard detail beats ten opinions.",
        PartnerStage::UneasyPartners => {
            "That helps. Keep the chain of proof tighter than the gossip around it."
        }
        PartnerStage::WorkingRapport => "Nice pull. The case just got narrower.",
        PartnerStage::TrustedPartners => "That's the kind of evidence that makes bad alibis sweat.",
        PartnerStage::BestFriends => {
            "Beautiful. Now the rest of the story has fewer places to hide."
        }
    };

    format!("Vasquez: {line} ({case_id}, {evidence_name})")
}

fn vasquez_patrol_comment(case_id: &str, stage: PartnerStage, map_name: &str) -> String {
    let line = match stage {
        PartnerStage::Stranger => "Eyes up. Streets tell the truth faster than interviews do.",
        PartnerStage::UneasyPartners => {
            "This is where the file stops helping and the neighborhood starts talking."
        }
        PartnerStage::WorkingRapport => {
            "Walk it slow. Patterns show themselves when you quit trying to force them."
        }
        PartnerStage::TrustedPartners => {
            "Good ground. The block will tell us what the reports left polite."
        }
        PartnerStage::BestFriends => {
            "Right street, right instincts. Let's prove it before sunset does."
        }
    };

    format!("Vasquez: {line} ({case_id} near {map_name})")
}

fn ghost_case_tip(case_id: &str, hour: u8) -> String {
    let prefix = match time_band(hour) {
        TimeBand::Morning => "Burner note before breakfast:",
        TimeBand::Afternoon => "Folded receipt by your elbow:",
        TimeBand::Night => "Night message from Ghost:",
    };
    let quote = witness_quote(case_id, GHOST_ID).unwrap_or(
        "Somebody in this file is praying routine holds for one more hour. Break the routine.",
    );

    format!("{prefix} \"{quote}\"")
}

fn partner_stage_line(stage: PartnerStage) -> &'static str {
    match stage {
        PartnerStage::Stranger => "Vasquez does not bother pretending warmth. \"Keep up, keep your notes straight, and do not make me translate rookie mistakes into report language.\"",
        PartnerStage::UneasyPartners => "Vasquez gives you a measured glance. \"You are asking better questions. Ask them a little quieter and we might get somewhere.\"",
        PartnerStage::WorkingRapport => "He nods toward the street. \"You are finally reading the room instead of just entering it. That helps.\"",
        PartnerStage::TrustedPartners => "Vasquez relaxes a fraction. \"I can trust your first instincts now, which saves us both time.\"",
        PartnerStage::BestFriends => "He gives you the kind of look reserved for survivors. \"If this goes sideways, you are not standing in it alone.\"",
    }
}

fn time_line_for_partner(band: TimeBand) -> &'static str {
    match band {
        TimeBand::Morning => {
            "\"Morning patrol tells you which lies the city rehearsed overnight,\" Vasquez says."
        }
        TimeBand::Afternoon => {
            "\"Afternoons are for leaning on timelines before they soften,\" Vasquez says."
        }
        TimeBand::Night => "\"Night work is when bad decisions travel farthest,\" Vasquez says.",
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

fn npc_sprite(
    npc_id: &str,
    facing: Facing,
    asset_server: Option<&AssetServer>,
    atlas_layouts: Option<&mut Assets<TextureAtlasLayout>>,
) -> Option<Sprite> {
    let spec = npc_sprite_sheet_spec(npc_id)?;
    character_sprite(spec, facing, asset_server, atlas_layouts)
}

fn character_sprite(
    spec: CharacterSpriteSheetSpec,
    facing: Facing,
    asset_server: Option<&AssetServer>,
    atlas_layouts: Option<&mut Assets<TextureAtlasLayout>>,
) -> Option<Sprite> {
    let asset_server = asset_server?;
    let atlas_layouts = atlas_layouts?;
    let texture = asset_server.load(spec.path);
    let layout = atlas_layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        spec.columns,
        spec.rows,
        None,
        None,
    ));
    let mut sprite = Sprite::from_atlas_image(
        texture,
        TextureAtlas {
            layout,
            index: character_facing_frame(facing),
        },
    );
    sprite.custom_size = Some(Vec2::splat(WORLD_TILE_SIZE));
    Some(sprite)
}

fn sync_character_sprite(sprite: &mut Sprite, facing: Facing) {
    if let Some(texture_atlas) = sprite.texture_atlas.as_mut() {
        texture_atlas.index = character_facing_frame(facing);
    }
}

fn character_facing_frame(facing: Facing) -> usize {
    match facing {
        Facing::Left => 0,
        Facing::Right => 1,
        Facing::Up => 2,
        Facing::Down => 3,
    }
}

fn default_npc_facing(npc_id: &str) -> Facing {
    match npc_id {
        "captain_torres" | "sgt_murphy" | "mayor_aldridge" | "dr_okafor" => Facing::Down,
        "det_vasquez" | "father_brennan" | "lucia_vega" => Facing::Left,
        "officer_chen" | "rita_gomez" | "nadia_park" => Facing::Right,
        _ => Facing::Up,
    }
}

fn facing_from_grid_delta(
    previous: GridPosition,
    current: GridPosition,
    fallback: Facing,
) -> Facing {
    let delta = IVec2::new(current.x - previous.x, current.y - previous.y);

    if delta.x.abs() > delta.y.abs() {
        if delta.x > 0 {
            Facing::Right
        } else if delta.x < 0 {
            Facing::Left
        } else {
            fallback
        }
    } else if delta.y != 0 {
        if delta.y > 0 {
            Facing::Up
        } else {
            Facing::Down
        }
    } else {
        fallback
    }
}

fn npc_sprite_sheet_spec(npc_id: &str) -> Option<CharacterSpriteSheetSpec> {
    Some(match npc_id {
        "captain_torres" => CharacterSpriteSheetSpec {
            path: "characters/captain_torres.png",
            columns: 24,
            rows: 14,
        },
        "det_vasquez" => CharacterSpriteSheetSpec {
            path: "characters/det_vasquez.png",
            columns: 24,
            rows: 14,
        },
        "officer_chen" => CharacterSpriteSheetSpec {
            path: "characters/officer_chen.png",
            columns: 24,
            rows: 14,
        },
        "sgt_murphy" => CharacterSpriteSheetSpec {
            path: "characters/sgt_murphy.png",
            columns: 4,
            rows: 2,
        },
        "mayor_aldridge" => CharacterSpriteSheetSpec {
            path: "characters/mayor_aldridge.png",
            columns: 4,
            rows: 2,
        },
        "dr_okafor" => CharacterSpriteSheetSpec {
            path: "characters/dr_okafor.png",
            columns: 4,
            rows: 2,
        },
        "rita_gomez" => CharacterSpriteSheetSpec {
            path: "characters/rita_gomez.png",
            columns: 24,
            rows: 2,
        },
        "father_brennan" => CharacterSpriteSheetSpec {
            path: "characters/father_brennan.png",
            columns: 4,
            rows: 2,
        },
        "ghost_tipster" => CharacterSpriteSheetSpec {
            path: "characters/ghost_tipster.png",
            columns: 24,
            rows: 2,
        },
        "nadia_park" => CharacterSpriteSheetSpec {
            path: "characters/nadia_park.png",
            columns: 24,
            rows: 2,
        },
        "marcus_cole" => CharacterSpriteSheetSpec {
            path: "characters/marcus_cole.png",
            columns: 24,
            rows: 2,
        },
        "lucia_vega" => CharacterSpriteSheetSpec {
            path: "characters/lucia_vega.png",
            columns: 24,
            rows: 2,
        },
        _ => return None,
    })
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
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.add_event::<DialogueStartEvent>();
        app.add_event::<DialogueEndEvent>();
        app.add_event::<CaseAssignedEvent>();
        app.add_event::<InterrogationStartEvent>();
        app.add_event::<InterrogationEndEvent>();
        app.add_event::<NpcTrustChangeEvent>();
        app.add_event::<CaseAssignedEvent>();
        app.add_event::<EvidenceCollectedEvent>();
        app.add_event::<MapTransitionEvent>();
        app.add_event::<XpGainedEvent>();
        app.add_event::<ToastEvent>();
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
    fn ghost_never_spawns_face_to_face() {
        let mut app = build_test_app();
        app.update();
        app.world_mut().resource_mut::<ShiftClock>().hour = 12;
        set_player_grid(&mut app, MapId::Downtown, GridPosition { x: 20, y: 18 });

        enter_playing(&mut app);

        let mut npc_query = app.world_mut().query::<&Npc>();
        let spawned: HashSet<_> = npc_query
            .iter(app.world())
            .map(|npc| npc.id.clone())
            .collect();

        assert!(!spawned.contains(GHOST_ID));
    }

    #[test]
    fn case_assignment_emits_partner_and_ghost_commentary_toasts() {
        let mut app = build_test_app();
        app.update();

        app.world_mut()
            .resource_mut::<Events<CaseAssignedEvent>>()
            .send(CaseAssignedEvent {
                case_id: "detective_005_arson".to_string(),
            });

        app.update();

        let events = app.world().resource::<Events<ToastEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader
            .read(events)
            .map(|event| event.message.clone())
            .collect();

        assert!(emitted.iter().any(|message| message.contains("Vasquez:")));
        assert!(emitted.iter().any(|message| {
            message.contains("Burner note")
                || message.contains("Folded receipt")
                || message.contains("Night message")
        }));
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
            "confession".to_string(),
            "patrol_001_petty_theft".to_string()
        )));
    }

    #[test]
    fn suspect_interaction_emits_interrogation_start_event() {
        let mut app = build_test_app();
        app.update();
        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_001_petty_theft".to_string(),
                status: CaseStatus::Active,
                evidence_collected: Vec::new(),
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });

        set_player_grid(
            &mut app,
            MapId::ResidentialSouth,
            GridPosition { x: 12, y: 16 },
        );
        enter_playing(&mut app);

        app.world_mut().resource_mut::<PlayerInput>().interact = true;
        app.update();

        let events = app.world().resource::<Events<InterrogationStartEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader
            .read(events)
            .map(|event| (event.npc_id.clone(), event.case_id.clone()))
            .collect();

        assert!(emitted.contains(&(
            "marcus_cole".to_string(),
            "patrol_001_petty_theft".to_string()
        )));
    }

    #[test]
    fn leaving_dialogue_emits_dialogue_end_event() {
        let mut app = build_test_app();
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Dialogue);
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);

        app.update();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.update();

        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Playing
        );

        let events = app.world().resource::<Events<DialogueEndEvent>>();
        let mut reader = events.get_cursor();
        assert_eq!(reader.read(events).count(), 1);
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

    #[test]
    fn ghost_dialogue_resolves_to_toast_only_clue() {
        let mut app = build_test_app();
        app.update();
        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_007_graffiti".to_string(),
                status: CaseStatus::Active,
                evidence_collected: Vec::new(),
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });
        let toast = ghost_dialogue_toast(
            app.world().resource::<NpcRegistry>(),
            app.world().resource::<CaseBoard>(),
            None,
            app.world().resource::<ShiftClock>(),
        );

        let lowered = toast.to_ascii_lowercase();
        assert!(lowered.contains("burner") || lowered.contains("paint"));
    }

    #[test]
    fn high_trust_dialogue_volunteers_extra_case_context() {
        let mut app = build_test_app();
        app.update();
        app.world_mut()
            .resource_mut::<CaseBoard>()
            .active
            .push(ActiveCase {
                case_id: "patrol_001_petty_theft".to_string(),
                status: CaseStatus::Investigating,
                evidence_collected: vec!["fingerprint".to_string()],
                witnesses_interviewed: HashSet::new(),
                suspects_interrogated: HashSet::new(),
                shifts_elapsed: 0,
                notes: Vec::new(),
            });
        app.world_mut()
            .resource_mut::<NpcRegistry>()
            .relationships
            .get_mut("rita_gomez")
            .unwrap()
            .trust = 75;

        let dialogue = dialogue_text(
            "rita_gomez",
            Some("case_interview:patrol_001_petty_theft"),
            app.world().resource::<NpcRegistry>(),
            app.world().resource::<CaseBoard>(),
            None,
            app.world().resource::<ShiftClock>(),
            app.world().resource::<PartnerArc>(),
        );

        assert!(dialogue.contains("Petty Theft at General Store"));
        assert!(dialogue.contains("Here is the free part"));
        assert!(dialogue.contains("Marcus kept circling the register"));
    }
}
