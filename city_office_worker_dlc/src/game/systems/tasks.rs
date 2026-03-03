use bevy::prelude::*;

use super::task_board::sync_task_board_active_with_inbox;
use crate::game::components::OfficeWorker;
use crate::game::events::{CoffeeBreakEvent, ProcessInboxEvent, WaitEvent};
use crate::game::resources::{
    format_clock, DayClock, DayStats, InboxState, OfficeRules, TaskBoard,
};

pub fn handle_process_requests(
    mut requests: EventReader<ProcessInboxEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut inbox: ResMut<InboxState>,
    mut stats: ResMut<DayStats>,
    mut worker_query: Query<&mut OfficeWorker>,
    mut task_board: Option<ResMut<TaskBoard>>,
) {
    let Ok(mut worker) = worker_query.get_single_mut() else {
        return;
    };

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

        if worker.energy < rules.process_energy_cost {
            stats.failed_process_attempts += 1;
            info!("Not enough energy to process mail. Take coffee or wait.");
            continue;
        }

        worker.energy -= rules.process_energy_cost;
        inbox.remaining_items -= 1;
        stats.processed_items += 1;
        if let Some(task_board) = task_board.as_deref_mut() {
            sync_task_board_active_with_inbox(
                task_board,
                clock.day_number,
                inbox.remaining_items,
                rules.day_end_minute,
            );
        }

        info!(
            "Processed 1 item -> energy: {}, inbox left: {}, clock: {}",
            worker.energy,
            inbox.remaining_items,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_coffee_requests(
    mut requests: EventReader<CoffeeBreakEvent>,
    rules: Res<OfficeRules>,
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
        clock.advance(rules.coffee_minutes);
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
