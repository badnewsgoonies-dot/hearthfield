use bevy::prelude::*;

use crate::game::components::OfficeWorker;
use crate::game::events::{DayAdvanced, EndDayRequested, EndOfDayEvent};
use crate::game::resources::{
    format_clock, DayClock, DayOutcome, DayStats, InboxState, OfficeRules, OfficeRunConfig,
    PlayerCareerState, PlayerMindState, TaskBoard, WorkerStats,
};
use crate::game::OfficeGameState;

use super::task_board::{
    fail_remaining_task_board_work, seed_task_board, sync_task_board_active_with_inbox,
};

fn build_day_outcome(stats: &DayStats, task_board: &TaskBoard) -> DayOutcome {
    let completed_tasks = task_board.completed_today.len() as u32;
    let failed_tasks = task_board.failed_today.len() as u32;

    DayOutcome {
        salary_earned: completed_tasks as i32 * 12 - failed_tasks as i32 * 4,
        reputation_delta: stats.manager_checkins as i32 * 2
            + stats.coworker_helps as i32 * 3
            + (completed_tasks as i32 / 6)
            - failed_tasks as i32
            - stats.panic_responses as i32 * 2,
        stress_delta: stats.interruptions_triggered as i32 * 3 + stats.panic_responses as i32 * 5
            - stats.calm_responses as i32 * 3
            - stats.coworker_helps as i32 * 2,
        completed_tasks,
        failed_tasks,
    }
}

pub fn sync_taskboard_bridge(
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    inbox: Res<InboxState>,
    mut task_board: ResMut<TaskBoard>,
) {
    if clock.ended {
        return;
    }

    sync_task_board_active_with_inbox(
        &mut task_board,
        clock.day_number,
        inbox.remaining_items,
        rules.day_end_minute,
    );
}

pub fn update_day_outcome_preview(
    stats: Res<DayStats>,
    task_board: Res<TaskBoard>,
    mut day_outcome: ResMut<DayOutcome>,
) {
    *day_outcome = build_day_outcome(&stats, &task_board);
}

pub fn check_end_of_day(
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    inbox: Res<InboxState>,
    mut end_day_requested_writer: EventWriter<EndDayRequested>,
) {
    if clock.ended {
        return;
    }

    let reached_shift_end = clock.current_minute >= rules.day_end_minute;
    let finished_all_work = inbox.remaining_items == 0;
    if !reached_shift_end && !finished_all_work {
        return;
    }

    end_day_requested_writer.send(EndDayRequested);
}

#[allow(clippy::too_many_arguments)]
pub fn finalize_end_day_request(
    mut requests: EventReader<EndDayRequested>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
    mut day_advanced_writer: EventWriter<DayAdvanced>,
    mut end_of_day_writer: EventWriter<EndOfDayEvent>,
    mut clock: ResMut<DayClock>,
    inbox: Res<InboxState>,
    stats: Res<DayStats>,
    mind: Res<PlayerMindState>,
    career: Res<PlayerCareerState>,
    worker_query: Query<&OfficeWorker>,
    mut task_board: ResMut<TaskBoard>,
    mut day_outcome: ResMut<DayOutcome>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        clock.ended = true;
        fail_remaining_task_board_work(&mut task_board);
        *day_outcome = build_day_outcome(&stats, &task_board);

        let summary = EndOfDayEvent {
            day_number: clock.day_number,
            finished_minute: clock.current_minute,
            processed_items: stats.processed_items,
            remaining_items: inbox.remaining_items,
            coffee_breaks: stats.coffee_breaks,
            wait_actions: stats.wait_actions,
            failed_process_attempts: stats.failed_process_attempts,
            interruptions_triggered: stats.interruptions_triggered,
            calm_responses: stats.calm_responses,
            panic_responses: stats.panic_responses,
            unresolved_interruptions: mind.pending_interruptions,
            manager_checkins: stats.manager_checkins,
            coworker_helps: stats.coworker_helps,
            final_energy: worker_query.get_single().map_or(0, |worker| worker.energy),
            final_stress: mind.stress,
            final_focus: mind.focus,
            final_reputation: career.reputation,
        };

        println!();
        println!(
            "=== City Office Worker DLC: End Of Day {} ===",
            summary.day_number
        );
        println!("Clock: {}", format_clock(summary.finished_minute));
        println!("Processed inbox items: {}", summary.processed_items);
        println!("Remaining inbox items: {}", summary.remaining_items);
        println!("Coffee breaks: {}", summary.coffee_breaks);
        println!("Wait actions: {}", summary.wait_actions);
        println!("Interruptions: {}", summary.interruptions_triggered);
        println!("Calm responses: {}", summary.calm_responses);
        println!("Panic responses: {}", summary.panic_responses);
        println!(
            "Unresolved interruptions: {}",
            summary.unresolved_interruptions
        );
        println!("Manager check-ins: {}", summary.manager_checkins);
        println!("Coworker helps: {}", summary.coworker_helps);
        println!(
            "Failed process attempts: {}",
            summary.failed_process_attempts
        );
        println!("Final energy: {}", summary.final_energy);
        println!("Final stress: {}", summary.final_stress);
        println!("Final focus: {}", summary.final_focus);
        println!("Final reputation: {}", summary.final_reputation);
        println!("=============================================");

        let new_day_index = clock.day_number.saturating_add(1);
        end_of_day_writer.send(summary);
        day_advanced_writer.send(DayAdvanced { new_day_index });
        next_state.set(OfficeGameState::DaySummary);
    }
}

