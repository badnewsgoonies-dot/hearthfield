use std::fs;
use std::path::PathBuf;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::game::components::OfficeWorker;
use crate::game::events::DayAdvanced;
use crate::game::resources::{
    CareerProgression, CoworkerProfile, CoworkerRole, DayClock, DayOutcome, DayStats,
    FiredMilestones, InboxState, MilestoneKind, OfficeEconomyRules, OfficeRunConfig, OfficeTask,
    PlayerCareerState, PlayerMindState, SocialGraphState, TaskBoard, TaskId, TaskKind, TaskPriority,
    UnlockCatalogState, WorkerStats,
};
use crate::game::OfficeGameState;

#[derive(Resource, Debug, Clone)]
pub struct SaveSlotConfig {
    pub save_dir: PathBuf,
    pub active_slot: u8,
}

impl Default for SaveSlotConfig {
    fn default() -> Self {
        Self {
            save_dir: PathBuf::from("city_office_worker_dlc/saves"),
            active_slot: 0,
        }
    }
}

#[derive(Resource, Debug, Default, Clone)]
pub struct OfficeSaveStore {
    pub latest_snapshot_json: Option<String>,
    pub last_io_error: Option<String>,
    pub last_loaded_slot: Option<u8>,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SaveSlotRequest {
    pub slot: u8,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct LoadSlotRequest {
    pub slot: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfficeSaveSnapshot {
    pub schema_version: u16,
    pub day_index: u32,
    pub minute_of_day: u32,
    pub day_ended: bool,
    pub inbox_remaining: u32,
    pub run_seed: u64,
    pub worker_stats: WorkerStatsSnapshot,
    pub task_board: TaskBoardSnapshot,
    #[serde(default)]
    pub day_stats: DayStatsSnapshot,
    #[serde(default)]
    pub pending_interruptions: u32,
    #[serde(default)]
    pub progression: CareerProgressionSnapshot,
    #[serde(default)]
    pub social_graph: SocialGraphSnapshot,
    #[serde(default)]
    pub unlocks: UnlockCatalogSnapshot,
    #[serde(default)]
    pub day_outcome: DayOutcomeSnapshot,
    #[serde(default)]
    pub fired_milestones: FiredMilestonesSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct OfficeSaveSnapshotV0 {
    pub day_index: u32,
    pub minute_of_day: u32,
    pub day_ended: bool,
    pub inbox_remaining: u32,
    pub run_seed: u64,
    pub energy: i32,
    pub stress: i32,
    pub focus: i32,
    pub money: i32,
    pub reputation: i32,
    pub active_task_ids: Vec<u64>,
    pub completed_today: Vec<u64>,
    pub failed_today: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerStatsSnapshot {
    pub energy: i32,
    pub stress: i32,
    pub focus: i32,
    pub money: i32,
    pub reputation: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DayStatsSnapshot {
    pub processed_items: u32,
    pub coffee_breaks: u32,
    pub wait_actions: u32,
    pub failed_process_attempts: u32,
    pub interruptions_triggered: u32,
    pub calm_responses: u32,
    pub panic_responses: u32,
    pub manager_checkins: u32,
    pub coworker_helps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CareerProgressionSnapshot {
    pub level: u32,
    pub xp: u32,
    pub success_streak: u32,
    pub burnout_days: u32,
    pub efficiency_perk: u8,
    pub resilience_perk: u8,
    pub diplomacy_perk: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SocialGraphSnapshot {
    pub profiles: Vec<CoworkerSnapshot>,
    pub scenario_cursor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CoworkerSnapshot {
    pub id: u8,
    pub codename: String,
    pub role: String,
    pub affinity: i32,
    pub trust: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct DayOutcomeSnapshot {
    pub salary_earned: i32,
    pub reputation_delta: i32,
    pub stress_delta: i32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct UnlockCatalogSnapshot {
    pub quick_coffee: bool,
    pub efficient_processing: bool,
    pub conflict_training: bool,
    pub escalation_license: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FiredMilestonesSnapshot {
    pub fired: Vec<(u8, String)>,
}

fn milestone_kind_str(kind: MilestoneKind) -> &'static str {
    match kind {
        MilestoneKind::Friendly => "friendly",
        MilestoneKind::Trusted => "trusted",
        MilestoneKind::CloseFriend => "close_friend",
        MilestoneKind::DeepTrust => "deep_trust",
        MilestoneKind::Rival => "rival",
        MilestoneKind::Distrusted => "distrusted",
    }
}

fn parse_milestone_kind(raw: &str) -> Result<MilestoneKind, String> {
    match raw {
        "friendly" => Ok(MilestoneKind::Friendly),
        "trusted" => Ok(MilestoneKind::Trusted),
        "close_friend" => Ok(MilestoneKind::CloseFriend),
        "deep_trust" => Ok(MilestoneKind::DeepTrust),
        "rival" => Ok(MilestoneKind::Rival),
        "distrusted" => Ok(MilestoneKind::Distrusted),
        other => Err(format!("unknown milestone kind: {other}")),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskBoardSnapshot {
    pub active: Vec<OfficeTaskSnapshot>,
    pub completed_today: Vec<u64>,
    pub failed_today: Vec<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OfficeTaskSnapshot {
    pub id: u64,
    pub kind: String,
    pub priority: String,
    pub required_focus: i32,
    pub stress_impact: i32,
    pub reward_money: i32,
    pub reward_reputation: i32,
    pub deadline_minute: u16,
    pub progress: f32,
}

fn task_kind_str(kind: TaskKind) -> &'static str {
    match kind {
        TaskKind::DataEntry => "data_entry",
        TaskKind::Filing => "filing",
        TaskKind::EmailTriage => "email_triage",
        TaskKind::PermitReview => "permit_review",
        TaskKind::PhoneCall => "phone_call",
        TaskKind::MeetingPrep => "meeting_prep",
        TaskKind::ReportWriting => "report_writing",
        TaskKind::BudgetReview => "budget_review",
    }
}

fn parse_task_kind(raw: &str) -> Result<TaskKind, String> {
    match raw {
        "data_entry" => Ok(TaskKind::DataEntry),
        "filing" => Ok(TaskKind::Filing),
        "email_triage" => Ok(TaskKind::EmailTriage),
        "permit_review" => Ok(TaskKind::PermitReview),
        "phone_call" => Ok(TaskKind::PhoneCall),
        "meeting_prep" => Ok(TaskKind::MeetingPrep),
        "report_writing" => Ok(TaskKind::ReportWriting),
        "budget_review" => Ok(TaskKind::BudgetReview),
        other => Err(format!("unknown task kind: {other}")),
    }
}

fn task_priority_str(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "low",
        TaskPriority::Medium => "medium",
        TaskPriority::High => "high",
        TaskPriority::Critical => "critical",
    }
}

fn parse_task_priority(raw: &str) -> Result<TaskPriority, String> {
    match raw {
        "low" => Ok(TaskPriority::Low),
        "medium" => Ok(TaskPriority::Medium),
        "high" => Ok(TaskPriority::High),
        "critical" => Ok(TaskPriority::Critical),
        other => Err(format!("unknown task priority: {other}")),
    }
}

fn coworker_role_str(role: CoworkerRole) -> &'static str {
    match role {
        CoworkerRole::Manager => "manager",
        CoworkerRole::Clerk => "clerk",
        CoworkerRole::Analyst => "analyst",
        CoworkerRole::Coordinator => "coordinator",
        CoworkerRole::Intern => "intern",
    }
}

fn parse_coworker_role(raw: &str) -> Result<CoworkerRole, String> {
    match raw {
        "manager" => Ok(CoworkerRole::Manager),
        "clerk" => Ok(CoworkerRole::Clerk),
        "analyst" => Ok(CoworkerRole::Analyst),
        "coordinator" => Ok(CoworkerRole::Coordinator),
        "intern" => Ok(CoworkerRole::Intern),
        other => Err(format!("unknown coworker role: {other}")),
    }
}

fn migrate_v0_to_v1(v0: OfficeSaveSnapshotV0) -> OfficeSaveSnapshot {
    OfficeSaveSnapshot {
        schema_version: 1,
        day_index: v0.day_index,
        minute_of_day: v0.minute_of_day,
        day_ended: v0.day_ended,
        inbox_remaining: v0.inbox_remaining,
        run_seed: v0.run_seed,
        worker_stats: WorkerStatsSnapshot {
            energy: v0.energy,
            stress: v0.stress,
            focus: v0.focus,
            money: v0.money,
            reputation: v0.reputation,
        },
        task_board: TaskBoardSnapshot {
            active: v0
                .active_task_ids
                .into_iter()
                .map(|task_id| OfficeTaskSnapshot {
                    id: task_id,
                    kind: "data_entry".to_string(),
                    priority: "medium".to_string(),
                    required_focus: 18,
                    stress_impact: 3,
                    reward_money: 12,
                    reward_reputation: 1,
                    deadline_minute: 17 * 60,
                    progress: 0.0,
                })
                .collect(),
            completed_today: v0.completed_today,
            failed_today: v0.failed_today,
        },
        day_stats: DayStatsSnapshot::default(),
        pending_interruptions: 0,
        progression: CareerProgressionSnapshot::default(),
        social_graph: SocialGraphSnapshot::default(),
        unlocks: UnlockCatalogSnapshot::default(),
        day_outcome: DayOutcomeSnapshot::default(),
        fired_milestones: FiredMilestonesSnapshot::default(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn capture_snapshot(
    clock: &DayClock,
    worker_stats: &WorkerStats,
    task_board: &TaskBoard,
    run_config: &OfficeRunConfig,
    inbox: &InboxState,
    day_stats: &DayStats,
    mind: &PlayerMindState,
    progression: &CareerProgression,
    social_graph: &SocialGraphState,
    unlocks: &UnlockCatalogState,
    day_outcome: &DayOutcome,
    fired_milestones: &FiredMilestones,
) -> OfficeSaveSnapshot {
    OfficeSaveSnapshot {
        schema_version: 1,
        day_index: clock.day_number,
        minute_of_day: clock.current_minute,
        day_ended: clock.ended,
        inbox_remaining: inbox.remaining_items,
        run_seed: run_config.seed,
        worker_stats: WorkerStatsSnapshot {
            energy: worker_stats.energy,
            stress: worker_stats.stress,
            focus: worker_stats.focus,
            money: worker_stats.money,
            reputation: worker_stats.reputation,
        },
        task_board: TaskBoardSnapshot {
            active: task_board
                .active
                .iter()
                .map(|task| OfficeTaskSnapshot {
                    id: task.id.0,
                    kind: task_kind_str(task.kind).to_string(),
                    priority: task_priority_str(task.priority).to_string(),
                    required_focus: task.required_focus,
                    stress_impact: task.stress_impact,
                    reward_money: task.reward_money,
                    reward_reputation: task.reward_reputation,
                    deadline_minute: task.deadline_minute,
                    progress: task.progress,
                })
                .collect(),
            completed_today: task_board
                .completed_today
                .iter()
                .map(|task_id| task_id.0)
                .collect(),
            failed_today: task_board
                .failed_today
                .iter()
                .map(|task_id| task_id.0)
                .collect(),
        },
        day_stats: DayStatsSnapshot {
            processed_items: day_stats.processed_items,
            coffee_breaks: day_stats.coffee_breaks,
            wait_actions: day_stats.wait_actions,
            failed_process_attempts: day_stats.failed_process_attempts,
            interruptions_triggered: day_stats.interruptions_triggered,
            calm_responses: day_stats.calm_responses,
            panic_responses: day_stats.panic_responses,
            manager_checkins: day_stats.manager_checkins,
            coworker_helps: day_stats.coworker_helps,
        },
        pending_interruptions: mind.pending_interruptions,
        progression: CareerProgressionSnapshot {
            level: progression.level,
            xp: progression.xp,
            success_streak: progression.success_streak,
            burnout_days: progression.burnout_days,
            efficiency_perk: progression.efficiency_perk,
            resilience_perk: progression.resilience_perk,
            diplomacy_perk: progression.diplomacy_perk,
        },
        social_graph: SocialGraphSnapshot {
            profiles: social_graph
                .profiles
                .iter()
                .map(|profile| CoworkerSnapshot {
                    id: profile.id,
                    codename: profile.codename.clone(),
                    role: coworker_role_str(profile.role).to_string(),
                    affinity: profile.affinity,
                    trust: profile.trust,
                })
                .collect(),
            scenario_cursor: social_graph.scenario_cursor,
        },
        unlocks: UnlockCatalogSnapshot {
            quick_coffee: unlocks.quick_coffee,
            efficient_processing: unlocks.efficient_processing,
            conflict_training: unlocks.conflict_training,
            escalation_license: unlocks.escalation_license,
        },
        day_outcome: DayOutcomeSnapshot {
            salary_earned: day_outcome.salary_earned,
            reputation_delta: day_outcome.reputation_delta,
            stress_delta: day_outcome.stress_delta,
            completed_tasks: day_outcome.completed_tasks,
            failed_tasks: day_outcome.failed_tasks,
        },
        fired_milestones: FiredMilestonesSnapshot {
            fired: fired_milestones
                .fired
                .iter()
                .map(|(id, kind)| (*id, milestone_kind_str(*kind).to_string()))
                .collect(),
        },
    }
}

pub fn serialize_snapshot(snapshot: &OfficeSaveSnapshot) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(snapshot)
}

pub fn deserialize_snapshot(json: &str) -> Result<OfficeSaveSnapshot, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn migrate_snapshot_json(json: &str) -> Result<OfficeSaveSnapshot, String> {
    let raw_value: Value = serde_json::from_str(json)
        .map_err(|error| format!("failed to parse snapshot JSON: {error}"))?;

    let schema_version = raw_value
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u16;

    match schema_version {
        1 => deserialize_snapshot(json)
            .map_err(|error| format!("failed to decode v1 snapshot: {error}")),
        0 => {
            let legacy: OfficeSaveSnapshotV0 = serde_json::from_value(raw_value)
                .map_err(|error| format!("failed to decode v0 snapshot: {error}"))?;
            Ok(migrate_v0_to_v1(legacy))
        }
        other => Err(format!("unsupported snapshot schema version {other}")),
    }
}

fn slot_file_path(config: &SaveSlotConfig, slot: u8) -> PathBuf {
    config.save_dir.join(format!("slot_{slot:02}.json"))
}

pub fn write_snapshot_to_slot(
    config: &SaveSlotConfig,
    slot: u8,
    snapshot: &OfficeSaveSnapshot,
) -> Result<PathBuf, String> {
    fs::create_dir_all(&config.save_dir)
        .map_err(|error| format!("failed to create save dir {:?}: {error}", config.save_dir))?;

    let path = slot_file_path(config, slot);
    let snapshot_json = serialize_snapshot(snapshot)
        .map_err(|error| format!("failed to serialize snapshot: {error}"))?;
    fs::write(&path, snapshot_json)
        .map_err(|error| format!("failed to write snapshot {:?}: {error}", path))?;

    Ok(path)
}

pub fn read_snapshot_from_slot(
    config: &SaveSlotConfig,
    slot: u8,
) -> Result<OfficeSaveSnapshot, String> {
    let path = slot_file_path(config, slot);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read snapshot {:?}: {error}", path))?;
    migrate_snapshot_json(&content)
}

#[allow(clippy::too_many_arguments)]
pub fn apply_snapshot(
    snapshot: &OfficeSaveSnapshot,
    clock: &mut DayClock,
    worker_stats: &mut WorkerStats,
    task_board: &mut TaskBoard,
    run_config: &mut OfficeRunConfig,
    inbox: &mut InboxState,
    day_stats: &mut DayStats,
    mind: &mut PlayerMindState,
    progression: &mut CareerProgression,
    social_graph: &mut SocialGraphState,
    unlocks: &mut UnlockCatalogState,
    economy: &OfficeEconomyRules,
    day_outcome: &mut DayOutcome,
    fired_milestones: &mut FiredMilestones,
) -> Result<(), String> {
    if snapshot.schema_version != 1 {
        return Err(format!(
            "unsupported snapshot schema version {}",
            snapshot.schema_version
        ));
    }

    let mut restored_active = Vec::with_capacity(snapshot.task_board.active.len());
    for task in &snapshot.task_board.active {
        restored_active.push(OfficeTask {
            id: TaskId(task.id),
            kind: parse_task_kind(&task.kind)?,
            priority: parse_task_priority(&task.priority)?,
            required_focus: task.required_focus,
            stress_impact: task.stress_impact,
            reward_money: task.reward_money,
            reward_reputation: task.reward_reputation,
            deadline_minute: task.deadline_minute,
            progress: task.progress,
        });
    }

    clock.day_number = snapshot.day_index;
    clock.current_minute = snapshot.minute_of_day;
    clock.ended = snapshot.day_ended;

    inbox.remaining_items = snapshot.inbox_remaining;

    run_config.seed = snapshot.run_seed;
    run_config.normalize();

    worker_stats.energy = snapshot.worker_stats.energy;
    worker_stats.stress = snapshot.worker_stats.stress;
    worker_stats.focus = snapshot.worker_stats.focus;
    worker_stats.money = snapshot.worker_stats.money;
    worker_stats.reputation = snapshot.worker_stats.reputation;
    worker_stats.normalize();

    day_stats.processed_items = snapshot.day_stats.processed_items;
    day_stats.coffee_breaks = snapshot.day_stats.coffee_breaks;
    day_stats.wait_actions = snapshot.day_stats.wait_actions;
    day_stats.failed_process_attempts = snapshot.day_stats.failed_process_attempts;
    day_stats.interruptions_triggered = snapshot.day_stats.interruptions_triggered;
    day_stats.calm_responses = snapshot.day_stats.calm_responses;
    day_stats.panic_responses = snapshot.day_stats.panic_responses;
    day_stats.manager_checkins = snapshot.day_stats.manager_checkins;
    day_stats.coworker_helps = snapshot.day_stats.coworker_helps;

    mind.pending_interruptions = snapshot.pending_interruptions;
    mind.stress = worker_stats.stress;
    mind.focus = worker_stats.focus;
    progression.level = snapshot.progression.level.max(1);
    progression.xp = snapshot.progression.xp;
    progression.success_streak = snapshot.progression.success_streak;
    progression.burnout_days = snapshot.progression.burnout_days;
    progression.efficiency_perk = snapshot.progression.efficiency_perk;
    progression.resilience_perk = snapshot.progression.resilience_perk;
    progression.diplomacy_perk = snapshot.progression.diplomacy_perk;
    progression.normalize(economy);

    let mut restored_profiles = Vec::with_capacity(snapshot.social_graph.profiles.len());
    for profile in &snapshot.social_graph.profiles {
        restored_profiles.push(CoworkerProfile {
            id: profile.id,
            codename: profile.codename.clone(),
            role: parse_coworker_role(&profile.role)?,
            affinity: profile.affinity,
            trust: profile.trust,
        });
    }
    if !restored_profiles.is_empty() {
        social_graph.profiles = restored_profiles;
    }
    social_graph.scenario_cursor = snapshot.social_graph.scenario_cursor;
    social_graph.normalize();

    unlocks.quick_coffee = snapshot.unlocks.quick_coffee;
    unlocks.efficient_processing = snapshot.unlocks.efficient_processing;
    unlocks.conflict_training = snapshot.unlocks.conflict_training;
    unlocks.escalation_license = snapshot.unlocks.escalation_license;
    unlocks.sync_with_progression(progression);

    day_outcome.salary_earned = snapshot.day_outcome.salary_earned;
    day_outcome.reputation_delta = snapshot.day_outcome.reputation_delta;
    day_outcome.stress_delta = snapshot.day_outcome.stress_delta;
    day_outcome.completed_tasks = snapshot.day_outcome.completed_tasks;
    day_outcome.failed_tasks = snapshot.day_outcome.failed_tasks;

    fired_milestones.fired.clear();
    for (id, kind_str) in &snapshot.fired_milestones.fired {
        let kind = parse_milestone_kind(kind_str)?;
        fired_milestones.fired.insert((*id, kind));
    }

    task_board.active = restored_active;
    task_board.completed_today = snapshot
        .task_board
        .completed_today
        .iter()
        .copied()
        .map(TaskId)
        .collect();
    task_board.failed_today = snapshot
        .task_board
        .failed_today
        .iter()
        .copied()
        .map(TaskId)
        .collect();
    task_board.normalize();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn save_slot(
    slot: u8,
    config: &SaveSlotConfig,
    clock: &DayClock,
    worker_stats: &WorkerStats,
    task_board: &TaskBoard,
    run_config: &OfficeRunConfig,
    inbox: &InboxState,
    day_stats: &DayStats,
    mind: &PlayerMindState,
    progression: &CareerProgression,
    social_graph: &SocialGraphState,
    unlocks: &UnlockCatalogState,
    day_outcome: &DayOutcome,
    fired_milestones: &FiredMilestones,
    store: &mut OfficeSaveStore,
) -> bool {
    let snapshot = capture_snapshot(
        clock,
        worker_stats,
        task_board,
        run_config,
        inbox,
        day_stats,
        mind,
        progression,
        social_graph,
        unlocks,
        day_outcome,
        fired_milestones,
    );

    match serialize_snapshot(&snapshot) {
        Ok(json) => {
            store.latest_snapshot_json = Some(json);
        }
        Err(error) => {
            store.last_io_error = Some(format!("snapshot serialize error: {error}"));
            return false;
        }
    }

    match write_snapshot_to_slot(config, slot, &snapshot) {
        Ok(_) => {
            store.last_io_error = None;
            true
        }
        Err(error) => {
            store.last_io_error = Some(error);
            false
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn load_slot(
    slot: u8,
    config: &SaveSlotConfig,
    store: &mut OfficeSaveStore,
    clock: &mut DayClock,
    worker_stats: &mut WorkerStats,
    task_board: &mut TaskBoard,
    run_config: &mut OfficeRunConfig,
    inbox: &mut InboxState,
    day_stats: &mut DayStats,
    mind: &mut PlayerMindState,
    progression: &mut CareerProgression,
    social_graph: &mut SocialGraphState,
    unlocks: &mut UnlockCatalogState,
    career: &mut PlayerCareerState,
    economy: &OfficeEconomyRules,
    worker_query: &mut Query<&mut OfficeWorker>,
    day_outcome: &mut DayOutcome,
    fired_milestones: &mut FiredMilestones,
) -> Option<OfficeSaveSnapshot> {
    match read_snapshot_from_slot(config, slot) {
        Ok(snapshot) => {
            let apply_result = apply_snapshot(
                &snapshot,
                clock,
                worker_stats,
                task_board,
                run_config,
                inbox,
                day_stats,
                mind,
                progression,
                social_graph,
                unlocks,
                economy,
                day_outcome,
                fired_milestones,
            );

            if let Err(error) = apply_result {
                store.last_io_error = Some(format!("snapshot apply error: {error}"));
                return None;
            }

            mind.stress = worker_stats.stress;
            mind.focus = worker_stats.focus;
            career.reputation = worker_stats.reputation;
            if let Ok(mut worker) = worker_query.get_single_mut() {
                worker.energy = worker_stats.energy;
            }

            store.last_loaded_slot = Some(slot);
            store.last_io_error = None;
            Some(snapshot)
        }
        Err(error) => {
            store.last_io_error = Some(error);
            None
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn persist_day_summary_snapshot(
    clock: Res<DayClock>,
    worker_stats: Res<WorkerStats>,
    task_board: Res<TaskBoard>,
    run_config: Res<OfficeRunConfig>,
    inbox: Res<InboxState>,
    day_stats: Res<DayStats>,
    mind: Res<PlayerMindState>,
    progression: Res<CareerProgression>,
    social_graph: Res<SocialGraphState>,
    unlocks: Res<UnlockCatalogState>,
    day_outcome: Res<DayOutcome>,
    fired_milestones: Res<FiredMilestones>,
    config: Res<SaveSlotConfig>,
    mut store: ResMut<OfficeSaveStore>,
) {
    if !clock.ended {
        return;
    }

    save_slot(
        config.active_slot,
        &config,
        &clock,
        &worker_stats,
        &task_board,
        &run_config,
        &inbox,
        &day_stats,
        &mind,
        &progression,
        &social_graph,
        &unlocks,
        &day_outcome,
        &fired_milestones,
        &mut store,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn handle_save_slot_requests(
    mut requests: EventReader<SaveSlotRequest>,
    clock: Res<DayClock>,
    worker_stats: Res<WorkerStats>,
    task_board: Res<TaskBoard>,
    run_config: Res<OfficeRunConfig>,
    inbox: Res<InboxState>,
    day_stats: Res<DayStats>,
    mind: Res<PlayerMindState>,
    progression: Res<CareerProgression>,
    social_graph: Res<SocialGraphState>,
    unlocks: Res<UnlockCatalogState>,
    day_outcome: Res<DayOutcome>,
    fired_milestones: Res<FiredMilestones>,
    mut config: ResMut<SaveSlotConfig>,
    mut store: ResMut<OfficeSaveStore>,
) {
    for request in requests.read() {
        let saved = save_slot(
            request.slot,
            &config,
            &clock,
            &worker_stats,
            &task_board,
            &run_config,
            &inbox,
            &day_stats,
            &mind,
            &progression,
            &social_graph,
            &unlocks,
            &day_outcome,
            &fired_milestones,
            &mut store,
        );
        if saved {
            config.active_slot = request.slot;
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_load_slot_requests(
    mut requests: EventReader<LoadSlotRequest>,
    mut params: LoadSlotSystemParams,
) {
    for request in requests.read() {
        let loaded_snapshot = load_slot(
            request.slot,
            &params.config,
            &mut params.store,
            &mut params.clock,
            &mut params.worker_stats,
            &mut params.task_board,
            &mut params.run_config,
            &mut params.inbox,
            &mut params.day_stats,
            &mut params.mind,
            &mut params.progression,
            &mut params.social_graph,
            &mut params.unlocks,
            &mut params.career,
            &params.economy,
            &mut params.worker_query,
            &mut params.day_outcome,
            &mut params.fired_milestones,
        );
        let Some(snapshot) = loaded_snapshot else {
            continue;
        };

        params.config.active_slot = request.slot;
        if snapshot.day_ended {
            params.day_advanced_writer.send(DayAdvanced {
                new_day_index: snapshot.day_index.saturating_add(1),
            });
            params.next_state.set(OfficeGameState::DaySummary);
        }
    }
}

#[derive(SystemParam)]
pub struct LoadSlotSystemParams<'w, 's> {
    config: ResMut<'w, SaveSlotConfig>,
    store: ResMut<'w, OfficeSaveStore>,
    clock: ResMut<'w, DayClock>,
    worker_stats: ResMut<'w, WorkerStats>,
    task_board: ResMut<'w, TaskBoard>,
    run_config: ResMut<'w, OfficeRunConfig>,
    inbox: ResMut<'w, InboxState>,
    day_stats: ResMut<'w, DayStats>,
    mind: ResMut<'w, PlayerMindState>,
    progression: ResMut<'w, CareerProgression>,
    social_graph: ResMut<'w, SocialGraphState>,
    unlocks: ResMut<'w, UnlockCatalogState>,
    career: ResMut<'w, PlayerCareerState>,
    day_outcome: ResMut<'w, DayOutcome>,
    fired_milestones: ResMut<'w, FiredMilestones>,
    economy: Res<'w, OfficeEconomyRules>,
    next_state: ResMut<'w, NextState<OfficeGameState>>,
    day_advanced_writer: EventWriter<'w, DayAdvanced>,
    worker_query: Query<'w, 's, &'static mut OfficeWorker>,
}
