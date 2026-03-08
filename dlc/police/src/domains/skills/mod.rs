use bevy::prelude::*;

use crate::shared::{
    CaseSolvedEvent, DispatchResolvedEvent, EvidenceCollectedEvent, GameState, NpcTrustChangeEvent,
    SkillPointSpentEvent, SkillTree, Skills, UpdatePhase, XpGainedEvent, SKILL_POINT_INTERVAL,
    XP_PER_EVIDENCE, XP_PER_PATROL_EVENT,
};

const MAX_SKILL_LEVEL: u8 = 5;
const COMMUNITY_TRUST_BONUS: i32 = 5;
const COMMUNITY_TRUST_NPC_IDS: [&str; 8] = [
    "mayor_aldridge",
    "dr_okafor",
    "rita_gomez",
    "father_brennan",
    "ghost_tipster",
    "nadia_park",
    "marcus_cole",
    "lucia_vega",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerkDef {
    pub tree: SkillTree,
    pub level: u8,
    pub name: &'static str,
    pub description: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
struct SkillUnlockedEvent {
    tree: SkillTree,
    level: u8,
}

static PERK_DEFS: [PerkDef; 20] = [
    PerkDef {
        tree: SkillTree::Investigation,
        level: 1,
        name: "Quick Search",
        description: "+20% evidence collection speed (future: reduces scene interaction time)",
    },
    PerkDef {
        tree: SkillTree::Investigation,
        level: 2,
        name: "Keen Eye",
        description: "Spot hidden evidence in scenes (future: reveals extra evidence nodes)",
    },
    PerkDef {
        tree: SkillTree::Investigation,
        level: 3,
        name: "Forensic Intuition",
        description: "+0.1 base evidence quality",
    },
    PerkDef {
        tree: SkillTree::Investigation,
        level: 4,
        name: "Cold Case Reader",
        description: "Access cold case files in records room",
    },
    PerkDef {
        tree: SkillTree::Investigation,
        level: 5,
        name: "Master Investigator",
        description: "All evidence at +0.15 quality",
    },
    PerkDef {
        tree: SkillTree::Interrogation,
        level: 1,
        name: "Good Cop",
        description: "Trust-building dialogue options available",
    },
    PerkDef {
        tree: SkillTree::Interrogation,
        level: 2,
        name: "Bad Cop",
        description: "Pressure dialogue options available",
    },
    PerkDef {
        tree: SkillTree::Interrogation,
        level: 3,
        name: "Read The Room",
        description: "See NPC trust/pressure values in dialogue",
    },
    PerkDef {
        tree: SkillTree::Interrogation,
        level: 4,
        name: "Confession Artist",
        description: "+25% confession chance in interrogation",
    },
    PerkDef {
        tree: SkillTree::Interrogation,
        level: 5,
        name: "Master Interrogator",
        description: "Unlock all dialogue paths",
    },
    PerkDef {
        tree: SkillTree::Patrol,
        level: 1,
        name: "Beat Knowledge",
        description: "Minimap shows NPC locations (future)",
    },
    PerkDef {
        tree: SkillTree::Patrol,
        level: 2,
        name: "Quick Response",
        description: "-20% travel time (fuel cost reduction)",
    },
    PerkDef {
        tree: SkillTree::Patrol,
        level: 3,
        name: "Pursuit Training",
        description: "Catch fleeing suspects in dispatch events",
    },
    PerkDef {
        tree: SkillTree::Patrol,
        level: 4,
        name: "Community Trust",
        description: "+5 trust with all town NPCs (one-time bonus)",
    },
    PerkDef {
        tree: SkillTree::Patrol,
        level: 5,
        name: "Master Patrol",
        description: "Dispatch calls show difficulty rating",
    },
    PerkDef {
        tree: SkillTree::Leadership,
        level: 1,
        name: "Radio Discipline",
        description: "Clearer dispatch information text",
    },
    PerkDef {
        tree: SkillTree::Leadership,
        level: 2,
        name: "Partner Synergy",
        description: "+10% partner bonus effectiveness",
    },
    PerkDef {
        tree: SkillTree::Leadership,
        level: 3,
        name: "Budget Request",
        description: "Access department budget for equipment",
    },
    PerkDef {
        tree: SkillTree::Leadership,
        level: 4,
        name: "Task Delegation",
        description: "Assign simple tasks to AI officers (future)",
    },
    PerkDef {
        tree: SkillTree::Leadership,
        level: 5,
        name: "Master Commander",
        description: "Direct multi-unit responses (future)",
    },
];

pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SkillUnlockedEvent>()
            .add_systems(
                Update,
                (emit_case_xp, emit_evidence_xp, accumulate_xp)
                    .chain()
                    .in_set(UpdatePhase::Reactions)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (handle_skill_spend, apply_one_time_perks)
                    .chain()
                    .in_set(UpdatePhase::Reactions),
            );
    }
}

