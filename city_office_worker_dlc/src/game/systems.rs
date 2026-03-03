use bevy::prelude::*;

use crate::game::components::{InboxAvatar, OfficeWorker, WorkerAvatar};
use crate::game::events::{
    CoffeeBreakEvent, CoworkerHelpEvent, DayAdvanced, EndDayRequested, EndOfDayEvent,
    InterruptionEvent, ManagerCheckInEvent, PanicResponseEvent, ProcessInboxEvent,
    ResolveCalmlyEvent, WaitEvent,
};
use crate::game::resources::{
    format_clock, DayClock, DayOutcome, DayStats, InboxState, OfficeRules, OfficeRunConfig,
    OfficeTask, PlayerCareerState, PlayerMindState, TaskBoard, TaskId, TaskKind, TaskPriority,
    WorkerStats,
};
use crate::game::OfficeGameState;

fn task_id_for_slot(day_number: u32, slot_index: u32) -> TaskId {
    TaskId(((day_number as u64) << 32) | (slot_index as u64 + 1))
}

fn inbox_task(day_number: u32, slot_index: u32, day_end_minute: u32) -> OfficeTask {
    OfficeTask {
        id: task_id_for_slot(day_number, slot_index),
        kind: TaskKind::DataEntry,
        priority: TaskPriority::Medium,
        required_focus: 18,
        stress_impact: 3,
        reward_money: 12,
        reward_reputation: 1,
        deadline_minute: day_end_minute.min(u16::MAX as u32) as u16,
        progress: 0.0,
    }
}

fn seed_task_board(day_number: u32, inbox_items: u32, day_end_minute: u32) -> TaskBoard {
    TaskBoard {
        active: (0..inbox_items)
            .map(|index| inbox_task(day_number, index, day_end_minute))
            .collect(),
        completed_today: Vec::new(),
        failed_today: Vec::new(),
    }
}

fn sync_task_board_active_with_inbox(
    task_board: &mut TaskBoard,
    day_number: u32,
    inbox_items: u32,
    day_end_minute: u32,
) {
    task_board.normalize();
    let target_len = inbox_items as usize;

    while task_board.active.len() > target_len {
        if let Some(task) = task_board.active.pop() {
            task_board.completed_today.push(task.id);
        }
    }

    while task_board.active.len() < target_len {
        let next_slot = task_board.active.len() as u32;
        let _ = task_board.try_add_task(inbox_task(day_number, next_slot, day_end_minute));
    }

    task_board.normalize();
}

fn fail_remaining_task_board_work(task_board: &mut TaskBoard) {
    for task in task_board.active.drain(..) {
        task_board.failed_today.push(task.id);
    }
    task_board.normalize();
}

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

pub fn boot_to_main_menu(mut next_state: ResMut<NextState<OfficeGameState>>) {
    next_state.set(OfficeGameState::MainMenu);
}

pub fn main_menu_to_in_day(mut next_state: ResMut<NextState<OfficeGameState>>) {
    next_state.set(OfficeGameState::InDay);
}

pub fn toggle_pause(
    keyboard: Res<ButtonInput<KeyCode>>,
    game_state: Res<State<OfficeGameState>>,
    mut next_state: ResMut<NextState<OfficeGameState>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    match game_state.get() {
        OfficeGameState::InDay => next_state.set(OfficeGameState::Paused),
        OfficeGameState::Paused => next_state.set(OfficeGameState::InDay),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
pub fn setup_scene(
    mut commands: Commands,
    rules: Res<OfficeRules>,
    mut inbox: ResMut<InboxState>,
    mut clock: ResMut<DayClock>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
    mut stats: ResMut<DayStats>,
    task_board: Option<ResMut<TaskBoard>>,
) {
    inbox.remaining_items = rules.starting_inbox_items;
    clock.current_minute = rules.day_start_minute;
    clock.ended = false;
    mind.stress = rules.starting_stress.clamp(0, rules.max_stress);
    mind.focus = rules.starting_focus.clamp(0, rules.max_focus);
    mind.pending_interruptions = 0;
    career.reputation = rules
        .starting_reputation
        .clamp(-rules.max_reputation, rules.max_reputation);
    stats.processed_items = 0;
    stats.coffee_breaks = 0;
    stats.wait_actions = 0;
    stats.failed_process_attempts = 0;
    stats.interruptions_triggered = 0;
    stats.calm_responses = 0;
    stats.panic_responses = 0;
    stats.manager_checkins = 0;
    stats.coworker_helps = 0;

    let seeded_board = seed_task_board(
        clock.day_number,
        inbox.remaining_items,
        rules.day_end_minute,
    );
    if let Some(mut existing_task_board) = task_board {
        *existing_task_board = seeded_board;
    } else {
        commands.insert_resource(seeded_board);
    }

    commands.spawn((Camera2d,));
    commands.spawn((
        OfficeWorker {
            energy: rules.max_energy,
        },
        WorkerAvatar,
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.4), Vec2::new(130.0, 130.0)),
        Transform::from_xyz(-180.0, 0.0, 0.0),
    ));
    commands.spawn((
        InboxAvatar,
        Sprite::from_color(Color::srgb(0.8, 0.75, 0.2), Vec2::new(120.0, 120.0)),
        Transform::from_xyz(190.0, 0.0, 0.0),
    ));

    println!(
        "Day {} starts at {} with {} inbox items. Stress: {}, focus: {}, reputation: {}.",
        clock.day_number,
        format_clock(clock.current_minute),
        inbox.remaining_items,
        mind.stress,
        mind.focus,
        career.reputation
    );
}

