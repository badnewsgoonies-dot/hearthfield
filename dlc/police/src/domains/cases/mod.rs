use std::collections::HashSet;

use bevy::prelude::*;

use crate::shared::{
    ActiveCase, CaseAssignedEvent, CaseBoard, CaseDef, CaseFailedEvent, CaseId, CaseSolvedEvent,
    CaseStatus, EvidenceCollectedEvent, EvidenceId, GameState, MapId, NpcId, PromotionEvent, Rank,
    ShiftClock, ShiftEndEvent, UpdatePhase,
};

#[derive(Resource, Debug, Clone, Default)]
struct CaseRegistry {
    defs: Vec<CaseDef>,
}

impl CaseRegistry {
    fn get(&self, case_id: &str) -> Option<&CaseDef> {
        self.defs.iter().find(|case_def| case_def.id == case_id)
    }
}

#[derive(Event, Debug, Clone)]
pub struct CaseCloseRequestedEvent {
    pub case_id: CaseId,
}

pub struct CasesPlugin;

impl Plugin for CasesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CaseRegistry>()
            .add_event::<CaseCloseRequestedEvent>()
            .add_systems(Startup, populate_case_registry)
            .add_systems(
                Update,
                (
                    handle_case_assignment,
                    track_evidence_for_cases,
                    check_evidence_complete,
                    handle_case_close,
                    advance_case_shifts,
                    check_case_expiry,
                    refresh_available_cases,
                )
                    .chain()
                    .in_set(UpdatePhase::Reactions)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn populate_case_registry(
    mut registry: ResMut<CaseRegistry>,
    mut case_board: ResMut<CaseBoard>,
    clock: Res<ShiftClock>,
) {
    registry.defs = build_case_registry();
    case_board.available.clear();
    replenish_available_cases(&registry.defs, &mut case_board, clock.rank);
}

fn handle_case_assignment(
    mut assignments: EventReader<CaseAssignedEvent>,
    registry: Res<CaseRegistry>,
    mut case_board: ResMut<CaseBoard>,
) {
    for assignment in assignments.read() {
        if case_board.active.len() >= crate::shared::MAX_ACTIVE_CASES {
            continue;
        }

        if registry.get(&assignment.case_id).is_none() {
            continue;
        }

        let Some(available_index) = case_board
            .available
            .iter()
            .position(|case_id| case_id == &assignment.case_id)
        else {
            continue;
        };

        let case_id = case_board.available.remove(available_index);
        case_board.active.push(ActiveCase {
            case_id,
            status: CaseStatus::Active,
            evidence_collected: Vec::new(),
            witnesses_interviewed: HashSet::default(),
            suspects_interrogated: HashSet::default(),
            shifts_elapsed: 0,
            notes: Vec::new(),
        });
    }
}

fn track_evidence_for_cases(
    mut evidence_events: EventReader<EvidenceCollectedEvent>,
    registry: Res<CaseRegistry>,
    mut case_board: ResMut<CaseBoard>,
) {
    for evidence_event in evidence_events.read() {
        let Some(active_case) = case_board
            .active
            .iter_mut()
            .find(|active_case| active_case.case_id == evidence_event.case_id)
        else {
            continue;
        };

        let Some(case_def) = registry.get(&active_case.case_id) else {
            continue;
        };

        if !case_def
            .evidence_required
            .iter()
            .any(|evidence_id| evidence_id == &evidence_event.evidence_id)
        {
            continue;
        }

        if active_case
            .evidence_collected
            .iter()
            .any(|evidence_id| evidence_id == &evidence_event.evidence_id)
        {
            continue;
        }

        active_case
            .evidence_collected
            .push(evidence_event.evidence_id.clone());

        if active_case.status == CaseStatus::Active {
            active_case.status = CaseStatus::Investigating;
        }
    }
}

fn check_evidence_complete(registry: Res<CaseRegistry>, mut case_board: ResMut<CaseBoard>) {
    for active_case in &mut case_board.active {
        let Some(case_def) = registry.get(&active_case.case_id) else {
            continue;
        };

        if case_def
            .evidence_required
            .iter()
            .all(|required| active_case.evidence_collected.contains(required))
        {
            active_case.status = CaseStatus::EvidenceComplete;
        }
    }
}

fn handle_case_close(
    mut close_requests: EventReader<CaseCloseRequestedEvent>,
    registry: Res<CaseRegistry>,
    mut case_board: ResMut<CaseBoard>,
    mut case_solved: EventWriter<CaseSolvedEvent>,
) {
    for close_request in close_requests.read() {
        let Some(active_index) = case_board.active.iter().position(|active_case| {
            active_case.case_id == close_request.case_id
                && active_case.status == CaseStatus::EvidenceComplete
        }) else {
            continue;
        };

        let solved_case = case_board.active.remove(active_index);
        let Some(case_def) = registry.get(&solved_case.case_id) else {
            continue;
        };

        case_solved.send(CaseSolvedEvent {
            case_id: solved_case.case_id.clone(),
            xp_reward: case_def.reward_xp,
            gold_reward: case_def.reward_gold,
            reputation_reward: case_def.reward_reputation,
        });
        case_board.solved.push(solved_case.case_id);
        case_board.total_cases_solved = case_board.total_cases_solved.saturating_add(1);
    }
}

fn advance_case_shifts(
    mut shift_end_events: EventReader<ShiftEndEvent>,
    mut case_board: ResMut<CaseBoard>,
) {
    let shifts_to_advance = shift_end_events.read().count() as u8;
    if shifts_to_advance == 0 {
        return;
    }

    for active_case in &mut case_board.active {
        active_case.shifts_elapsed = active_case.shifts_elapsed.saturating_add(shifts_to_advance);
    }
}

fn check_case_expiry(
    registry: Res<CaseRegistry>,
    mut case_board: ResMut<CaseBoard>,
    mut case_failed: EventWriter<CaseFailedEvent>,
) {
    let active_cases = std::mem::take(&mut case_board.active);
    let mut remaining_active = Vec::with_capacity(active_cases.len());
    let mut cold_cases = Vec::new();

    for mut active_case in active_cases {
        let Some(case_def) = registry.get(&active_case.case_id) else {
            remaining_active.push(active_case);
            continue;
        };

        let is_expired = case_def
            .time_limit_shifts
            .is_some_and(|time_limit| active_case.shifts_elapsed >= time_limit);

        if is_expired {
            active_case.status = CaseStatus::Cold;
            case_failed.send(CaseFailedEvent {
                case_id: active_case.case_id.clone(),
                reason: format!(
                    "Case went cold after {} shifts.",
                    case_def.time_limit_shifts.unwrap_or_default()
                ),
            });
            cold_cases.push(active_case.case_id);
            continue;
        }

        remaining_active.push(active_case);
    }

    case_board.active = remaining_active;
    case_board.cold.extend(cold_cases);
}

fn refresh_available_cases(
    mut shift_end_events: EventReader<ShiftEndEvent>,
    mut promotion_events: EventReader<PromotionEvent>,
    clock: Res<ShiftClock>,
    registry: Res<CaseRegistry>,
    mut case_board: ResMut<CaseBoard>,
) {
    let shift_refresh_requested = shift_end_events.read().next().is_some();
    let mut refresh_rank = clock.rank;
    let mut promotion_refresh_requested = false;

    for promotion_event in promotion_events.read() {
        promotion_refresh_requested = true;
        if promotion_event.new_rank > refresh_rank {
            refresh_rank = promotion_event.new_rank;
        }
    }

    if !shift_refresh_requested && !promotion_refresh_requested {
        return;
    }

    replenish_available_cases(&registry.defs, &mut case_board, refresh_rank);
}

fn replenish_available_cases(case_defs: &[CaseDef], case_board: &mut CaseBoard, rank: Rank) {
    let mut tracked_case_ids: HashSet<String> = case_board
        .available
        .iter()
        .cloned()
        .chain(case_board.solved.iter().cloned())
        .chain(case_board.cold.iter().cloned())
        .chain(case_board.failed.iter().cloned())
        .collect();

    tracked_case_ids.extend(
        case_board
            .active
            .iter()
            .map(|active_case| active_case.case_id.clone()),
    );

    for case_def in case_defs {
        if case_def.rank_required > rank {
            continue;
        }

        if tracked_case_ids.insert(case_def.id.clone()) {
            case_board.available.push(case_def.id.clone());
        }
    }
}

struct CaseDefSpec<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
    rank_required: Rank,
    evidence_required: &'a [&'a str],
    witnesses: &'a [&'a str],
    suspects: &'a [&'a str],
    scenes: &'a [MapId],
    time_limit_shifts: Option<u8>,
    reward_xp: u32,
    reward_reputation: i32,
    reward_gold: i32,
    difficulty: u8,
    is_major: bool,
}