pub fn perk_definitions() -> &'static [PerkDef] {
    &PERK_DEFS
}

pub fn perk_definition(tree: SkillTree, level: u8) -> Option<&'static PerkDef> {
    PERK_DEFS
        .iter()
        .find(|perk| perk.tree == tree && perk.level == level)
}

pub fn skill_level(skills: &Skills, tree: SkillTree) -> u8 {
    match tree {
        SkillTree::Investigation => skills.investigation_level,
        SkillTree::Interrogation => skills.interrogation_level,
        SkillTree::Patrol => skills.patrol_level,
        SkillTree::Leadership => skills.leadership_level,
    }
}

pub fn has_perk(skills: &Skills, tree: SkillTree, level: u8) -> bool {
    (1..=MAX_SKILL_LEVEL).contains(&level) && skill_level(skills, tree) >= level
}

pub fn investigation_quality_bonus(skills: &Skills) -> f32 {
    match skills.investigation_level {
        5 => 0.15,
        3 | 4 => 0.1,
        _ => 0.0,
    }
}

fn emit_case_xp(
    mut case_events: EventReader<CaseSolvedEvent>,
    mut xp_events: EventWriter<XpGainedEvent>,
) {
    for event in case_events.read() {
        xp_events.send(XpGainedEvent {
            amount: event.xp_reward,
            source: format!("case:{}", event.case_id),
        });
    }
}

fn emit_evidence_xp(
    mut evidence_events: EventReader<EvidenceCollectedEvent>,
    mut xp_events: EventWriter<XpGainedEvent>,
) {
    for event in evidence_events.read() {
        xp_events.send(XpGainedEvent {
            amount: XP_PER_EVIDENCE,
            source: format!("evidence:{}:{}", event.case_id, event.evidence_id),
        });
    }
}

fn accumulate_xp(
    mut dispatch_events: EventReader<DispatchResolvedEvent>,
    mut xp_events: EventReader<XpGainedEvent>,
    mut skills: ResMut<Skills>,
) {
    let mut xp_delta: u32 = 0;
    let mut patrol_xp_events = 0usize;

    for event in xp_events.read() {
        xp_delta = xp_delta.saturating_add(event.amount);
        if event.source.starts_with("patrol:") {
            patrol_xp_events += 1;
        }
    }

    for event in dispatch_events.read().skip(patrol_xp_events) {
        let amount = if event.xp_earned == 0 {
            XP_PER_PATROL_EVENT
        } else {
            event.xp_earned
        };
        xp_delta = xp_delta.saturating_add(amount);
    }

    if xp_delta == 0 {
        return;
    }

    skills.total_xp = skills.total_xp.saturating_add(xp_delta);
    skills.available_points = available_skill_points(&skills);
}

fn handle_skill_spend(
    mut spend_events: EventReader<SkillPointSpentEvent>,
    mut unlocked_events: EventWriter<SkillUnlockedEvent>,
    mut skills: ResMut<Skills>,
) {
    for event in spend_events.read() {
        if skills.available_points == 0 {
            continue;
        }

        let current_level = skill_level(&skills, event.tree);
        if current_level >= MAX_SKILL_LEVEL {
            continue;
        }

        let next_level = current_level.saturating_add(1);
        if event.new_level != 0 && event.new_level != next_level {
            continue;
        }

        set_skill_level(&mut skills, event.tree, next_level);
        skills.available_points = available_skill_points(&skills);
        unlocked_events.send(SkillUnlockedEvent {
            tree: event.tree,
            level: next_level,
        });
    }
}

