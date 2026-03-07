use crate::game::resources::{OfficeTask, TaskBoard, TaskId, TaskKind, TaskPriority};

#[derive(Clone, Copy)]
struct TaskTemplate {
    kind: TaskKind,
    priority: TaskPriority,
    base_focus: i32,
    base_stress: i32,
    base_reward_money: i32,
    base_reward_reputation: i32,
    deadline_window_minutes: u32,
}

const TASK_TEMPLATES: [TaskTemplate; 24] = [
    // Original 12
    TaskTemplate {
        kind: TaskKind::DataEntry,
        priority: TaskPriority::Low,
        base_focus: 22,
        base_stress: 2,
        base_reward_money: 8,
        base_reward_reputation: 0,
        deadline_window_minutes: 12,
    },
    TaskTemplate {
        kind: TaskKind::DataEntry,
        priority: TaskPriority::Medium,
        base_focus: 34,
        base_stress: 3,
        base_reward_money: 11,
        base_reward_reputation: 1,
        deadline_window_minutes: 28,
    },
    TaskTemplate {
        kind: TaskKind::Filing,
        priority: TaskPriority::Low,
        base_focus: 26,
        base_stress: 2,
        base_reward_money: 9,
        base_reward_reputation: 0,
        deadline_window_minutes: 16,
    },
    TaskTemplate {
        kind: TaskKind::Filing,
        priority: TaskPriority::High,
        base_focus: 52,
        base_stress: 5,
        base_reward_money: 16,
        base_reward_reputation: 2,
        deadline_window_minutes: 58,
    },
    TaskTemplate {
        kind: TaskKind::EmailTriage,
        priority: TaskPriority::Medium,
        base_focus: 38,
        base_stress: 3,
        base_reward_money: 12,
        base_reward_reputation: 1,
        deadline_window_minutes: 32,
    },
    TaskTemplate {
        kind: TaskKind::EmailTriage,
        priority: TaskPriority::Critical,
        base_focus: 66,
        base_stress: 8,
        base_reward_money: 22,
        base_reward_reputation: 3,
        deadline_window_minutes: 118,
    },
    TaskTemplate {
        kind: TaskKind::PermitReview,
        priority: TaskPriority::Medium,
        base_focus: 44,
        base_stress: 4,
        base_reward_money: 13,
        base_reward_reputation: 1,
        deadline_window_minutes: 36,
    },
    TaskTemplate {
        kind: TaskKind::PermitReview,
        priority: TaskPriority::High,
        base_focus: 58,
        base_stress: 6,
        base_reward_money: 18,
        base_reward_reputation: 2,
        deadline_window_minutes: 74,
    },
    TaskTemplate {
        kind: TaskKind::DataEntry,
        priority: TaskPriority::Critical,
        base_focus: 62,
        base_stress: 7,
        base_reward_money: 21,
        base_reward_reputation: 3,
        deadline_window_minutes: 102,
    },
    TaskTemplate {
        kind: TaskKind::Filing,
        priority: TaskPriority::Medium,
        base_focus: 40,
        base_stress: 4,
        base_reward_money: 12,
        base_reward_reputation: 1,
        deadline_window_minutes: 40,
    },
    TaskTemplate {
        kind: TaskKind::EmailTriage,
        priority: TaskPriority::High,
        base_focus: 56,
        base_stress: 6,
        base_reward_money: 17,
        base_reward_reputation: 2,
        deadline_window_minutes: 66,
    },
    TaskTemplate {
        kind: TaskKind::PermitReview,
        priority: TaskPriority::Critical,
        base_focus: 70,
        base_stress: 9,
        base_reward_money: 24,
        base_reward_reputation: 3,
        deadline_window_minutes: 126,
    },
    // New 12 — ClientCall, MeetingPrep, ReportWriting, BudgetReview
    TaskTemplate {
        kind: TaskKind::ClientCall,
        priority: TaskPriority::Low,
        base_focus: 20,
        base_stress: 1,
        base_reward_money: 7,
        base_reward_reputation: 0,
        deadline_window_minutes: 10,
    },
    TaskTemplate {
        kind: TaskKind::ClientCall,
        priority: TaskPriority::Medium,
        base_focus: 32,
        base_stress: 3,
        base_reward_money: 10,
        base_reward_reputation: 1,
        deadline_window_minutes: 24,
    },
    TaskTemplate {
        kind: TaskKind::ClientCall,
        priority: TaskPriority::High,
        base_focus: 48,
        base_stress: 5,
        base_reward_money: 15,
        base_reward_reputation: 2,
        deadline_window_minutes: 52,
    },
    TaskTemplate {
        kind: TaskKind::MeetingPrep,
        priority: TaskPriority::Medium,
        base_focus: 42,
        base_stress: 4,
        base_reward_money: 13,
        base_reward_reputation: 1,
        deadline_window_minutes: 38,
    },
    TaskTemplate {
        kind: TaskKind::MeetingPrep,
        priority: TaskPriority::High,
        base_focus: 60,
        base_stress: 7,
        base_reward_money: 19,
        base_reward_reputation: 3,
        deadline_window_minutes: 80,
    },
    TaskTemplate {
        kind: TaskKind::MeetingPrep,
        priority: TaskPriority::Critical,
        base_focus: 72,
        base_stress: 9,
        base_reward_money: 25,
        base_reward_reputation: 4,
        deadline_window_minutes: 130,
    },
    TaskTemplate {
        kind: TaskKind::ReportWriting,
        priority: TaskPriority::Low,
        base_focus: 28,
        base_stress: 2,
        base_reward_money: 10,
        base_reward_reputation: 1,
        deadline_window_minutes: 18,
    },
    TaskTemplate {
        kind: TaskKind::ReportWriting,
        priority: TaskPriority::Medium,
        base_focus: 36,
        base_stress: 3,
        base_reward_money: 11,
        base_reward_reputation: 1,
        deadline_window_minutes: 30,
    },
    TaskTemplate {
        kind: TaskKind::ReportWriting,
        priority: TaskPriority::High,
        base_focus: 54,
        base_stress: 6,
        base_reward_money: 17,
        base_reward_reputation: 2,
        deadline_window_minutes: 62,
    },
    TaskTemplate {
        kind: TaskKind::BudgetReview,
        priority: TaskPriority::Medium,
        base_focus: 46,
        base_stress: 5,
        base_reward_money: 14,
        base_reward_reputation: 1,
        deadline_window_minutes: 42,
    },
    TaskTemplate {
        kind: TaskKind::BudgetReview,
        priority: TaskPriority::High,
        base_focus: 62,
        base_stress: 7,
        base_reward_money: 20,
        base_reward_reputation: 3,
        deadline_window_minutes: 70,
    },
    TaskTemplate {
        kind: TaskKind::BudgetReview,
        priority: TaskPriority::Critical,
        base_focus: 74,
        base_stress: 10,
        base_reward_money: 26,
        base_reward_reputation: 4,
        deadline_window_minutes: 136,
    },
];