macro_rules! case_def {
    (
        $id:expr,
        $name:expr,
        $description:expr,
        $rank_required:expr,
        $evidence_required:expr,
        $witnesses:expr,
        $suspects:expr,
        $scenes:expr,
        $time_limit_shifts:expr,
        $reward_xp:expr,
        $reward_reputation:expr,
        $reward_gold:expr,
        $difficulty:expr,
        $is_major:expr $(,)?
    ) => {
        CaseDef::from(CaseDefSpec {
            id: $id,
            name: $name,
            description: $description,
            rank_required: $rank_required,
            evidence_required: $evidence_required,
            witnesses: $witnesses,
            suspects: $suspects,
            scenes: $scenes,
            time_limit_shifts: $time_limit_shifts,
            reward_xp: $reward_xp,
            reward_reputation: $reward_reputation,
            reward_gold: $reward_gold,
            difficulty: $difficulty,
            is_major: $is_major,
        })
    };
}

fn build_case_registry() -> Vec<CaseDef> {
    vec![
        case_def!(
            "patrol_001_petty_theft",
            "Petty Theft at General Store",
            "The general store owner wants a quick answer about missing cash and a familiar face near the till.",
            Rank::PatrolOfficer,
            &["fingerprint", "witness_statement"],
            &["rita_gomez"],
            &["marcus_cole"],
            &[MapId::Downtown],
            Some(8),
            30,
            5,
            50,
            2,
            false,
        ),
        case_def!(
            "patrol_002_vandalism",
            "Park Vandalism",
            "Fresh damage in the park needs documenting before weather or foot traffic wipes the trail away.",
            Rank::PatrolOfficer,
            &["photo_of_scene", "footprint"],
            &["father_brennan"],
            &[],
            &[MapId::ForestPark],
            Some(10),
            30,
            5,
            50,
            2,
            false,
        ),
        case_def!(
            "patrol_003_noise",
            "Noise Complaint",
            "Neighbors want a late-night disturbance handled before the block turns on itself.",
            Rank::PatrolOfficer,
            &["witness_statement"],
            &[],
            &[],
            &[MapId::ResidentialNorth],
            Some(4),
            15,
            3,
            25,
            1,
            false,
        ),
        case_def!(
            "patrol_004_lost_pet",
            "Lost Pet Report",
            "A worried resident needs help tracing where a missing pet ran after bolting from home.",
            Rank::PatrolOfficer,
            &["photo_of_scene", "tip_off"],
            &[],
            &[],
            &[MapId::ResidentialSouth, MapId::ForestPark],
            Some(14),
            15,
            5,
            25,
            1,
            false,
        ),
        case_def!(
            "patrol_005_shoplifting",
            "Shoplifting in Progress",
            "A live shoplifting call needs a fast response before the suspect disappears into the lunch crowd.",
            Rank::PatrolOfficer,
            &["security_footage", "witness_statement"],
            &["rita_gomez"],
            &[],
            &[MapId::Downtown],
            Some(2),
            45,
            5,
            75,
            3,
            false,
        ),
        case_def!(
            "patrol_006_car_breakin",
            "Car Break-In",
            "A smashed lock in the precinct lot means one of your own neighbors expects answers.",
            Rank::PatrolOfficer,
            &["fingerprint", "broken_lock", "photo_of_scene"],
            &[],
            &[],
            &[MapId::PrecinctExterior],
            Some(8),
            45,
            5,
            75,
            3,
            false,
        ),
        case_def!(
            "patrol_007_graffiti",
            "Graffiti Investigation",
            "Fresh tags across the industrial district point to someone who knows exactly when the patrol routes thin out.",
            Rank::PatrolOfficer,
            &["photo_of_scene", "tire_track"],
            &["ghost_tipster"],
            &[],
            &[MapId::IndustrialDistrict],
            Some(12),
            30,
            3,
            50,
            2,
            false,
        ),
        case_def!(
            "patrol_008_trespassing",
            "Trespassing at Rail Yard",
            "The rail yard foreman wants proof of who crossed the fence before the next freight run comes through.",
            Rank::PatrolOfficer,
            &["footprint", "witness_statement", "photo_of_scene"],
            &[],
            &[],
            &[MapId::IndustrialDistrict],
            Some(6),
            45,
            5,
            75,
            3,
            false,
        ),
        case_def!(
            "detective_001_burglary",
            "Residential Burglary",
            "A quiet home invasion left just enough physical and paper trail to chase if you move before the neighborhood clams up.",
            Rank::Detective,
            &["fingerprint", "broken_lock", "receipt", "witness_statement"],
            &[],
            &["marcus_cole"],
            &[MapId::ResidentialNorth],
            Some(10),
            75,
            10,
            125,
            5,
            false,
        ),
        case_def!(
            "detective_002_assault",
            "Downtown Assault",
            "An assault downtown has medical evidence waiting at the hospital and bystanders already second-guessing what they saw.",
            Rank::Detective,
            &["blood_sample", "witness_statement", "security_footage"],
            &["dr_okafor"],
            &[],
            &[MapId::Downtown, MapId::Hospital],
            Some(8),
            75,
            10,
            125,
            5,
            false,
        ),
        case_def!(
            "detective_003_fraud",
            "Bank Fraud Scheme",
            "A tidy fraud complaint starts opening into a larger paper trail that could embarrass half of downtown.",
            Rank::Detective,
            &["bank_statement", "phone_record", "receipt", "motive_document"],
            &["mayor_aldridge"],
            &[],
            &[MapId::Downtown],
            Some(14),
            90,
            12,
            150,
            6,
            false,
        ),
        case_def!(
            "detective_004_missing",
            "Missing Person",
            "A disappearance stretches from apartments to the park and highway, forcing you to stitch together the last clean timeline.",
            Rank::Detective,
            &[
                "phone_record",
                "witness_statement",
                "photo_of_scene",
                "clothing_fiber",
            ],
            &["nadia_park"],
            &[],
            &[MapId::ResidentialSouth, MapId::ForestPark, MapId::Highway],
            Some(12),
            90,
            15,
            150,
            6,
            true,
        ),
        case_def!(
            "detective_005_arson",
            "Warehouse Arson",
            "The warehouse fire looks accidental from the street, but the scene and forecast records tell a different story.",
            Rank::Detective,
            &[
                "photo_of_scene",
                "weather_log",
                "forensic_report",
                "witness_statement",
            ],
            &["ghost_tipster"],
            &[],
            &[MapId::IndustrialDistrict],
            Some(10),
            90,
            12,
            150,
            6,
            false,
        ),
        case_def!(
            "detective_006_drugs",
            "Drug Possession Ring",
            "What looks like a routine possession bust may actually connect a few regulars who never appear in the same report twice.",
            Rank::Detective,
            &["tip_off", "security_footage", "phone_record"],
            &["ghost_tipster", "lucia_vega"],
            &[],
            &[MapId::Downtown, MapId::IndustrialDistrict],
            Some(8),
            75,
            10,
            125,
            5,
            false,
        ),
        case_def!(
            "detective_007_hitrun",
            "Hit and Run",
            "A violent collision on the highway left just enough trace evidence to catch the driver before repairs hide the damage.",
            Rank::Detective,
            &[
                "traffic_cam",
                "tire_track",
                "witness_statement",
                "blood_sample",
            ],
            &["dr_okafor"],
            &[],
            &[MapId::Highway],
            Some(6),
            75,
            10,
            125,
            5,
            false,
        ),
        case_def!(
            "detective_008_blackmail",
            "Blackmail Case",
            "A blackmail scheme is squeezing a powerful target, and every message or bank record points to something uglier underneath.",
            Rank::Detective,
            &["letter", "phone_record", "bank_statement", "motive_document"],
            &["nadia_park"],
            &["mayor_aldridge"],
            &[MapId::Downtown, MapId::CourtHouse],
            Some(10),
            105,
            15,
            175,
            7,
            false,
        ),
        case_def!(
            "sergeant_001_homicide",
            "Downtown Homicide",
            "A downtown killing forces the precinct into a full-scene investigation with public pressure rising every shift.",
            Rank::Sergeant,
            &[
                "blood_sample",
                "dna_match",
                "weapon",
                "witness_statement",
                "photo_of_scene",
                "motive_document",
            ],
            &["dr_okafor", "ghost_tipster"],
            &[],
            &[MapId::Downtown, MapId::Hospital, MapId::CrimeSceneTemplate],
            Some(14),
            120,
            20,
            200,
            8,
            true,
        ),
        case_def!(
            "sergeant_002_kidnapping",
            "Child Kidnapping",
            "A child abduction leaves only a narrow window to connect witness recollections with vehicle movement across town.",
            Rank::Sergeant,
            &[
                "phone_record",
                "witness_statement",
                "traffic_cam",
                "clothing_fiber",
                "tire_track",
            ],
            &["father_brennan", "nadia_park"],
            &[],
            &[MapId::ResidentialNorth, MapId::Highway, MapId::ForestPark],
            Some(6),
            120,
            25,
            200,
            8,
            false,
        ),
        case_def!(
            "sergeant_003_theft_ring",
            "Organized Theft Ring",
            "Separate theft reports are starting to line up into something coordinated, profitable, and much harder to pin on one suspect.",
            Rank::Sergeant,
            &[
                "security_footage",
                "phone_record",
                "receipt",
                "financial_motive",
                "relationship_map",
            ],
            &["rita_gomez", "ghost_tipster"],
            &["marcus_cole"],
            &[MapId::Downtown, MapId::IndustrialDistrict],
            Some(16),
            105,
            18,
            175,
            7,
            false,
        ),
        case_def!(
            "sergeant_004_corruption",
            "Police Corruption",
            "The evidence points back inside the precinct, where every new lead risks turning coworkers defensive or dangerous.",
            Rank::Sergeant,
            &[
                "bank_statement",
                "phone_record",
                "letter",
                "behavioral_pattern",
                "motive_document",
            ],
            &["lucia_vega", "nadia_park"],
            &["officer_chen"],
            &[MapId::PrecinctInterior, MapId::CourtHouse],
            Some(20),
            120,
            25,
            200,
            8,
            false,
        ),
        case_def!(
            "sergeant_005_cold_case",
            "Cold Case Revival",
            "A shelved investigation finally has modern evidence to reopen it, if you can make the old story stand up under fresh scrutiny.",
            Rank::Sergeant,
            &[
                "dna_match",
                "digital_forensics",
                "witness_statement",
                "opportunity_timeline",
            ],
            &["dr_okafor"],
            &[],
            &[MapId::CrimeSceneTemplate, MapId::Hospital],
            None,
            105,
            18,
            175,
            7,
            false,
        ),
        case_def!(
            "sergeant_006_serial_vandal",
            "Serial Vandal Pattern",
            "What the town sees as random damage is starting to read like one deliberate pattern across multiple neighborhoods.",
            Rank::Sergeant,
            &[
                "photo_of_scene",
                "behavioral_pattern",
                "traffic_cam",
                "witness_statement",
                "tool_mark",
            ],
            &["father_brennan"],
            &[],
            &[MapId::ResidentialNorth, MapId::ResidentialSouth, MapId::ForestPark],
            Some(18),
            90,
            15,
            150,
            6,
            false,
        ),
        case_def!(
            "lieutenant_001_serial",
            "Serial Killer Investigation",
            "Multiple scenes now point to one killer, and every witness account matters because the pattern is tightening around the town.",
            Rank::Lieutenant,
            &[
                "blood_sample",
                "dna_match",
                "ballistic_report",
                "behavioral_pattern",
                "relationship_map",
                "witness_statement",
            ],
            &["dr_okafor", "ghost_tipster", "nadia_park"],
            &[],
            &[
                MapId::CrimeSceneTemplate,
                MapId::Hospital,
                MapId::Downtown,
                MapId::ForestPark,
            ],
            Some(20),
            150,
            30,
            250,
            10,
            true,
        ),
        case_def!(
            "lieutenant_002_conspiracy",
            "City-Wide Conspiracy",
            "Financial records, private letters, and digital traces suggest the town's power structure has been coordinating for years.",
            Rank::Lieutenant,
            &[
                "bank_statement",
                "phone_record",
                "digital_forensics",
                "letter",
                "financial_motive",
                "opportunity_timeline",
                "relationship_map",
            ],
            &["mayor_aldridge", "lucia_vega", "nadia_park", "ghost_tipster"],
            &[],
            &[MapId::Downtown, MapId::CourtHouse, MapId::PrecinctInterior],
            Some(28),
            150,
            30,
            250,
            10,
            true,
        ),
        case_def!(
            "lieutenant_003_final",
            "The Final Case",
            "The rookie's entire arc comes due in one last investigation that threads through every institution you've had to trust or challenge.",
            Rank::Lieutenant,
            &[
                "dna_match",
                "confession",
                "witness_statement",
                "ballistic_report",
                "motive_document",
                "relationship_map",
            ],
            &["captain_torres", "det_vasquez"],
            &[],
            &[
                MapId::CrimeSceneTemplate,
                MapId::PrecinctInterior,
                MapId::Downtown,
                MapId::CourtHouse,
            ],
            None,
            200,
            50,
            500,
            10,
            true,
        ),
    ]
}