pub fn apply_day_summary_rollover(
    clock: Res<DayClock>,
    mut day_outcome: ResMut<DayOutcome>,
    mut worker_stats: ResMut<WorkerStats>,
    mut career: ResMut<PlayerCareerState>,
) {
    if !clock.ended {
        return;
    }

    if day_outcome.salary_earned == 0
        && day_outcome.reputation_delta == 0
        && day_outcome.stress_delta == 0
    {
        return;
    }

    worker_stats.money = worker_stats.money.saturating_add(day_outcome.salary_earned);
    worker_stats.stress = (worker_stats.stress + day_outcome.stress_delta).clamp(0, 100);
    career.reputation = (career.reputation + day_outcome.reputation_delta).clamp(-100, 100);
    worker_stats.reputation = career.reputation;
    worker_stats.normalize();

    info!(
        "Day rollover -> salary: {}, rep delta: {}, stress delta: {}, totals money: {}, reputation: {}",
        day_outcome.salary_earned,
        day_outcome.reputation_delta,
        day_outcome.stress_delta,
        worker_stats.money,
        career.reputation
    );

    day_outcome.salary_earned = 0;
    day_outcome.reputation_delta = 0;
    day_outcome.stress_delta = 0;
}

#[allow(clippy::too_many_arguments)]
pub fn transition_day_summary_to_inday(
    mut day_advanced_reader: EventReader<DayAdvanced>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut inbox: ResMut<InboxState>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut task_board: ResMut<TaskBoard>,
    mut worker_query: Query<&mut OfficeWorker>,
    mut worker_stats: ResMut<WorkerStats>,
    mut run_config: ResMut<OfficeRunConfig>,
    mut day_outcome: ResMut<DayOutcome>,
) {
    let mut transition_target = None;
    for event in day_advanced_reader.read() {
        transition_target = Some(event.new_day_index);
    }

    let Some(new_day_index) = transition_target else {
        return;
    };

    run_config.normalize();
    clock.day_number = new_day_index;
    clock.current_minute = rules.day_start_minute;
    clock.ended = false;

    inbox.remaining_items = rules.starting_inbox_items;
    stats.processed_items = 0;
    stats.coffee_breaks = 0;
    stats.wait_actions = 0;
    stats.failed_process_attempts = 0;
    stats.interruptions_triggered = 0;
    stats.calm_responses = 0;
    stats.panic_responses = 0;
    stats.manager_checkins = 0;
    stats.coworker_helps = 0;

    mind.stress = rules.starting_stress.clamp(0, rules.max_stress);
    mind.focus = rules.starting_focus.clamp(0, rules.max_focus);
    mind.pending_interruptions = 0;

    if let Ok(mut worker) = worker_query.get_single_mut() {
        worker.energy = rules.max_energy;
    }

    worker_stats.energy = rules.max_energy;
    worker_stats.stress = mind.stress;
    worker_stats.focus = mind.focus;
    worker_stats.reputation = worker_stats.reputation.clamp(-100, 100);
    worker_stats.normalize();
    day_outcome.reset();
    *task_board = seed_task_board(new_day_index, inbox.remaining_items, rules.day_end_minute);

    next_state.set(OfficeGameState::InDay);
}