fn task_id_for_slot(day_number: u32, slot_index: u32) -> TaskId {
    TaskId(((day_number as u64) << 32) | (slot_index as u64 + 1))
}

fn task_template_for_slot(day_number: u32, slot_index: u32) -> TaskTemplate {
    // Deterministic content rotation across days and slots.
    let index = ((day_number as usize).wrapping_mul(7) + (slot_index as usize).wrapping_mul(13))
        % TASK_TEMPLATES.len();
    TASK_TEMPLATES[index]
}

fn day_difficulty_tier(day_number: u32) -> i32 {
    // Increase roughly every three days to keep progression readable.
    ((day_number.saturating_sub(1) / 3) as i32).min(10)
}

fn inbox_task(day_number: u32, slot_index: u32, day_end_minute: u32) -> OfficeTask {
    let template = task_template_for_slot(day_number, slot_index);
    let tier = day_difficulty_tier(day_number);
    let required_focus = template.base_focus + tier * 2 + (slot_index % 3) as i32;
    let stress_impact = template.base_stress + (tier / 3);
    let reward_money = template.base_reward_money + tier * 2 + template.priority.rank() as i32;
    let reward_reputation = template.base_reward_reputation + tier / 4;
    let deadline_window = template
        .deadline_window_minutes
        .saturating_add(tier.max(0) as u32 * 3);

    // Ensure deadline is at least 30 minutes after the implicit day start
    // to prevent tasks from being generated with already-expired deadlines.
    let day_start_minute = day_end_minute.saturating_sub(480);
    let raw_deadline = day_end_minute.saturating_sub(deadline_window);
    let deadline_minute = raw_deadline.max(day_start_minute + 30).min(u16::MAX as u32) as u16;

    OfficeTask {
        id: task_id_for_slot(day_number, slot_index),
        kind: template.kind,
        priority: template.priority,
        required_focus,
        stress_impact,
        reward_money,
        reward_reputation,
        deadline_minute,
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