impl From<CaseDefSpec<'_>> for CaseDef {
    fn from(spec: CaseDefSpec<'_>) -> Self {
        Self {
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            description: spec.description.to_string(),
            rank_required: spec.rank_required,
            evidence_required: evidence_ids(spec.evidence_required),
            witnesses: npc_ids(spec.witnesses),
            suspects: npc_ids(spec.suspects),
            scenes: spec.scenes.to_vec(),
            time_limit_shifts: spec.time_limit_shifts,
            reward_xp: spec.reward_xp,
            reward_reputation: spec.reward_reputation,
            reward_gold: spec.reward_gold,
            difficulty: spec.difficulty,
            is_major: spec.is_major,
        }
    }
}

fn evidence_ids(ids: &[&str]) -> Vec<EvidenceId> {
    ids.iter().map(|id| (*id).to_string()).collect()
}

fn npc_ids(ids: &[&str]) -> Vec<NpcId> {
    ids.iter().map(|id| (*id).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;

    use crate::shared::{CASE_CLOSE_BONUS_MULTIPLIER, MAX_ACTIVE_CASES, XP_CASE_MULTIPLIER};

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
        app.init_resource::<ShiftClock>();
        app.init_resource::<CaseBoard>();
        app.add_event::<CaseAssignedEvent>();
        app.add_event::<CaseSolvedEvent>();
        app.add_event::<CaseFailedEvent>();
        app.add_event::<EvidenceCollectedEvent>();
        app.add_event::<ShiftEndEvent>();
        app.add_event::<PromotionEvent>();
        app.add_plugins(CasesPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    #[test]
    fn startup_populates_all_twenty_five_cases_and_initial_patrol_board() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let registry = app.world().resource::<CaseRegistry>();
        let board = app.world().resource::<CaseBoard>();

        assert_eq!(registry.defs.len(), 25);
        assert_eq!(
            registry
                .defs
                .iter()
                .filter(|case_def| case_def.rank_required == Rank::PatrolOfficer)
                .count(),
            8
        );
        assert_eq!(
            registry
                .defs
                .iter()
                .filter(|case_def| case_def.rank_required == Rank::Detective)
                .count(),
            8
        );
        assert_eq!(
            registry
                .defs
                .iter()
                .filter(|case_def| case_def.rank_required == Rank::Sergeant)
                .count(),
            6
        );
        assert_eq!(
            registry
                .defs
                .iter()
                .filter(|case_def| case_def.rank_required == Rank::Lieutenant)
                .count(),
            3
        );
        assert_eq!(board.available.len(), 8);
        assert!(board
            .available
            .iter()
            .all(|case_id| case_id.starts_with("patrol_")));
    }

    #[test]
    fn case_assigned_event_moves_case_from_available_to_active() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<CaseAssignedEvent>>()
            .send(CaseAssignedEvent {
                case_id: "patrol_001_petty_theft".to_string(),
            });

        app.update();

        let board = app.world().resource::<CaseBoard>();
        assert!(!board
            .available
            .iter()
            .any(|case_id| case_id == "patrol_001_petty_theft"));
        assert_eq!(board.active.len(), 1);
        assert_eq!(board.active[0].case_id, "patrol_001_petty_theft");
        assert_eq!(board.active[0].status, CaseStatus::Active);
    }

    #[test]
    fn max_active_cases_prevents_fourth_assignment() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        for case_id in [
            "patrol_001_petty_theft",
            "patrol_002_vandalism",
            "patrol_003_noise",
            "patrol_004_lost_pet",
        ] {
            app.world_mut()
                .resource_mut::<Events<CaseAssignedEvent>>()
                .send(CaseAssignedEvent {
                    case_id: case_id.to_string(),
                });
        }

        app.update();

        let board = app.world().resource::<CaseBoard>();
        assert_eq!(board.active.len(), MAX_ACTIVE_CASES);
        assert!(board
            .available
            .iter()
            .any(|case_id| case_id == "patrol_004_lost_pet"));
    }

    #[test]
    fn evidence_collection_updates_only_the_matching_active_case() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        for case_id in ["patrol_001_petty_theft", "patrol_002_vandalism"] {
            app.world_mut()
                .resource_mut::<Events<CaseAssignedEvent>>()
                .send(CaseAssignedEvent {
                    case_id: case_id.to_string(),
                });
        }

        app.update();

        app.world_mut()
            .resource_mut::<Events<EvidenceCollectedEvent>>()
            .send(EvidenceCollectedEvent {
                evidence_id: "fingerprint".to_string(),
                case_id: "patrol_001_petty_theft".to_string(),
                quality: 0.5,
            });

        app.update();

        let board = app.world().resource::<CaseBoard>();
        let theft_case = board
            .active
            .iter()
            .find(|active_case| active_case.case_id == "patrol_001_petty_theft")
            .unwrap();
        let vandalism_case = board
            .active
            .iter()
            .find(|active_case| active_case.case_id == "patrol_002_vandalism")
            .unwrap();

        assert_eq!(theft_case.status, CaseStatus::Investigating);
        assert_eq!(
            theft_case.evidence_collected,
            vec!["fingerprint".to_string()]
        );
        assert!(vandalism_case.evidence_collected.is_empty());
    }

    #[test]
    fn case_expires_when_shifts_elapsed_reaches_time_limit() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<CaseAssignedEvent>>()
            .send(CaseAssignedEvent {
                case_id: "patrol_005_shoplifting".to_string(),
            });

        app.update();

        for shift_number in [1, 2] {
            app.world_mut()
                .resource_mut::<Events<ShiftEndEvent>>()
                .send(ShiftEndEvent {
                    shift_number,
                    cases_progressed: 0,
                    evidence_collected: 0,
                    xp_earned: 0,
                });
        }

        app.update();

        let failed_events = app
            .world_mut()
            .resource_mut::<Events<CaseFailedEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let board = app.world().resource::<CaseBoard>();

        assert!(board.active.is_empty());
        assert!(board
            .cold
            .iter()
            .any(|case_id| case_id == "patrol_005_shoplifting"));
        assert_eq!(failed_events.len(), 1);
        assert_eq!(failed_events[0].case_id, "patrol_005_shoplifting");
    }

    #[test]
    fn case_solved_event_emits_expected_rewards() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<CaseAssignedEvent>>()
            .send(CaseAssignedEvent {
                case_id: "patrol_006_car_breakin".to_string(),
            });

        app.update();

        {
            let mut board = app.world_mut().resource_mut::<CaseBoard>();
            let active_case = board
                .active
                .iter_mut()
                .find(|active_case| active_case.case_id == "patrol_006_car_breakin")
                .unwrap();
            active_case.status = CaseStatus::EvidenceComplete;
        }

        app.world_mut()
            .resource_mut::<Events<CaseCloseRequestedEvent>>()
            .send(CaseCloseRequestedEvent {
                case_id: "patrol_006_car_breakin".to_string(),
            });

        app.update();

        let solved_events = app
            .world_mut()
            .resource_mut::<Events<CaseSolvedEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let board = app.world().resource::<CaseBoard>();

        assert_eq!(solved_events.len(), 1);
        assert_eq!(solved_events[0].case_id, "patrol_006_car_breakin");
        assert_eq!(solved_events[0].xp_reward, 3 * XP_CASE_MULTIPLIER);
        assert_eq!(
            solved_events[0].gold_reward,
            3 * CASE_CLOSE_BONUS_MULTIPLIER
        );
        assert_eq!(solved_events[0].reputation_reward, 5);
        assert!(board
            .solved
            .iter()
            .any(|case_id| case_id == "patrol_006_car_breakin"));
        assert_eq!(board.total_cases_solved, 1);
    }

    #[test]
    fn rank_gated_cases_refresh_when_rank_increases() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        assert!(!app
            .world()
            .resource::<CaseBoard>()
            .available
            .iter()
            .any(|case_id| case_id.starts_with("detective_")));

        app.world_mut().resource_mut::<ShiftClock>().rank = Rank::Detective;
        app.world_mut()
            .resource_mut::<Events<PromotionEvent>>()
            .send(PromotionEvent {
                new_rank: Rank::Detective,
            });

        app.update();

        let board = app.world().resource::<CaseBoard>();
        assert!(board
            .available
            .iter()
            .any(|case_id| case_id == "detective_001_burglary"));
        assert!(!board
            .available
            .iter()
            .any(|case_id| case_id.starts_with("sergeant_")));
    }

    #[test]
    fn final_case_keeps_authored_table_rewards() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let registry = app.world().resource::<CaseRegistry>();
        let final_case = registry.get("lieutenant_003_final").unwrap();

        assert_eq!(final_case.reward_xp, 200);
        assert_eq!(final_case.reward_gold, 500);
        assert!(final_case.is_major);
    }
}