fn apply_one_time_perks(
    mut unlocked_events: EventReader<SkillUnlockedEvent>,
    mut trust_events: EventWriter<NpcTrustChangeEvent>,
) {
    for event in unlocked_events.read() {
        if event.tree == SkillTree::Patrol && event.level == 4 {
            for npc_id in COMMUNITY_TRUST_NPC_IDS {
                trust_events.send(NpcTrustChangeEvent {
                    npc_id: npc_id.to_string(),
                    trust_delta: COMMUNITY_TRUST_BONUS,
                    pressure_delta: 0,
                });
            }
        }
    }
}

fn set_skill_level(skills: &mut Skills, tree: SkillTree, level: u8) {
    let clamped_level = level.min(MAX_SKILL_LEVEL);

    match tree {
        SkillTree::Investigation => skills.investigation_level = clamped_level,
        SkillTree::Interrogation => skills.interrogation_level = clamped_level,
        SkillTree::Patrol => skills.patrol_level = clamped_level,
        SkillTree::Leadership => skills.leadership_level = clamped_level,
    }
}

fn spent_skill_points(skills: &Skills) -> u32 {
    u32::from(skills.investigation_level)
        + u32::from(skills.interrogation_level)
        + u32::from(skills.patrol_level)
        + u32::from(skills.leadership_level)
}

