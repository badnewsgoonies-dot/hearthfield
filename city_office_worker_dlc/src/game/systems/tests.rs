use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::*;
use crate::game::events::{
    CoffeeBreakEvent, CoworkerHelpEvent, DayAdvanced, EndDayRequested, EndOfDayEvent,
    InterruptionEvent, ManagerCheckInEvent, PanicResponseEvent, ProcessInboxEvent,
    ResolveCalmlyEvent, WaitEvent,
};
use crate::game::resources::{
    DayClock, DayOutcome, DayStats, InboxState, OfficeRules, OfficeRunConfig, PlayerCareerState,
    PlayerMindState, TaskBoard, WorkerStats,
};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReplayDaySummary {
    day_number: u32,
    finished_minute: u32,
    processed_items: u32,
    remaining_items: u32,
    coffee_breaks: u32,
    wait_actions: u32,
    failed_process_attempts: u32,
    interruptions_triggered: u32,
    calm_responses: u32,
    panic_responses: u32,
    unresolved_interruptions: u32,
    manager_checkins: u32,
    coworker_helps: u32,
    final_energy: i32,
    final_stress: i32,
    final_focus: i32,
    final_reputation: i32,
}

#[derive(Resource, Default)]
struct ReplaySummaries(Vec<ReplayDaySummary>);

#[derive(Debug, Clone, Copy)]
enum ScriptAction {
    Process,
    Coffee,
    Wait(u32),
    InterruptionCalm,
    InterruptionPanic,
    ManagerCheckIn,
    CoworkerHelp,
}

#[derive(Clone, Copy)]
struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn action(&mut self) -> ScriptAction {
        match self.next_u32() % 7 {
            0 => ScriptAction::Process,
            1 => ScriptAction::Coffee,
            2 => ScriptAction::Wait(10 + (self.next_u32() % 16)),
            3 => ScriptAction::InterruptionCalm,
            4 => ScriptAction::InterruptionPanic,
            5 => ScriptAction::ManagerCheckIn,
            _ => ScriptAction::CoworkerHelp,
        }
    }
}

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

fn record_replay_summaries(
    mut events: EventReader<EndOfDayEvent>,
    mut summaries: ResMut<ReplaySummaries>,
) {
    for event in events.read() {
        summaries.0.push(ReplayDaySummary {
            day_number: event.day_number,
            finished_minute: event.finished_minute,
            processed_items: event.processed_items,
            remaining_items: event.remaining_items,
            coffee_breaks: event.coffee_breaks,
            wait_actions: event.wait_actions,
            failed_process_attempts: event.failed_process_attempts,
            interruptions_triggered: event.interruptions_triggered,
            calm_responses: event.calm_responses,
            panic_responses: event.panic_responses,
            unresolved_interruptions: event.unresolved_interruptions,
            manager_checkins: event.manager_checkins,
            coworker_helps: event.coworker_helps,
            final_energy: event.final_energy,
            final_stress: event.final_stress,
            final_focus: event.final_focus,
            final_reputation: event.final_reputation,
        });
    }
}

fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));

    app.init_state::<crate::game::OfficeGameState>()
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
        .init_resource::<LastAdvancedDay>()
        .init_resource::<ReplaySummaries>();

    let seeded_board = {
        let rules = app.world().resource::<OfficeRules>();
        let inbox = app.world().resource::<InboxState>();
        let clock = app.world().resource::<DayClock>();
        super::task_board::seed_task_board(
            clock.day_number,
            inbox.remaining_items,
            rules.day_end_minute,
        )
    };
    app.world_mut().insert_resource(seeded_board);

    let worker = app
        .world_mut()
        .spawn(crate::game::components::OfficeWorker { energy: 100 })
        .id();
    app.world_mut().insert_resource(TestWorkerEntity(worker));
    app
}

fn push_action(world: &mut World, action: ScriptAction) {
    match action {
        ScriptAction::Process => {
            world.send_event(ProcessInboxEvent);
        }
        ScriptAction::Coffee => {
            world.send_event(CoffeeBreakEvent);
        }
        ScriptAction::Wait(minutes) => {
            world.send_event(WaitEvent { minutes });
        }
        ScriptAction::InterruptionCalm => {
            world.send_event(InterruptionEvent);
            world.send_event(ResolveCalmlyEvent);
        }
        ScriptAction::InterruptionPanic => {
            world.send_event(InterruptionEvent);
            world.send_event(PanicResponseEvent);
        }
        ScriptAction::ManagerCheckIn => {
            world.send_event(ManagerCheckInEvent);
        }
        ScriptAction::CoworkerHelp => {
            world.send_event(CoworkerHelpEvent);
        }
    }
}

fn run_seeded_replay(seed: u64, days: usize, scripted: &[ScriptAction]) -> Vec<ReplayDaySummary> {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        (
            handle_wait_requests,
            handle_process_requests,
            handle_coffee_requests,
            handle_interruption_requests,
            handle_resolve_calmly_requests,
            handle_panic_response_requests,
            handle_manager_checkin_requests,
            handle_coworker_help_requests,
            check_end_of_day,
            finalize_end_day_request,
            apply_day_summary_rollover,
            transition_day_summary_to_inday,
            record_replay_summaries,
        )
            .chain(),
    );

    app.world_mut().resource_mut::<OfficeRunConfig>().seed = seed;

    let mut rng = Lcg::new(seed);
    let mut script_index = 0usize;
    let mut safety_ticks = 0usize;

    while app.world().resource::<ReplaySummaries>().0.len() < days && safety_ticks < 20_000 {
        let action = if scripted.is_empty() {
            rng.action()
        } else {
            let current = scripted[script_index % scripted.len()];
            script_index += 1;
            current
        };

        push_action(app.world_mut(), action);
        app.update();
        safety_ticks += 1;
    }

    app.world().resource::<ReplaySummaries>().0.clone()
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
        .get::<crate::game::components::OfficeWorker>(worker_entity)
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

#[test]
fn fixed_seed_three_day_replay_is_deterministic() {
    let script = [
        ScriptAction::Process,
        ScriptAction::Coffee,
        ScriptAction::InterruptionCalm,
        ScriptAction::ManagerCheckIn,
        ScriptAction::CoworkerHelp,
        ScriptAction::Wait(14),
    ];

    let run_a = run_seeded_replay(42, 3, &script);
    let run_b = run_seeded_replay(42, 3, &script);

    assert_eq!(run_a.len(), 3);
    assert_eq!(run_a, run_b);
}

#[test]
fn five_day_seeded_autoplay_completes_without_panic() {
    let summaries = run_seeded_replay(2026, 5, &[]);

    assert_eq!(summaries.len(), 5);
    let day_numbers: Vec<u32> = summaries.iter().map(|summary| summary.day_number).collect();
    assert_eq!(day_numbers, vec![1, 2, 3, 4, 5]);
    assert!(summaries
        .iter()
        .all(|summary| summary.finished_minute >= 9 * 60));
}
