use crate::game::resources::{OfficeTask, TaskBoard, TaskId, TaskKind, TaskPriority};

fn task_id_for_slot(day_number: u32, slot_index: u32) -> TaskId {
    TaskId(((day_number as u64) << 32) | (slot_index as u64 + 1))
}

fn inbox_task(day_number: u32, slot_index: u32, day_end_minute: u32) -> OfficeTask {
    let priority = match slot_index % 4 {
        0 => TaskPriority::Medium,
        1 => TaskPriority::High,
        2 => TaskPriority::Low,
        _ => TaskPriority::Critical,
    };
    let required_focus = match priority {
        TaskPriority::Low => 24,
        TaskPriority::Medium => 42,
        TaskPriority::High => 58,
        TaskPriority::Critical => 72,
    };
    let stress_impact = match priority {
        TaskPriority::Low => 2,
        TaskPriority::Medium => 3,
        TaskPriority::High => 5,
        TaskPriority::Critical => 8,
    };
    let reward_money = match priority {
        TaskPriority::Low => 9,
        TaskPriority::Medium => 12,
        TaskPriority::High => 17,
        TaskPriority::Critical => 22,
    };
    let reward_reputation = match priority {
        TaskPriority::Low => 0,
        TaskPriority::Medium => 1,
        TaskPriority::High => 2,
        TaskPriority::Critical => 3,
    };
    let deadline_window = match priority {
        TaskPriority::Low => 0,
        TaskPriority::Medium => 30,
        TaskPriority::High => 60,
        TaskPriority::Critical => 120,
    };

    OfficeTask {
        id: task_id_for_slot(day_number, slot_index),
        kind: TaskKind::DataEntry,
        priority,
        required_focus,
        stress_impact,
        reward_money,
        reward_reputation,
        deadline_minute: day_end_minute
            .saturating_sub(deadline_window)
            .min(u16::MAX as u32) as u16,
        progress: 0.0,
    }
}

pub(crate) fn seed_task_board(day_number: u32, inbox_items: u32, day_end_minute: u32) -> TaskBoard {
    TaskBoard {
        active: (0..inbox_items)
            .map(|index| inbox_task(day_number, index, day_end_minute))
            .collect(),
        completed_today: Vec::new(),
        failed_today: Vec::new(),
    }
}

pub(crate) fn sync_task_board_active_with_inbox(
    task_board: &mut TaskBoard,
    day_number: u32,
    inbox_items: u32,
    day_end_minute: u32,
) {
    task_board.normalize();
    let target_len = inbox_items as usize;

    while task_board.active.len() > target_len {
        let Some(task_id) = task_board.active.last().map(|task| task.id) else {
            break;
        };
        let _ = task_board.complete_task(task_id);
    }

    while task_board.active.len() < target_len {
        let next_slot = task_board.active.len() as u32;
        let _ = task_board.try_add_task(inbox_task(day_number, next_slot, day_end_minute));
    }

    task_board.normalize();
}

pub(crate) fn fail_remaining_task_board_work(task_board: &mut TaskBoard) -> Vec<TaskId> {
    let remaining_ids: Vec<TaskId> = task_board.active.iter().map(|task| task.id).collect();
    let mut newly_failed = Vec::new();
    for task_id in remaining_ids {
        if task_board.fail_task(task_id) {
            newly_failed.push(task_id);
        }
    }
    task_board.normalize();
    newly_failed
}