fn available_skill_points(skills: &Skills) -> u32 {
    let earned_points = skills.total_xp / SKILL_POINT_INTERVAL;
    earned_points.saturating_sub(spent_skill_points(skills))
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
        app.init_resource::<Skills>();
        app.add_event::<CaseSolvedEvent>();
        app.add_event::<DispatchResolvedEvent>();
        app.add_event::<EvidenceCollectedEvent>();
        app.add_event::<NpcTrustChangeEvent>();
        app.add_event::<SkillPointSpentEvent>();
        app.add_event::<XpGainedEvent>();
        app.add_plugins(SkillsPlugin);
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
    fn defines_all_twenty_named_perks_across_four_trees() {
        let perks = perk_definitions();

        assert_eq!(perks.len(), 20);
        assert_eq!(
            perks
                .iter()
                .filter(|perk| perk.tree == SkillTree::Investigation)
                .count(),
            5
        );
        assert_eq!(
            perks
                .iter()
                .filter(|perk| perk.tree == SkillTree::Interrogation)
                .count(),
            5
        );
        assert_eq!(
            perks
                .iter()
                .filter(|perk| perk.tree == SkillTree::Patrol)
                .count(),
            5
        );
        assert_eq!(
            perks
                .iter()
                .filter(|perk| perk.tree == SkillTree::Leadership)
                .count(),
            5
        );
        assert_eq!(
            perk_definition(SkillTree::Investigation, 5).map(|perk| perk.name),
            Some("Master Investigator")
        );
        assert_eq!(
            perk_definition(SkillTree::Patrol, 4).map(|perk| perk.name),
            Some("Community Trust")
        );
        assert_eq!(
            perk_definition(SkillTree::Leadership, 3).map(|perk| perk.description),
            Some("Access department budget for equipment")
        );
    }

    #[test]
    fn case_solved_event_emits_and_accumulates_case_xp() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<CaseSolvedEvent>>()
            .send(CaseSolvedEvent {
                case_id: "patrol_001_petty_theft".to_string(),
                xp_reward: 45,
                gold_reward: 75,
                reputation_reward: 5,
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.total_xp, 45);
        assert_eq!(skills.available_points, 0);

        let events = app.world().resource::<Events<XpGainedEvent>>();
        let mut reader = events.get_cursor();
        let emitted: Vec<_> = reader
            .read(events)
            .map(|event| (event.amount, event.source.clone()))
            .collect();

        assert!(emitted.contains(&(45, "case:patrol_001_petty_theft".to_string())));
    }

    #[test]
    fn evidence_collected_event_awards_evidence_xp() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<EvidenceCollectedEvent>>()
            .send(EvidenceCollectedEvent {
                evidence_id: "fingerprint".to_string(),
                case_id: "patrol_001_petty_theft".to_string(),
                quality: 0.7,
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.total_xp, XP_PER_EVIDENCE);
        assert_eq!(skills.available_points, 0);
    }

    #[test]
    fn skill_point_is_awarded_at_one_hundred_xp() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<XpGainedEvent>>()
            .send(XpGainedEvent {
                amount: 100,
                source: "test:milestone".to_string(),
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.total_xp, 100);
        assert_eq!(skills.available_points, 1);
    }

    #[test]
    fn skill_spend_increments_tree_level_and_decrements_available_points() {
        let mut app = build_test_app();

        {
            let mut skills = app.world_mut().resource_mut::<Skills>();
            skills.total_xp = 100;
            skills.available_points = 1;
        }

        app.world_mut()
            .resource_mut::<Events<SkillPointSpentEvent>>()
            .send(SkillPointSpentEvent {
                tree: SkillTree::Investigation,
                new_level: 1,
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.investigation_level, 1);
        assert_eq!(skills.available_points, 0);
        assert!(has_perk(&skills, SkillTree::Investigation, 1));
    }

    #[test]
    fn cannot_spend_without_available_points() {
        let mut app = build_test_app();

        app.world_mut()
            .resource_mut::<Events<SkillPointSpentEvent>>()
            .send(SkillPointSpentEvent {
                tree: SkillTree::Interrogation,
                new_level: 1,
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.interrogation_level, 0);
        assert_eq!(skills.available_points, 0);
    }

    #[test]
    fn cannot_exceed_level_five_in_any_tree() {
        let mut app = build_test_app();

        {
            let mut skills = app.world_mut().resource_mut::<Skills>();
            skills.total_xp = 600;
            skills.available_points = 1;
            skills.leadership_level = 5;
        }

        app.world_mut()
            .resource_mut::<Events<SkillPointSpentEvent>>()
            .send(SkillPointSpentEvent {
                tree: SkillTree::Leadership,
                new_level: 6,
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.leadership_level, 5);
        assert_eq!(skills.available_points, 1);
    }

    #[test]
    fn investigation_quality_bonus_matches_unlock_thresholds() {
        let mut skills = Skills::default();
        assert_eq!(investigation_quality_bonus(&skills), 0.0);

        skills.investigation_level = 3;
        assert_eq!(investigation_quality_bonus(&skills), 0.1);

        skills.investigation_level = 5;
        assert_eq!(investigation_quality_bonus(&skills), 0.15);
    }

    #[test]
    fn dispatch_resolved_without_patrol_xp_event_uses_fallback_reward() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut()
            .resource_mut::<Events<DispatchResolvedEvent>>()
            .send(DispatchResolvedEvent {
                kind: crate::shared::DispatchEventKind::TrafficStop,
                xp_earned: 25,
            });

        app.update();

        let skills = app.world().resource::<Skills>();
        assert_eq!(skills.total_xp, 25);
    }

    #[test]
    fn patrol_level_four_emits_community_trust_bonus_events() {
        let mut app = build_test_app();

        {
            let mut skills = app.world_mut().resource_mut::<Skills>();
            skills.total_xp = 400;
            skills.available_points = 1;
            skills.patrol_level = 3;
        }

        app.world_mut()
            .resource_mut::<Events<SkillPointSpentEvent>>()
            .send(SkillPointSpentEvent {
                tree: SkillTree::Patrol,
                new_level: 4,
            });

        app.update();

        let trust_events = app
            .world_mut()
            .resource_mut::<Events<NpcTrustChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(trust_events.len(), COMMUNITY_TRUST_NPC_IDS.len());

        let mut emitted_ids = trust_events
            .iter()
            .map(|event| event.npc_id.as_str())
            .collect::<Vec<_>>();
        emitted_ids.sort_unstable();

        let mut expected_ids = COMMUNITY_TRUST_NPC_IDS.to_vec();
        expected_ids.sort_unstable();

        assert_eq!(emitted_ids, expected_ids);
        assert!(trust_events.iter().all(|event| event.trust_delta == 5));
        assert!(trust_events.iter().all(|event| event.pressure_delta == 0));
    }
}
