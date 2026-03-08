use bevy::prelude::*;
use std::collections::HashMap;

use crate::shared::{
    CaseId, EvidenceCategory, EvidenceCollectedEvent, EvidenceId, EvidenceLocker, EvidencePiece,
    EvidenceProcessedEvent, EvidenceProcessingState, GameState, MapId, PlayerState, ShiftClock,
    ShiftEndEvent, UpdatePhase, Weather, EVIDENCE_BASE_QUALITY, EVIDENCE_MAX_QUALITY,
    EVIDENCE_SKILL_BONUS, EVIDENCE_WEATHER_PENALTY,
};

const WAVE_TWO_SKILL_LEVEL: u32 = 0;

#[derive(Debug, Clone, PartialEq, Eq)]
struct EvidenceDef {
    id: EvidenceId,
    name: String,
    category: EvidenceCategory,
    description: String,
}

#[derive(Resource, Debug, Clone, Default)]
struct EvidenceRegistry {
    definitions: HashMap<EvidenceId, EvidenceDef>,
}

pub struct EvidencePlugin;

impl Plugin for EvidencePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EvidenceRegistry>()
            .add_systems(Startup, populate_evidence_registry)
            .add_systems(
                Update,
                (collect_evidence, process_evidence)
                    .chain()
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn populate_evidence_registry(mut registry: ResMut<EvidenceRegistry>) {
    registry.definitions = evidence_definitions()
        .into_iter()
        .map(|definition| (definition.id.clone(), definition))
        .collect();
}

fn collect_evidence(
    mut collected_events: EventReader<EvidenceCollectedEvent>,
    registry: Res<EvidenceRegistry>,
    clock: Res<ShiftClock>,
    player_state: Res<PlayerState>,
    mut locker: ResMut<EvidenceLocker>,
) {
    let quality = calculate_evidence_quality(WAVE_TWO_SKILL_LEVEL, &clock);
    let collected_map = collected_map_from_player(&player_state);

    for event in collected_events.read() {
        let Some(definition) = registry.definitions.get(&event.evidence_id) else {
            continue;
        };

        locker.pieces.push(EvidencePiece {
            id: definition.id.clone(),
            name: definition.name.clone(),
            category: definition.category,
            description: definition.description.clone(),
            quality,
            linked_case: linked_case(&event.case_id),
            processing_state: EvidenceProcessingState::Raw,
            collected_shift: clock.shift_number,
            collected_map,
        });
    }
}

fn process_evidence(
    mut shift_end_events: EventReader<ShiftEndEvent>,
    mut locker: ResMut<EvidenceLocker>,
    mut processed_events: EventWriter<EvidenceProcessedEvent>,
) {
    let shift_end_count = shift_end_events.read().count();

    if shift_end_count == 0 {
        return;
    }

    for _ in 0..shift_end_count {
        let mut processed_ids = Vec::new();

        for piece in locker
            .pieces
            .iter_mut()
            .filter(|piece| piece.processing_state == EvidenceProcessingState::Processing)
        {
            piece.processing_state = EvidenceProcessingState::Analyzed;
            processed_ids.push(piece.id.clone());
        }

        for evidence_id in processed_ids {
            processed_events.send(EvidenceProcessedEvent { evidence_id });
        }
    }
}

pub fn start_processing_evidence(locker: &mut EvidenceLocker, evidence_id: &EvidenceId) -> usize {
    let mut started = 0;

    for piece in locker
        .pieces
        .iter_mut()
        .filter(|piece| piece.id == *evidence_id)
        .filter(|piece| piece.processing_state == EvidenceProcessingState::Raw)
    {
        piece.processing_state = EvidenceProcessingState::Processing;
        started += 1;
    }

    started
}

fn calculate_evidence_quality(skill_level: u32, clock: &ShiftClock) -> f32 {
    let weather_penalty = active_penalty_conditions(clock) as f32 * EVIDENCE_WEATHER_PENALTY;
    let quality =
        EVIDENCE_BASE_QUALITY + skill_level as f32 * EVIDENCE_SKILL_BONUS - weather_penalty;

    quality.min(EVIDENCE_MAX_QUALITY)
}

fn active_penalty_conditions(clock: &ShiftClock) -> u8 {
    let mut conditions = 0;

    if matches!(clock.weather, Weather::Rainy | Weather::Foggy) {
        conditions += 1;
    }

    if is_night_hour(clock.hour) {
        conditions += 1;
    }

    conditions
}

fn is_night_hour(hour: u8) -> bool {
    hour >= 22 || hour < 6
}

fn linked_case(case_id: &CaseId) -> Option<CaseId> {
    (!case_id.is_empty()).then_some(case_id.clone())
}

fn collected_map_from_player(player_state: &PlayerState) -> MapId {
    player_state.position_map
}

fn evidence_definitions() -> Vec<EvidenceDef> {
    vec![
        evidence_def(
            "fingerprint",
            "Fingerprint",
            EvidenceCategory::Physical,
            "Fingerprint lifted from surface",
        ),
        evidence_def(
            "footprint",
            "Footprint",
            EvidenceCategory::Physical,
            "Shoe impression found at scene",
        ),
        evidence_def(
            "weapon",
            "Weapon",
            EvidenceCategory::Physical,
            "Weapon recovered from scene",
        ),
        evidence_def(
            "clothing_fiber",
            "Clothing Fiber",
            EvidenceCategory::Physical,
            "Fiber sample from clothing",
        ),
        evidence_def(
            "tool_mark",
            "Tool Mark",
            EvidenceCategory::Physical,
            "Tool impression on lock or surface",
        ),
        evidence_def(
            "receipt",
            "Receipt",
            EvidenceCategory::Documentary,
            "Transaction receipt",
        ),
        evidence_def(
            "letter",
            "Letter",
            EvidenceCategory::Documentary,
            "Written correspondence",
        ),
        evidence_def(
            "phone_record",
            "Phone Record",
            EvidenceCategory::Documentary,
            "Call log or text messages",
        ),
        evidence_def(
            "security_footage",
            "Security Footage",
            EvidenceCategory::Documentary,
            "Security camera recording",
        ),
        evidence_def(
            "bank_statement",
            "Bank Statement",
            EvidenceCategory::Documentary,
            "Financial account records",
        ),
        evidence_def(
            "witness_statement",
            "Witness Statement",
            EvidenceCategory::Testimonial,
            "Sworn witness account",
        ),
        evidence_def(
            "alibi",
            "Alibi",
            EvidenceCategory::Testimonial,
            "Alibi documentation",
        ),
        evidence_def(
            "confession",
            "Confession",
            EvidenceCategory::Testimonial,
            "Suspect confession",
        ),
        evidence_def(
            "tip_off",
            "Tip-Off",
            EvidenceCategory::Testimonial,
            "Anonymous tip",
        ),
        evidence_def(
            "recording_911",
            "911 Recording",
            EvidenceCategory::Testimonial,
            "Emergency call recording",
        ),
        evidence_def(
            "blood_sample",
            "Blood Sample",
            EvidenceCategory::Forensic,
            "Blood evidence",
        ),
        evidence_def(
            "dna_match",
            "DNA Match",
            EvidenceCategory::Forensic,
            "DNA analysis result",
        ),
        evidence_def(
            "ballistic_report",
            "Ballistic Report",
            EvidenceCategory::Forensic,
            "Ballistics analysis",
        ),
        evidence_def(
            "toxicology",
            "Toxicology",
            EvidenceCategory::Forensic,
            "Toxicology screen results",
        ),
        evidence_def(
            "digital_forensics",
            "Digital Forensics",
            EvidenceCategory::Forensic,
            "Digital device analysis",
        ),
        evidence_def(
            "photo_of_scene",
            "Photo of Scene",
            EvidenceCategory::Environmental,
            "Crime scene photograph",
        ),
        evidence_def(
            "weather_log",
            "Weather Log",
            EvidenceCategory::Environmental,
            "Weather conditions record",
        ),
        evidence_def(
            "traffic_cam",
            "Traffic Cam",
            EvidenceCategory::Environmental,
            "Traffic camera footage",
        ),
        evidence_def(
            "broken_lock",
            "Broken Lock",
            EvidenceCategory::Environmental,
            "Damaged lock or entry point",
        ),
        evidence_def(
            "tire_track",
            "Tire Track",
            EvidenceCategory::Environmental,
            "Tire impression",
        ),
        evidence_def(
            "motive_document",
            "Motive Document",
            EvidenceCategory::Circumstantial,
            "Evidence of motive",
        ),
        evidence_def(
            "opportunity_timeline",
            "Opportunity Timeline",
            EvidenceCategory::Circumstantial,
            "Timeline of suspect movements",
        ),
        evidence_def(
            "behavioral_pattern",
            "Behavioral Pattern",
            EvidenceCategory::Circumstantial,
            "Pattern of behavior analysis",
        ),
        evidence_def(
            "financial_motive",
            "Financial Motive",
            EvidenceCategory::Circumstantial,
            "Financial gain evidence",
        ),
        evidence_def(
            "relationship_map",
            "Relationship Map",
            EvidenceCategory::Circumstantial,
            "Relationship connections diagram",
        ),
    ]
}

fn evidence_def(
    id: &str,
    name: &str,
    category: EvidenceCategory,
    description: &str,
) -> EvidenceDef {
    EvidenceDef {
        id: id.to_string(),
        name: name.to_string(),
        category,
        description: description.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;

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
        app.init_resource::<PlayerState>();
        app.init_resource::<EvidenceLocker>();
        app.add_event::<EvidenceCollectedEvent>();
        app.add_event::<EvidenceProcessedEvent>();
        app.add_event::<ShiftEndEvent>();
        app.add_plugins(EvidencePlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    fn send_collection_event(app: &mut App, evidence_id: &str, case_id: &str) {
        app.world_mut()
            .resource_mut::<Events<EvidenceCollectedEvent>>()
            .send(EvidenceCollectedEvent {
                evidence_id: evidence_id.to_string(),
                case_id: case_id.to_string(),
                quality: 0.0,
            });
    }

    #[test]
    fn registers_all_thirty_evidence_types() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        let registry = app.world().resource::<EvidenceRegistry>();
        assert_eq!(registry.definitions.len(), 30);

        for (category, expected_count) in [
            (EvidenceCategory::Physical, 5),
            (EvidenceCategory::Documentary, 5),
            (EvidenceCategory::Testimonial, 5),
            (EvidenceCategory::Forensic, 5),
            (EvidenceCategory::Environmental, 5),
            (EvidenceCategory::Circumstantial, 5),
        ] {
            let actual_count = registry
                .definitions
                .values()
                .filter(|definition| definition.category == category)
                .count();

            assert_eq!(actual_count, expected_count);
        }
    }

    #[test]
    fn evidence_quality_is_base_value_in_clear_weather_with_zero_skill() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.weather = Weather::Clear;
            clock.hour = 12;
        }
        app.world_mut().resource_mut::<PlayerState>().position_map = MapId::Downtown;

        send_collection_event(&mut app, "fingerprint", "patrol_001_petty_theft");
        app.update();

        let locker = app.world().resource::<EvidenceLocker>();
        let piece = locker.pieces.last().expect("evidence should be collected");
        assert!((piece.quality - EVIDENCE_BASE_QUALITY).abs() < f32::EPSILON);
        assert_eq!(piece.processing_state, EvidenceProcessingState::Raw);
        assert_eq!(piece.collected_shift, 1);
        assert_eq!(piece.collected_map, MapId::Downtown);
    }

    #[test]
    fn evidence_quality_applies_rain_penalty() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.weather = Weather::Rainy;
            clock.hour = 12;
        }

        send_collection_event(&mut app, "receipt", "patrol_002_vandalism");
        app.update();

        let locker = app.world().resource::<EvidenceLocker>();
        let piece = locker.pieces.last().expect("evidence should be collected");
        assert!((piece.quality - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn evidence_quality_caps_at_maximum() {
        let mut clock = ShiftClock::default();
        clock.weather = Weather::Clear;
        clock.hour = 12;

        let quality = calculate_evidence_quality(20, &clock);

        assert!((quality - EVIDENCE_MAX_QUALITY).abs() < f32::EPSILON);
    }

    #[test]
    fn processing_transitions_from_raw_to_processing_to_analyzed_over_one_shift() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        send_collection_event(&mut app, "weapon", "detective_001_burglary");
        app.update();

        let evidence_id = "weapon".to_string();
        {
            let mut locker = app.world_mut().resource_mut::<EvidenceLocker>();
            assert_eq!(start_processing_evidence(&mut locker, &evidence_id), 1);
            assert_eq!(
                locker.pieces[0].processing_state,
                EvidenceProcessingState::Processing
            );
        }

        app.world_mut()
            .resource_mut::<Events<ShiftEndEvent>>()
            .send(ShiftEndEvent {
                shift_number: 1,
                cases_progressed: 0,
                evidence_collected: 1,
                xp_earned: 0,
            });
        app.update();

        let locker = app.world().resource::<EvidenceLocker>();
        assert_eq!(
            locker.pieces[0].processing_state,
            EvidenceProcessingState::Analyzed
        );
    }

    #[test]
    fn evidence_processed_event_emits_when_processing_completes() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        send_collection_event(&mut app, "dna_match", "sergeant_001_homicide");
        app.update();

        {
            let mut locker = app.world_mut().resource_mut::<EvidenceLocker>();
            assert_eq!(
                start_processing_evidence(&mut locker, &"dna_match".to_string()),
                1
            );
        }

        app.world_mut()
            .resource_mut::<Events<ShiftEndEvent>>()
            .send(ShiftEndEvent {
                shift_number: 1,
                cases_progressed: 0,
                evidence_collected: 1,
                xp_earned: 0,
            });
        app.update();

        let events = app.world().resource::<Events<EvidenceProcessedEvent>>();
        let mut reader = events.get_cursor();
        let processed_ids: Vec<_> = reader
            .read(events)
            .map(|event| event.evidence_id.clone())
            .collect();

        assert_eq!(processed_ids, vec!["dna_match".to_string()]);
    }

    #[test]
    fn collected_evidence_links_to_the_source_case() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        send_collection_event(&mut app, "broken_lock", "detective_005_arson");
        app.update();

        let locker = app.world().resource::<EvidenceLocker>();
        let piece = locker.pieces.last().expect("evidence should be collected");
        assert_eq!(piece.linked_case.as_deref(), Some("detective_005_arson"));
    }
}
