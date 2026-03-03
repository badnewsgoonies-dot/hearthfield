use bevy::prelude::*;

use crate::game::components::{InboxAvatar, OfficeWorker, WorkerAvatar};
use crate::game::events::{
    CoffeeBreakEvent, EndOfDayEvent, InterruptionEvent, PanicResponseEvent, ProcessInboxEvent,
    ResolveCalmlyEvent, WaitEvent,
};
use crate::game::resources::{
    format_clock, DayClock, DayStats, InboxState, OfficeRules, PlayerMindState,
};

pub fn setup_scene(
    mut commands: Commands,
    rules: Res<OfficeRules>,
    mut inbox: ResMut<InboxState>,
    mut clock: ResMut<DayClock>,
    mut mind: ResMut<PlayerMindState>,
    mut stats: ResMut<DayStats>,
) {
    inbox.remaining_items = rules.starting_inbox_items;
    clock.current_minute = rules.day_start_minute;
    clock.ended = false;
    mind.stress = rules.starting_stress.clamp(0, rules.max_stress);
    mind.focus = rules.starting_focus.clamp(0, rules.max_focus);
    mind.pending_interruptions = 0;
    stats.processed_items = 0;
    stats.coffee_breaks = 0;
    stats.wait_actions = 0;
    stats.failed_process_attempts = 0;
    stats.interruptions_triggered = 0;
    stats.calm_responses = 0;
    stats.panic_responses = 0;

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
        "Day {} starts at {} with {} inbox items. Stress: {}, focus: {}.",
        clock.day_number,
        format_clock(clock.current_minute),
        inbox.remaining_items,
        mind.stress,
        mind.focus
    );
}

pub fn collect_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    rules: Res<OfficeRules>,
    mut process_writer: EventWriter<ProcessInboxEvent>,
    mut coffee_writer: EventWriter<CoffeeBreakEvent>,
    mut interruption_writer: EventWriter<InterruptionEvent>,
    mut calm_writer: EventWriter<ResolveCalmlyEvent>,
    mut panic_writer: EventWriter<PanicResponseEvent>,
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
    if keyboard.just_pressed(KeyCode::KeyN) {
        wait_writer.send(WaitEvent {
            minutes: rules.wait_minutes,
        });
    }
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

pub fn handle_process_requests(
    mut requests: EventReader<ProcessInboxEvent>,
    rules: Res<OfficeRules>,
    mut clock: ResMut<DayClock>,
    mut inbox: ResMut<InboxState>,
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
    mut clock: ResMut<DayClock>,
    inbox: Res<InboxState>,
    stats: Res<DayStats>,
    mind: Res<PlayerMindState>,
    worker_query: Query<&OfficeWorker>,
    mut end_day_writer: EventWriter<EndOfDayEvent>,
) {
    if clock.ended {
        return;
    }

    let reached_shift_end = clock.current_minute >= rules.day_end_minute;
    let finished_all_work = inbox.remaining_items == 0;
    if !reached_shift_end && !finished_all_work {
        return;
    }

    clock.ended = true;
    let final_energy = worker_query.get_single().map_or(0, |worker| worker.energy);

    end_day_writer.send(EndOfDayEvent {
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
        final_energy,
        final_stress: mind.stress,
        final_focus: mind.focus,
    });
}

pub fn print_end_of_day_summary(mut events: EventReader<EndOfDayEvent>) {
    for summary in events.read() {
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
        println!(
            "Failed process attempts: {}",
            summary.failed_process_attempts
        );
        println!("Final energy: {}", summary.final_energy);
        println!("Final stress: {}", summary.final_stress);
        println!("Final focus: {}", summary.final_focus);
        println!("=============================================");
    }
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
