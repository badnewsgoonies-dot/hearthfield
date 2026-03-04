use bevy::prelude::*;

use super::task_board::sync_task_board_active_with_inbox;
use crate::game::components::OfficeWorker;
use crate::game::events::{
    CoffeeBreakEvent, ProcessInboxEvent, TaskCompleted, TaskFailed, TaskProgressed, WaitEvent,
};
use crate::game::resources::{
    format_clock, CareerProgression, DayClock, DayStats, InboxState, OfficeEconomyRules,
    OfficeRules, PlayerMindState, TaskBoard, TaskPriority, UnlockCatalogState,
};

fn priority_progress_multiplier(priority: TaskPriority) -> f32 {
    match priority {
        TaskPriority::Low => 1.0,
        TaskPriority::Medium => 0.88,
        TaskPriority::High => 0.74,
        TaskPriority::Critical => 0.62,
    }
}

fn progress_delta_for_task(required_focus: i32, priority: TaskPriority, current_focus: i32) -> f32 {
    let focus_ratio = (current_focus.max(0) as f32 / required_focus.max(1) as f32).min(1.0);
    (0.28 * focus_ratio * priority_progress_multiplier(priority)).clamp(0.08, 0.55)
}

#[allow(clippy::too_many_arguments)]
pub fn handle_process_requests(
    mut requests: EventReader<ProcessInboxEvent>,
    rules: Res<OfficeRules>,
    economy: Res<OfficeEconomyRules>,
    progression: Res<CareerProgression>,
    unlocks: Res<UnlockCatalogState>,
    mut clock: ResMut<DayClock>,
    mut inbox: ResMut<InboxState>,
    mut stats: ResMut<DayStats>,
    mind: Res<PlayerMindState>,
    mut task_progressed_writer: EventWriter<TaskProgressed>,
    mut task_completed_writer: EventWriter<TaskCompleted>,
    mut worker_query: Query<&mut OfficeWorker>,
    mut task_board: Option<ResMut<TaskBoard>>,
) {
    let Ok(mut worker) = worker_query.get_single_mut() else {
        return;
    };

    let process_energy_cost =
        (rules.process_energy_cost - progression.process_energy_discount(&economy)).max(1);

    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        clock.advance(rules.process_minutes);

        if inbox.remaining_items == 0 {
            stats.failed_process_attempts += 1;
            info!("Inbox is already empty. Time still moved forward.");
            continue;
        }

        if worker.energy < process_energy_cost {
            stats.failed_process_attempts += 1;
            info!("Not enough energy to process mail. Take coffee or wait.");
            continue;
        }

        worker.energy -= process_energy_cost;
        stats.processed_items += 1;
        if let Some(task_board) = task_board.as_deref_mut() {
            if let Some((task_id, required_focus, priority)) = task_board
                .active
                .first()
                .map(|task| (task.id, task.required_focus, task.priority))
            {
                let delta = progress_delta_for_task(required_focus, priority, mind.focus);
                let adjusted_delta =
                    (delta * unlocks.process_progress_multiplier()).clamp(0.0, 1.0);
                if let Some(applied_delta) = task_board.progress_task(task_id, adjusted_delta) {
                    task_progressed_writer.send(TaskProgressed {
                        task_id,
                        delta: applied_delta,
                    });
                }

                let task_is_complete = task_board
                    .active
                    .iter()
                    .find(|task| task.id == task_id)
                    .is_some_and(|task| task.is_complete());

                if task_is_complete && task_board.complete_task(task_id) {
                    inbox.remaining_items = inbox.remaining_items.saturating_sub(1);
                    task_completed_writer.send(TaskCompleted { task_id });
                }
            } else {
                sync_task_board_active_with_inbox(
                    task_board,
                    clock.day_number,
                    inbox.remaining_items,
                    rules.day_end_minute,
                );
            }
        }

        info!(
            "Processed 1 item -> energy: {}, inbox left: {}, clock: {}",
            worker.energy,
            inbox.remaining_items,
            format_clock(clock.current_minute)
        );
    }
}

pub fn enforce_task_deadlines(
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    mut inbox: ResMut<InboxState>,
    mut task_board: ResMut<TaskBoard>,
    mut task_failed_writer: EventWriter<TaskFailed>,
) {
    if clock.ended {
        return;
    }

    let overdue_task_ids = task_board
        .active
        .iter()
        .filter(|task| clock.current_minute > task.deadline_minute as u32)
        .map(|task| task.id)
        .collect::<Vec<_>>();

    for task_id in overdue_task_ids {
        if task_board.fail_task(task_id) {
            inbox.remaining_items = inbox.remaining_items.saturating_sub(1);
            task_failed_writer.send(TaskFailed { task_id });
        }
    }

    sync_task_board_active_with_inbox(
        &mut task_board,
        clock.day_number,
        inbox.remaining_items,
        rules.day_end_minute,
    );
}

pub fn handle_coffee_requests(
    mut requests: EventReader<CoffeeBreakEvent>,
    rules: Res<OfficeRules>,
    unlocks: Res<UnlockCatalogState>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut worker_query: Query<&mut OfficeWorker>,
) {
    let Ok(mut worker) = worker_query.get_single_mut() else {
        return;
    };

    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        let before = worker.energy;
        worker.energy = (worker.energy + rules.coffee_restore).min(rules.max_energy);
        clock.advance(unlocks.coffee_minutes(rules.coffee_minutes));
        stats.coffee_breaks += 1;

        info!(
            "Coffee break -> energy: {} -> {}, clock: {}",
            before,
            worker.energy,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_wait_requests(
    mut requests: EventReader<WaitEvent>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
) {
    for wait in requests.read() {
        if clock.ended {
            continue;
        }

        let minutes = wait.minutes.max(1);
        clock.advance(minutes);
        stats.wait_actions += 1;
        info!(
            "Waited {} minutes. Clock: {}",
            minutes,
            format_clock(clock.current_minute)
        );
    }
}