#[allow(clippy::too_many_arguments)]
pub fn collect_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    rules: Res<OfficeRules>,
    mut process_writer: EventWriter<ProcessInboxEvent>,
    mut coffee_writer: EventWriter<CoffeeBreakEvent>,
    mut interruption_writer: EventWriter<InterruptionEvent>,
    mut calm_writer: EventWriter<ResolveCalmlyEvent>,
    mut panic_writer: EventWriter<PanicResponseEvent>,
    mut manager_writer: EventWriter<ManagerCheckInEvent>,
    mut coworker_writer: EventWriter<CoworkerHelpEvent>,
    mut wait_writer: EventWriter<WaitEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        process_writer.send(ProcessInboxEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        coffee_writer.send(CoffeeBreakEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyI) {
        interruption_writer.send(InterruptionEvent);
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        calm_writer.send(ResolveCalmlyEvent);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        panic_writer.send(PanicResponseEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        manager_writer.send(ManagerCheckInEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyH) {
        coworker_writer.send(CoworkerHelpEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        wait_writer.send(WaitEvent {
            minutes: rules.wait_minutes,
        });
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

pub fn handle_interruption_requests(
    mut requests: EventReader<InterruptionEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        clock.advance(rules.interruption_minutes);
        mind.stress = (mind.stress + rules.interruption_stress_increase).clamp(0, rules.max_stress);
        mind.focus = (mind.focus - rules.interruption_focus_loss).clamp(0, rules.max_focus);
        mind.pending_interruptions = mind.pending_interruptions.saturating_add(1);
        stats.interruptions_triggered = stats.interruptions_triggered.saturating_add(1);

        info!(
            "Interruption! stress: {}, focus: {}, pending: {}, clock: {}",
            mind.stress,
            mind.focus,
            mind.pending_interruptions,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_resolve_calmly_requests(
    mut requests: EventReader<ResolveCalmlyEvent>,
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        if mind.pending_interruptions == 0 {
            info!("No interruption is pending; calm response ignored.");
            continue;
        }

        let before_focus = mind.focus;
        let before_stress = mind.stress;

        mind.pending_interruptions -= 1;
        mind.focus = (mind.focus + rules.calm_focus_restore).clamp(0, rules.max_focus);
        mind.stress = (mind.stress - rules.calm_stress_relief).clamp(0, rules.max_stress);
        stats.calm_responses = stats.calm_responses.saturating_add(1);

        info!(
            "Resolved calmly -> focus: {} -> {}, stress: {} -> {}, pending: {}",
            before_focus, mind.focus, before_stress, mind.stress, mind.pending_interruptions
        );
    }
}

pub fn handle_panic_response_requests(
    mut requests: EventReader<PanicResponseEvent>,
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        if mind.pending_interruptions == 0 {
            info!("No interruption is pending; panic response ignored.");
            continue;
        }

        let before_focus = mind.focus;
        let before_stress = mind.stress;

        mind.pending_interruptions -= 1;
        mind.focus = (mind.focus - rules.panic_focus_loss).clamp(0, rules.max_focus);
        mind.stress = (mind.stress + rules.panic_stress_increase).clamp(0, rules.max_stress);
        stats.panic_responses = stats.panic_responses.saturating_add(1);

        info!(
            "Panicked response -> focus: {} -> {}, stress: {} -> {}, pending: {}",
            before_focus, mind.focus, before_stress, mind.stress, mind.pending_interruptions
        );
    }
}

pub fn handle_manager_checkin_requests(
    mut requests: EventReader<ManagerCheckInEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        let before_stress = mind.stress;
        let before_rep = career.reputation;

        clock.advance(rules.manager_checkin_minutes);
        mind.stress =
            (mind.stress + rules.manager_checkin_stress_increase).clamp(0, rules.max_stress);
        career.reputation = (career.reputation + rules.manager_checkin_reputation_gain)
            .clamp(-rules.max_reputation, rules.max_reputation);
        stats.manager_checkins = stats.manager_checkins.saturating_add(1);

        info!(
            "Manager check-in -> stress: {} -> {}, reputation: {} -> {}, clock: {}",
            before_stress,
            mind.stress,
            before_rep,
            career.reputation,
            format_clock(clock.current_minute)
        );
    }
}

pub fn handle_coworker_help_requests(
    mut requests: EventReader<CoworkerHelpEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut stats: ResMut<DayStats>,
    mut mind: ResMut<PlayerMindState>,
    mut career: ResMut<PlayerCareerState>,
) {
    for _ in requests.read() {
        if clock.ended {
            continue;
        }

        let before_focus = mind.focus;
        let before_stress = mind.stress;
        let before_rep = career.reputation;

        clock.advance(rules.coworker_help_minutes);
        mind.focus = (mind.focus - rules.coworker_help_focus_cost).clamp(0, rules.max_focus);
        mind.stress = (mind.stress - rules.coworker_help_stress_relief).clamp(0, rules.max_stress);
        career.reputation = (career.reputation + rules.coworker_help_reputation_gain)
            .clamp(-rules.max_reputation, rules.max_reputation);
        stats.coworker_helps = stats.coworker_helps.saturating_add(1);

        info!(
            "Helped coworker -> focus: {} -> {}, stress: {} -> {}, reputation: {} -> {}, clock: {}",
            before_focus,
            mind.focus,
            before_stress,
            mind.stress,
            before_rep,
            career.reputation,
            format_clock(clock.current_minute)
        );
    }
}

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

pub fn update_visuals(
    rules: Res<OfficeRules>,
    clock: Res<DayClock>,
    inbox: Res<InboxState>,
    worker_query: Query<&OfficeWorker>,
    mut worker_avatar_query: Query<&mut Sprite, With<WorkerAvatar>>,
    mut inbox_avatar_query: Query<(&mut Sprite, &mut Transform), With<InboxAvatar>>,
    mut clear_color: ResMut<ClearColor>,
) {
    if let Ok(worker) = worker_query.get_single() {
        let energy_ratio = (worker.energy as f32 / rules.max_energy as f32).clamp(0.0, 1.0);
        if let Ok(mut sprite) = worker_avatar_query.get_single_mut() {
            sprite.color = Color::srgb(1.0 - energy_ratio, 0.2 + (0.7 * energy_ratio), 0.25);
        }
    }

    if let Ok((mut sprite, mut transform)) = inbox_avatar_query.get_single_mut() {
        let inbox_ratio = if rules.starting_inbox_items == 0 {
            0.0
        } else {
            (inbox.remaining_items as f32 / rules.starting_inbox_items as f32).clamp(0.0, 1.0)
        };
        let scale = 0.35 + (inbox_ratio * 1.15);
        transform.scale = Vec3::splat(scale);
        sprite.color = Color::srgb(0.4 + (0.5 * inbox_ratio), 0.35 + (0.45 * inbox_ratio), 0.2);
    }

    let shift_duration = (rules.day_end_minute - rules.day_start_minute).max(1);
    let elapsed = clock.current_minute.saturating_sub(rules.day_start_minute);
    let progress = (elapsed as f32 / shift_duration as f32).clamp(0.0, 1.0);

    let tone = 0.2 - (0.11 * progress);
    clear_color.0 = Color::srgb(tone, tone * 0.95, tone * 1.1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    #[derive(Resource, Copy, Clone)]
    struct TestWorkerEntity(Entity);

    #[derive(Resource, Default)]
    struct EndEventCount(u32);

    #[derive(Resource, Default)]
    struct EndDayRequestedCount(u32);

    #[derive(Resource, Default)]
    struct DayAdvancedCount(u32);

    #[derive(Resource, Default)]
    struct LastAdvancedDay(Option<u32>);

    fn count_end_of_day_events(
        mut events: EventReader<EndOfDayEvent>,
        mut count: ResMut<EndEventCount>,
    ) {
        for _ in events.read() {
            count.0 += 1;
        }
    }

    fn count_end_day_requested_events(
        mut events: EventReader<EndDayRequested>,
        mut count: ResMut<EndDayRequestedCount>,
    ) {
        for _ in events.read() {
            count.0 += 1;
        }
    }

    fn count_day_advanced_events(
        mut events: EventReader<DayAdvanced>,
        mut count: ResMut<DayAdvancedCount>,
        mut last_day: ResMut<LastAdvancedDay>,
    ) {
        for event in events.read() {
            count.0 += 1;
            last_day.0 = Some(event.new_day_index);
        }
    }

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));

        app.init_state::<OfficeGameState>()
            .init_resource::<OfficeRules>()
            .init_resource::<OfficeRunConfig>()
            .init_resource::<InboxState>()
            .init_resource::<DayClock>()
            .init_resource::<WorkerStats>()
            .init_resource::<PlayerMindState>()
            .init_resource::<PlayerCareerState>()
            .init_resource::<DayOutcome>()
            .init_resource::<DayStats>()
            .init_resource::<TaskBoard>()
            .add_event::<EndDayRequested>()
            .add_event::<DayAdvanced>()
            .add_event::<ProcessInboxEvent>()
            .add_event::<CoffeeBreakEvent>()
            .add_event::<InterruptionEvent>()
            .add_event::<ResolveCalmlyEvent>()
            .add_event::<PanicResponseEvent>()
            .add_event::<ManagerCheckInEvent>()
            .add_event::<CoworkerHelpEvent>()
            .add_event::<WaitEvent>()
            .add_event::<EndOfDayEvent>()
            .init_resource::<EndEventCount>()
            .init_resource::<EndDayRequestedCount>()
            .init_resource::<DayAdvancedCount>()
            .init_resource::<LastAdvancedDay>();

        let seeded_board = {
            let rules = app.world().resource::<OfficeRules>();
            let inbox = app.world().resource::<InboxState>();
            let clock = app.world().resource::<DayClock>();
            seed_task_board(
                clock.day_number,
                inbox.remaining_items,
                rules.day_end_minute,
            )
        };
        app.world_mut().insert_resource(seeded_board);

        let worker = app.world_mut().spawn(OfficeWorker { energy: 100 }).id();
        app.world_mut().insert_resource(TestWorkerEntity(worker));
        app
    }

    #[test]
    fn process_event_updates_energy_inbox_and_clock() {
        let mut app = build_test_app();
        app.add_systems(Update, handle_process_requests);

        app.world_mut().send_event(ProcessInboxEvent);
        app.update();

        let worker_entity = app.world().resource::<TestWorkerEntity>().0;
        let worker = app
            .world()
            .get::<OfficeWorker>(worker_entity)
            .expect("worker should exist");
        let inbox = app.world().resource::<InboxState>();
        let clock = app.world().resource::<DayClock>();
        let stats = app.world().resource::<DayStats>();
        let board = app.world().resource::<TaskBoard>();

        assert_eq!(worker.energy, 88);
        assert_eq!(inbox.remaining_items, 17);
        assert_eq!(clock.current_minute, 9 * 60 + 15);
        assert_eq!(stats.processed_items, 1);
        assert_eq!(board.active.len(), 17);
        assert_eq!(board.completed_today.len(), 1);
        assert_eq!(board.failed_today.len(), 0);
    }

    #[test]
    fn interruption_and_npc_pressure_events_are_deterministic() {
        let mut app = build_test_app();
        app.add_systems(
            Update,
            (
                handle_interruption_requests,
                handle_resolve_calmly_requests,
                handle_panic_response_requests,
                handle_manager_checkin_requests,
                handle_coworker_help_requests,
            ),
        );

        app.world_mut().send_event(InterruptionEvent);
        app.update();
        app.world_mut().send_event(ResolveCalmlyEvent);
        app.update();
        app.world_mut().send_event(InterruptionEvent);
        app.update();
        app.world_mut().send_event(PanicResponseEvent);
        app.update();
        app.world_mut().send_event(ManagerCheckInEvent);
        app.update();
        app.world_mut().send_event(CoworkerHelpEvent);
        app.update();

        let mind = app.world().resource::<PlayerMindState>();
        let career = app.world().resource::<PlayerCareerState>();
        let stats = app.world().resource::<DayStats>();

        assert_eq!(mind.pending_interruptions, 0);
        assert_eq!(stats.interruptions_triggered, 2);
        assert_eq!(stats.calm_responses, 1);
        assert_eq!(stats.panic_responses, 1);
        assert_eq!(stats.manager_checkins, 1);
        assert_eq!(stats.coworker_helps, 1);
        assert_eq!(career.reputation, 5);
        assert_eq!(mind.stress, 54);
        assert_eq!(mind.focus, 55);
    }

    #[test]
    fn end_day_request_advances_once_and_emits_summary_once() {
        let mut app = build_test_app();
        app.add_systems(
            Update,
            (
                check_end_of_day,
                finalize_end_day_request,
                count_end_day_requested_events,
                count_day_advanced_events,
                count_end_of_day_events,
            )
                .chain(),
        );

        let day_end = app.world().resource::<OfficeRules>().day_end_minute;
        {
            let mut clock = app.world_mut().resource_mut::<DayClock>();
            clock.current_minute = day_end;
            clock.ended = false;
        }

        app.update();
        assert_eq!(app.world().resource::<EndDayRequestedCount>().0, 1);
        assert_eq!(app.world().resource::<DayAdvancedCount>().0, 1);
        assert_eq!(app.world().resource::<EndEventCount>().0, 1);
        assert_eq!(app.world().resource::<LastAdvancedDay>().0, Some(2));

        app.update();
        assert_eq!(app.world().resource::<EndDayRequestedCount>().0, 1);
        assert_eq!(app.world().resource::<DayAdvancedCount>().0, 1);
        assert_eq!(app.world().resource::<EndEventCount>().0, 1);
        assert_eq!(app.world().resource::<DayClock>().day_number, 1);
        assert!(app.world().resource::<DayClock>().ended);
    }

    #[test]
    fn day_advanced_does_not_emit_without_end_day_request() {
        let mut app = build_test_app();
        app.add_systems(
            Update,
            (finalize_end_day_request, count_day_advanced_events).chain(),
        );

        app.update();
        app.update();

        assert_eq!(app.world().resource::<DayAdvancedCount>().0, 0);
    }

    #[test]
    fn duplicate_end_day_requests_are_debounced() {
        let mut app = build_test_app();
        app.add_systems(
            Update,
            (
                finalize_end_day_request,
                count_day_advanced_events,
                count_end_of_day_events,
            )
                .chain(),
        );

        app.world_mut().send_event(EndDayRequested);
        app.world_mut().send_event(EndDayRequested);
        app.update();

        assert_eq!(app.world().resource::<DayAdvancedCount>().0, 1);
        assert_eq!(app.world().resource::<EndEventCount>().0, 1);
        assert_eq!(app.world().resource::<LastAdvancedDay>().0, Some(2));

        app.world_mut().send_event(EndDayRequested);
        app.update();

        assert_eq!(app.world().resource::<DayAdvancedCount>().0, 1);
        assert_eq!(app.world().resource::<EndEventCount>().0, 1);
    }

    #[test]
    fn day_summary_rollover_applies_and_transitions_back_to_inday() {
        let mut app = build_test_app();
        app.add_systems(
            Update,
            (
                finalize_end_day_request,
                apply_day_summary_rollover,
                transition_day_summary_to_inday,
            )
                .chain(),
        );

        let rules = app.world().resource::<OfficeRules>();
        let day_end = rules.day_end_minute;
        let day_start = rules.day_start_minute;
        let starting_inbox = rules.starting_inbox_items;

        {
            let mut clock = app.world_mut().resource_mut::<DayClock>();
            clock.current_minute = day_end;
            clock.ended = false;
        }
        {
            let mut stats = app.world_mut().resource_mut::<DayStats>();
            stats.processed_items = 4;
            stats.manager_checkins = 1;
        }
        {
            let mut inbox = app.world_mut().resource_mut::<InboxState>();
            inbox.remaining_items = 14;
        }
        app.world_mut().send_event(EndDayRequested);

        app.update();

        let clock = app.world().resource::<DayClock>();
        let inbox = app.world().resource::<InboxState>();
        let stats = app.world().resource::<DayStats>();
        let worker_stats = app.world().resource::<WorkerStats>();

        assert_eq!(clock.day_number, 2);
        assert_eq!(clock.current_minute, day_start);
        assert!(!clock.ended);
        assert_eq!(inbox.remaining_items, starting_inbox);
        assert_eq!(stats.processed_items, 0);
        assert_ne!(worker_stats.money, 0);
    }
}
