use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::game::resources::{
    DayClock, InboxState, OfficeRunConfig, OfficeTask, TaskBoard, TaskId, TaskKind, TaskPriority,
    WorkerStats,
};

#[derive(Resource, Debug, Default, Clone)]
pub struct OfficeSaveStore {
    pub latest_snapshot_json: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerStatsSnapshot {
    pub energy: i32,
    pub stress: i32,
    pub focus: i32,
    pub money: i32,
    pub reputation: i32,
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
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn parse_task_kind(raw: &str) -> Result<TaskKind, String> {
    match raw {
        "data_entry" => Ok(TaskKind::DataEntry),
        "filing" => Ok(TaskKind::Filing),
        "email_triage" => Ok(TaskKind::EmailTriage),
        "permit_review" => Ok(TaskKind::PermitReview),
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

#[cfg_attr(not(test), allow(dead_code))]
fn parse_task_priority(raw: &str) -> Result<TaskPriority, String> {
    match raw {
        "low" => Ok(TaskPriority::Low),
        "medium" => Ok(TaskPriority::Medium),
        "high" => Ok(TaskPriority::High),
        "critical" => Ok(TaskPriority::Critical),
        other => Err(format!("unknown task priority: {other}")),
    }
}

pub fn capture_snapshot(
    clock: &DayClock,
    worker_stats: &WorkerStats,
    task_board: &TaskBoard,
    run_config: &OfficeRunConfig,
    inbox: &InboxState,
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
    }
}

pub fn serialize_snapshot(snapshot: &OfficeSaveSnapshot) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(snapshot)
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn deserialize_snapshot(json: &str) -> Result<OfficeSaveSnapshot, serde_json::Error> {
    serde_json::from_str(json)
}

#[allow(clippy::too_many_arguments)]
#[cfg_attr(not(test), allow(dead_code))]
pub fn apply_snapshot(
    snapshot: &OfficeSaveSnapshot,
    clock: &mut DayClock,
    worker_stats: &mut WorkerStats,
    task_board: &mut TaskBoard,
    run_config: &mut OfficeRunConfig,
    inbox: &mut InboxState,
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

pub fn persist_day_summary_snapshot(
    clock: Res<DayClock>,
    worker_stats: Res<WorkerStats>,
    task_board: Res<TaskBoard>,
    run_config: Res<OfficeRunConfig>,
    inbox: Res<InboxState>,
    mut store: ResMut<OfficeSaveStore>,
) {
    if !clock.ended {
        return;
    }

    let snapshot = capture_snapshot(&clock, &worker_stats, &task_board, &run_config, &inbox);
    if let Ok(json) = serialize_snapshot(&snapshot) {
        store.latest_snapshot_json = Some(json);
    }
}
