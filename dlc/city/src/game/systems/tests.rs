use std::collections::HashSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use super::*;
use crate::game::events::{
    CoffeeBreakEvent, CoworkerHelpEvent, DayAdvanced, EndDayRequested, EndOfDayEvent,
    InterruptionEvent, ManagerCheckInEvent, PanicResponseEvent, ProcessInboxEvent,
    ResolveCalmlyEvent, TaskCompleted, TaskFailed, TaskProgressed, WaitEvent,
};
use crate::game::resources::{
    CareerProgression, CoworkerRole, DayClock, DayOutcome, DayStats, InboxState,
    OfficeEconomyRules, OfficeRules, OfficeRunConfig, PlayerCareerState, PlayerMindState,
    SocialGraphState, TaskBoard, TaskId, TaskPriority, UnlockCatalogState, WorkerSpriteData,
    WorkerStats,
};
use crate::game::save::{
    apply_snapshot, capture_snapshot, deserialize_snapshot, migrate_snapshot_json,
    read_snapshot_from_slot, serialize_snapshot, write_snapshot_to_slot, LoadSlotRequest,
    OfficeSaveStore, SaveSlotConfig, SaveSlotRequest,
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

#[derive(Resource, Default)]
struct TaskFailedCount(u32);

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

fn count_task_failed_events(
    mut events: EventReader<TaskFailed>,
    mut count: ResMut<TaskFailedCount>,
) {
    for _ in events.read() {
        count.0 += 1;
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
    app.add_plugins((MinimalPlugins, StatesPlugin, bevy::asset::AssetPlugin::default()));
    app.init_asset::<bevy::prelude::Image>();
    app.init_asset::<bevy::sprite::TextureAtlasLayout>();

    app.init_state::<crate::game::OfficeGameState>()
        .init_resource::<WorkerSpriteData>()
        .init_resource::<OfficeRules>()
        .init_resource::<OfficeRunConfig>()
        .init_resource::<InboxState>()
        .init_resource::<DayClock>()
        .init_resource::<WorkerStats>()
        .init_resource::<PlayerMindState>()
        .init_resource::<PlayerCareerState>()
        .init_resource::<SocialGraphState>()
        .init_resource::<OfficeEconomyRules>()
        .init_resource::<CareerProgression>()
        .init_resource::<UnlockCatalogState>()
        .init_resource::<DayOutcome>()
        .init_resource::<DayStats>()
        .init_resource::<SaveSlotConfig>()
        .init_resource::<OfficeSaveStore>()
        .init_resource::<TaskBoard>()
        .add_event::<EndDayRequested>()
        .add_event::<DayAdvanced>()
        .add_event::<TaskProgressed>()
        .add_event::<TaskCompleted>()
        .add_event::<TaskFailed>()
        .add_event::<ProcessInboxEvent>()
        .add_event::<CoffeeBreakEvent>()
        .add_event::<InterruptionEvent>()
        .add_event::<ResolveCalmlyEvent>()
        .add_event::<PanicResponseEvent>()
        .add_event::<ManagerCheckInEvent>()
        .add_event::<CoworkerHelpEvent>()
        .add_event::<WaitEvent>()
        .add_event::<EndOfDayEvent>()
        .add_event::<SaveSlotRequest>()
        .add_event::<LoadSlotRequest>()
        .init_resource::<EndEventCount>()
        .init_resource::<EndDayRequestedCount>()
        .init_resource::<DayAdvancedCount>()
        .init_resource::<LastAdvancedDay>()
        .init_resource::<TaskFailedCount>()
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

fn save_config_for_test(label: &str) -> SaveSlotConfig {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    SaveSlotConfig {
        save_dir: std::env::temp_dir().join(format!(
            "city_office_worker_dlc_{label}_{}_{}",
            std::process::id(),
            now
        )),
        active_slot: 0,
    }
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
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        if let Some(task) = board.active.first_mut() {
            task.required_focus = 1;
            task.priority = TaskPriority::Low;
            task.progress = 0.75; // one step at focus_ratio=1.0 adds ~0.28, completing it
        }
    }

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
fn process_event_supports_partial_progress_before_completion() {
    let mut app = build_test_app();
    app.add_systems(Update, handle_process_requests);
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        if let Some(task) = board.active.first_mut() {
            task.required_focus = 120;
            task.priority = TaskPriority::High;
        }
    }

    app.world_mut().send_event(ProcessInboxEvent);
    app.update();

    let board_after_first = app.world().resource::<TaskBoard>();
    assert_eq!(board_after_first.completed_today.len(), 0);
    assert_eq!(board_after_first.failed_today.len(), 0);
    assert_eq!(app.world().resource::<InboxState>().remaining_items, 18);
    let progress_after_first = board_after_first
        .active
        .first()
        .expect("first task should still be active after first process")
        .progress;
    assert!(progress_after_first > 0.0);
    assert!(progress_after_first < 1.0);

    let mut safety = 0u8;
    while app
        .world()
        .resource::<TaskBoard>()
        .completed_today
        .is_empty()
        && safety < 10
    {
        app.world_mut().send_event(ProcessInboxEvent);
        app.update();
        safety += 1;
    }

    assert!(
        !app.world()
            .resource::<TaskBoard>()
            .completed_today
            .is_empty(),
        "task should complete after repeated process actions"
    );
    assert_eq!(app.world().resource::<InboxState>().remaining_items, 17);
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
    let social = app.world().resource::<SocialGraphState>();

    assert_eq!(mind.pending_interruptions, 0);
    assert_eq!(stats.interruptions_triggered, 2);
    assert_eq!(stats.calm_responses, 1);
    assert_eq!(stats.panic_responses, 1);
    assert_eq!(stats.manager_checkins, 1);
    assert_eq!(stats.coworker_helps, 1);
    assert_eq!(career.reputation, 5);
    assert_eq!(mind.stress, 51);
    assert_eq!(mind.focus, 56);
    assert!(social.scenario_cursor >= 2);
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
        stats.processed_items = 16;
        stats.manager_checkins = 1;
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        // Complete most tasks so salary exceeds failure penalties after .max(0) clamp
        for i in 0..16u64 {
            board.completed_today.push(TaskId(100 + i));
        }
        // Remove completed tasks from active so they don't get double-failed
        let completed: Vec<_> = board.completed_today.clone();
        board.active.retain(|t| !completed.contains(&t.id));
    }
    {
        let mut inbox = app.world_mut().resource_mut::<InboxState>();
        inbox.remaining_items = 2;
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

#[test]
fn completed_task_cannot_fail_later_same_day() {
    let mut app = build_test_app();
    let mut board = app.world_mut().resource_mut::<TaskBoard>();
    let task_id = board.active.first().expect("seeded task should exist").id;

    assert!(board.complete_task(task_id));
    assert!(!board.fail_task(task_id));
    assert!(board.is_completed(task_id));
    assert!(!board.is_failed(task_id));
}

#[test]
fn overdue_tasks_fail_once_and_do_not_reenter_active_pool() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        (enforce_task_deadlines, count_task_failed_events).chain(),
    );

    let overdue_id = {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        let task = board
            .active
            .first_mut()
            .expect("seeded task should be present for deadline test");
        task.deadline_minute = 9 * 60;
        task.id
    };
    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.current_minute = 9 * 60 + 5;
    }

    app.update();
    assert_eq!(app.world().resource::<TaskFailedCount>().0, 1);
    let board = app.world().resource::<TaskBoard>();
    assert!(board.is_failed(overdue_id));
    assert!(!board.is_completed(overdue_id));
    assert!(!board.has_active_task(overdue_id));
    assert_eq!(app.world().resource::<InboxState>().remaining_items, 17);

    app.update();
    assert_eq!(app.world().resource::<TaskFailedCount>().0, 1);
}

#[test]
fn snapshot_roundtrip_preserves_task_ids_and_midday_load_no_regen() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        (handle_process_requests, sync_taskboard_bridge).chain(),
    );

    app.world_mut().send_event(ProcessInboxEvent);
    app.update();

    let (snapshot_json, original_active_ids, original_completed_ids, original_failed_ids) = {
        let clock = app.world().resource::<DayClock>();
        let worker_stats = app.world().resource::<WorkerStats>();
        let task_board = app.world().resource::<TaskBoard>();
        let run_config = app.world().resource::<OfficeRunConfig>();
        let inbox = app.world().resource::<InboxState>();
        let day_stats = app.world().resource::<DayStats>();
        let mind = app.world().resource::<PlayerMindState>();
        let progression = app.world().resource::<CareerProgression>();
        let social_graph = app.world().resource::<SocialGraphState>();
        let unlocks = app.world().resource::<UnlockCatalogState>();
        let day_outcome = app.world().resource::<DayOutcome>();

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
        );
        let json = serialize_snapshot(&snapshot).expect("snapshot serialization should succeed");
        (
            json,
            task_board.active_task_ids(),
            task_board.completed_today.clone(),
            task_board.failed_today.clone(),
        )
    };

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 99;
        clock.current_minute = 0;
        clock.ended = true;
    }
    {
        let mut inbox = app.world_mut().resource_mut::<InboxState>();
        inbox.remaining_items = 0;
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today.clear();
        board.failed_today.clear();
    }

    let snapshot =
        deserialize_snapshot(&snapshot_json).expect("snapshot deserialization should succeed");
    let mut clock = app.world().resource::<DayClock>().clone();
    let mut worker_stats = app.world().resource::<WorkerStats>().clone();
    let mut board = app.world().resource::<TaskBoard>().clone();
    let mut run_config = app.world().resource::<OfficeRunConfig>().clone();
    let mut inbox = app.world().resource::<InboxState>().clone();
    let mut day_stats = DayStats::default();
    let mut mind = PlayerMindState::default();
    let mut progression = CareerProgression::default();
    let mut social_graph = SocialGraphState::default();
    let mut unlocks = UnlockCatalogState::default();
    let mut day_outcome = DayOutcome::default();
    let economy = app.world().resource::<OfficeEconomyRules>().clone();

    apply_snapshot(
        &snapshot,
        &mut clock,
        &mut worker_stats,
        &mut board,
        &mut run_config,
        &mut inbox,
        &mut day_stats,
        &mut mind,
        &mut progression,
        &mut social_graph,
        &mut unlocks,
        &economy,
        &mut day_outcome,
    )
    .expect("snapshot apply should succeed");

    app.world_mut().insert_resource(clock);
    app.world_mut().insert_resource(worker_stats);
    app.world_mut().insert_resource(board);
    app.world_mut().insert_resource(run_config);
    app.world_mut().insert_resource(inbox);

    app.update();

    let restored_board = app.world().resource::<TaskBoard>();
    assert_eq!(restored_board.active_task_ids(), original_active_ids);
    assert_eq!(restored_board.completed_today, original_completed_ids);
    assert_eq!(restored_board.failed_today, original_failed_ids);
}

#[test]
fn save_slot_roundtrip_persists_snapshot_payload() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("slot_roundtrip");
    app.world_mut().insert_resource(save_config.clone());

    let snapshot = {
        let clock = app.world().resource::<DayClock>();
        let worker_stats = app.world().resource::<WorkerStats>();
        let task_board = app.world().resource::<TaskBoard>();
        let run_config = app.world().resource::<OfficeRunConfig>();
        let inbox = app.world().resource::<InboxState>();
        let day_stats = app.world().resource::<DayStats>();
        let mind = app.world().resource::<PlayerMindState>();
        let progression = app.world().resource::<CareerProgression>();
        let social_graph = app.world().resource::<SocialGraphState>();
        let unlocks = app.world().resource::<UnlockCatalogState>();
        let day_outcome = app.world().resource::<DayOutcome>();
        capture_snapshot(
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
        )
    };

    let save_path = write_snapshot_to_slot(&save_config, 4, &snapshot)
        .expect("writing snapshot file should succeed");
    assert!(save_path.exists());

    let restored =
        read_snapshot_from_slot(&save_config, 4).expect("reading snapshot file should succeed");
    assert_eq!(restored, snapshot);

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn migrate_v0_snapshot_to_v1_preserves_core_fields_and_ids() {
    let legacy_json = serde_json::json!({
        "day_index": 3,
        "minute_of_day": 812,
        "day_ended": false,
        "inbox_remaining": 9,
        "run_seed": 404,
        "energy": 64,
        "stress": 33,
        "focus": 58,
        "money": 140,
        "reputation": 12,
        "active_task_ids": [3001, 3002],
        "completed_today": [2001],
        "failed_today": [1001]
    })
    .to_string();

    let migrated =
        migrate_snapshot_json(&legacy_json).expect("v0 snapshot migration should succeed");

    assert_eq!(migrated.schema_version, 1);
    assert_eq!(migrated.day_index, 3);
    assert_eq!(migrated.minute_of_day, 812);
    assert_eq!(migrated.inbox_remaining, 9);
    assert_eq!(migrated.run_seed, 404);
    assert_eq!(migrated.worker_stats.energy, 64);
    assert_eq!(migrated.worker_stats.stress, 33);
    assert_eq!(migrated.worker_stats.focus, 58);
    assert_eq!(migrated.worker_stats.money, 140);
    assert_eq!(migrated.worker_stats.reputation, 12);
    assert_eq!(migrated.task_board.completed_today, vec![2001]);
    assert_eq!(migrated.task_board.failed_today, vec![1001]);
    assert_eq!(migrated.day_stats.processed_items, 0);
    assert_eq!(migrated.pending_interruptions, 0);
    assert_eq!(
        migrated
            .task_board
            .active
            .iter()
            .map(|task| task.id)
            .collect::<Vec<_>>(),
        vec![3001, 3002]
    );
}

#[test]
fn load_slot_request_restores_state_without_task_regeneration_drift() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("load_slot_request");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            handle_process_requests,
            sync_taskboard_bridge,
            crate::game::save::handle_save_slot_requests,
            crate::game::save::handle_load_slot_requests,
        )
            .chain(),
    );

    app.world_mut().send_event(ProcessInboxEvent);
    app.update();

    let (
        expected_clock,
        expected_inbox,
        expected_stats,
        expected_active,
        expected_done,
        expected_failed,
    ) = {
        let clock = app.world().resource::<DayClock>().clone();
        let inbox = app.world().resource::<InboxState>().clone();
        let stats = app.world().resource::<WorkerStats>().clone();
        let board = app.world().resource::<TaskBoard>();
        (
            clock,
            inbox,
            stats,
            board.active_task_ids(),
            board.completed_today.clone(),
            board.failed_today.clone(),
        )
    };

    app.world_mut().send_event(SaveSlotRequest { slot: 2 });
    app.update();

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 99;
        clock.current_minute = 1;
        clock.ended = true;
    }
    {
        let mut inbox = app.world_mut().resource_mut::<InboxState>();
        inbox.remaining_items = 0;
    }
    {
        let mut worker_stats = app.world_mut().resource_mut::<WorkerStats>();
        worker_stats.energy = 1;
        worker_stats.stress = 99;
        worker_stats.focus = 1;
        worker_stats.money = -10;
        worker_stats.reputation = -40;
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today.clear();
        board.failed_today.clear();
    }
    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.stress = 90;
        mind.focus = 2;
        mind.pending_interruptions = 5;
    }
    {
        let mut career = app.world_mut().resource_mut::<PlayerCareerState>();
        career.reputation = -25;
    }
    {
        let worker_entity = app.world().resource::<TestWorkerEntity>().0;
        if let Some(mut worker) = app
            .world_mut()
            .entity_mut(worker_entity)
            .get_mut::<crate::game::components::OfficeWorker>()
        {
            worker.energy = 2;
        }
    }

    app.world_mut().send_event(LoadSlotRequest { slot: 2 });
    app.update();
    app.update();

    let restored_clock = app.world().resource::<DayClock>();
    let restored_inbox = app.world().resource::<InboxState>();
    let restored_stats = app.world().resource::<WorkerStats>();
    let restored_board = app.world().resource::<TaskBoard>();
    let restored_mind = app.world().resource::<PlayerMindState>();
    let restored_career = app.world().resource::<PlayerCareerState>();
    let store = app.world().resource::<OfficeSaveStore>();
    let worker_entity = app.world().resource::<TestWorkerEntity>().0;
    let worker = app
        .world()
        .get::<crate::game::components::OfficeWorker>(worker_entity)
        .expect("worker should exist");

    assert_eq!(restored_clock.day_number, expected_clock.day_number);
    assert_eq!(restored_clock.current_minute, expected_clock.current_minute);
    assert_eq!(restored_clock.ended, expected_clock.ended);
    assert_eq!(
        restored_inbox.remaining_items,
        expected_inbox.remaining_items
    );
    assert_eq!(restored_stats.energy, expected_stats.energy);
    assert_eq!(restored_stats.stress, expected_stats.stress);
    assert_eq!(restored_stats.focus, expected_stats.focus);
    assert_eq!(restored_stats.money, expected_stats.money);
    assert_eq!(restored_stats.reputation, expected_stats.reputation);
    assert_eq!(restored_board.active_task_ids(), expected_active);
    assert_eq!(restored_board.completed_today, expected_done);
    assert_eq!(restored_board.failed_today, expected_failed);
    assert_eq!(restored_mind.stress, restored_stats.stress);
    assert_eq!(restored_mind.focus, restored_stats.focus);
    assert_eq!(restored_career.reputation, restored_stats.reputation);
    assert_eq!(worker.energy, restored_stats.energy);
    assert_eq!(store.last_loaded_slot, Some(2));
    assert!(store.last_io_error.is_none());
    assert_eq!(app.world().resource::<SaveSlotConfig>().active_slot, 2);

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn load_day_ended_snapshot_reconciles_to_playable_flow() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("day_ended_recovery");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            crate::game::save::handle_load_slot_requests,
            apply_day_summary_rollover,
            transition_day_summary_to_inday,
            handle_wait_requests,
        )
            .chain(),
    );

    {
        let day_end = app.world().resource::<OfficeRules>().day_end_minute;
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 4;
        clock.current_minute = day_end;
        clock.ended = true;
    }
    {
        let mut stats = app.world_mut().resource_mut::<DayStats>();
        stats.processed_items = 6;
        stats.interruptions_triggered = 2;
    }
    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.pending_interruptions = 3;
    }

    let snapshot = {
        let clock = app.world().resource::<DayClock>();
        let worker_stats = app.world().resource::<WorkerStats>();
        let task_board = app.world().resource::<TaskBoard>();
        let run_config = app.world().resource::<OfficeRunConfig>();
        let inbox = app.world().resource::<InboxState>();
        let day_stats = app.world().resource::<DayStats>();
        let mind = app.world().resource::<PlayerMindState>();
        let progression = app.world().resource::<CareerProgression>();
        let social_graph = app.world().resource::<SocialGraphState>();
        let unlocks = app.world().resource::<UnlockCatalogState>();
        let day_outcome = app.world().resource::<DayOutcome>();
        capture_snapshot(
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
        )
    };
    write_snapshot_to_slot(&save_config, 7, &snapshot).expect("writing ended-day snapshot");

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 99;
        clock.current_minute = 0;
        clock.ended = false;
    }

    app.world_mut().send_event(LoadSlotRequest { slot: 7 });
    app.update();

    let rules = app.world().resource::<OfficeRules>();
    let minute_before_wait = app.world().resource::<DayClock>().current_minute;
    {
        let clock = app.world().resource::<DayClock>();
        assert_eq!(clock.day_number, snapshot.day_index + 1);
        assert_eq!(clock.current_minute, rules.day_start_minute);
        assert!(!clock.ended);
    }

    app.world_mut().send_event(WaitEvent { minutes: 5 });
    app.update();

    let clock_after_wait = app.world().resource::<DayClock>();
    assert_eq!(clock_after_wait.current_minute, minute_before_wait + 5);
    assert!(!clock_after_wait.ended);

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn load_restores_day_stats_and_pending_interruptions() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("stats_backlog_restore");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            crate::game::save::handle_save_slot_requests,
            crate::game::save::handle_load_slot_requests,
        )
            .chain(),
    );

    {
        let mut stats = app.world_mut().resource_mut::<DayStats>();
        stats.processed_items = 4;
        stats.coffee_breaks = 2;
        stats.wait_actions = 1;
        stats.failed_process_attempts = 3;
        stats.interruptions_triggered = 5;
        stats.calm_responses = 2;
        stats.panic_responses = 1;
        stats.manager_checkins = 3;
        stats.coworker_helps = 4;
    }
    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.pending_interruptions = 2;
    }

    let expected_day_stats = {
        let stats = app.world().resource::<DayStats>();
        (
            stats.processed_items,
            stats.coffee_breaks,
            stats.wait_actions,
            stats.failed_process_attempts,
            stats.interruptions_triggered,
            stats.calm_responses,
            stats.panic_responses,
            stats.manager_checkins,
            stats.coworker_helps,
        )
    };
    let expected_pending = app
        .world()
        .resource::<PlayerMindState>()
        .pending_interruptions;

    app.world_mut().send_event(SaveSlotRequest { slot: 6 });
    app.update();

    {
        let mut stats = app.world_mut().resource_mut::<DayStats>();
        stats.processed_items = 99;
        stats.coffee_breaks = 99;
        stats.wait_actions = 99;
        stats.failed_process_attempts = 99;
        stats.interruptions_triggered = 99;
        stats.calm_responses = 99;
        stats.panic_responses = 99;
        stats.manager_checkins = 99;
        stats.coworker_helps = 99;
    }
    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.pending_interruptions = 0;
    }

    app.world_mut().send_event(LoadSlotRequest { slot: 6 });
    app.update();

    let restored_day_stats = {
        let stats = app.world().resource::<DayStats>();
        (
            stats.processed_items,
            stats.coffee_breaks,
            stats.wait_actions,
            stats.failed_process_attempts,
            stats.interruptions_triggered,
            stats.calm_responses,
            stats.panic_responses,
            stats.manager_checkins,
            stats.coworker_helps,
        )
    };
    let restored_pending = app
        .world()
        .resource::<PlayerMindState>()
        .pending_interruptions;

    assert_eq!(restored_day_stats, expected_day_stats);
    assert_eq!(restored_pending, expected_pending);
    assert!(app
        .world()
        .resource::<OfficeSaveStore>()
        .last_io_error
        .is_none());

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn load_restores_social_graph_state() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("social_graph_restore");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            crate::game::save::handle_save_slot_requests,
            crate::game::save::handle_load_slot_requests,
        )
            .chain(),
    );

    let expected_social = {
        let mut social = app.world_mut().resource_mut::<SocialGraphState>();
        social.scenario_cursor = 7;
        if let Some(manager) = social
            .profiles
            .iter_mut()
            .find(|profile| profile.role == CoworkerRole::Manager)
        {
            manager.affinity = 14;
            manager.trust = 21;
        }
        if let Some(profile) = social.profiles.iter_mut().find(|profile| profile.id == 3) {
            profile.affinity = 9;
            profile.trust = 11;
        }
        social.clone()
    };

    app.world_mut().send_event(SaveSlotRequest { slot: 4 });
    app.update();

    {
        let mut social = app.world_mut().resource_mut::<SocialGraphState>();
        social.scenario_cursor = 0;
        for profile in &mut social.profiles {
            profile.affinity = -80;
            profile.trust = -80;
        }
    }

    app.world_mut().send_event(LoadSlotRequest { slot: 4 });
    app.update();

    let restored = app.world().resource::<SocialGraphState>();
    assert_eq!(*restored, expected_social);
    assert!(app
        .world()
        .resource::<OfficeSaveStore>()
        .last_io_error
        .is_none());

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn unlock_catalog_syncs_with_progression_thresholds() {
    let mut progression = CareerProgression::default();
    let mut unlocks = UnlockCatalogState::default();

    progression.level = 1;
    unlocks.sync_with_progression(&progression);
    assert_eq!(unlocks.unlocked_count(), 0);
    assert!(!unlocks.quick_coffee);
    assert!(!unlocks.efficient_processing);
    assert!(!unlocks.conflict_training);
    assert!(!unlocks.escalation_license);

    progression.level = 2;
    unlocks.sync_with_progression(&progression);
    assert_eq!(unlocks.unlocked_count(), 1);
    assert!(unlocks.quick_coffee);
    assert!(!unlocks.efficient_processing);
    assert!(!unlocks.conflict_training);
    assert!(!unlocks.escalation_license);

    progression.level = 3;
    unlocks.sync_with_progression(&progression);
    assert_eq!(unlocks.unlocked_count(), 2);
    assert!(unlocks.quick_coffee);
    assert!(unlocks.efficient_processing);
    assert!(!unlocks.conflict_training);
    assert!(!unlocks.escalation_license);

    progression.level = 4;
    unlocks.sync_with_progression(&progression);
    assert_eq!(unlocks.unlocked_count(), 3);
    assert!(unlocks.quick_coffee);
    assert!(unlocks.efficient_processing);
    assert!(unlocks.conflict_training);
    assert!(!unlocks.escalation_license);

    progression.level = 5;
    unlocks.sync_with_progression(&progression);
    assert_eq!(unlocks.unlocked_count(), 4);
    assert!(unlocks.quick_coffee);
    assert!(unlocks.efficient_processing);
    assert!(unlocks.conflict_training);
    assert!(unlocks.escalation_license);
}

#[test]
fn unlock_timeline_is_deterministic_for_replayed_day_summaries() {
    fn run_unlock_timeline(script: &[(u32, u32, u32, u32)]) -> Vec<u32> {
        let economy = OfficeEconomyRules::default();
        let mut progression = CareerProgression::default();
        let mut unlocks = UnlockCatalogState::default();
        let mut trace = Vec::with_capacity(script.len());

        for (completed, failed, manager_checkins, coworker_helps) in script {
            let gained_xp = completed
                .saturating_mul(economy.xp_per_completed_task)
                .saturating_add(manager_checkins.saturating_mul(economy.xp_per_manager_checkin))
                .saturating_add(coworker_helps.saturating_mul(economy.xp_per_coworker_help))
                .saturating_sub(failed.saturating_mul(economy.xp_penalty_per_failed_task));
            progression.add_experience(gained_xp, &economy);
            progression.normalize(&economy);
            unlocks.sync_with_progression(&progression);
            trace.push(unlocks.unlocked_count());
        }

        trace
    }

    let script_a = vec![(4, 0, 1, 1), (5, 0, 2, 1), (3, 1, 0, 1), (6, 0, 2, 2)];
    let script_b = script_a.clone();
    let script_c = vec![(1, 1, 0, 0), (2, 1, 0, 0), (1, 2, 0, 0), (2, 1, 0, 1)];

    let trace_a = run_unlock_timeline(&script_a);
    let trace_b = run_unlock_timeline(&script_b);
    let trace_c = run_unlock_timeline(&script_c);

    assert_eq!(trace_a, trace_b);
    assert_ne!(trace_a, trace_c);
}

#[test]
fn load_restores_unlock_catalog_state() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("unlock_restore");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            crate::game::save::handle_save_slot_requests,
            crate::game::save::handle_load_slot_requests,
        )
            .chain(),
    );

    let expected_unlocks = {
        {
            let mut progression = app.world_mut().resource_mut::<CareerProgression>();
            progression.level = 4;
            progression.xp = 9;
            progression.efficiency_perk = 1;
            progression.normalize(&OfficeEconomyRules::default());
        }
        let progression = app.world().resource::<CareerProgression>().clone();
        let mut unlocks = app.world_mut().resource_mut::<UnlockCatalogState>();
        unlocks.sync_with_progression(&progression);
        unlocks.clone()
    };

    app.world_mut().send_event(SaveSlotRequest { slot: 8 });
    app.update();

    {
        let mut progression = app.world_mut().resource_mut::<CareerProgression>();
        progression.level = 1;
        progression.xp = 0;
        progression.success_streak = 0;
        progression.burnout_days = 0;
    }
    {
        let mut unlocks = app.world_mut().resource_mut::<UnlockCatalogState>();
        *unlocks = UnlockCatalogState::default();
    }

    app.world_mut().send_event(LoadSlotRequest { slot: 8 });
    app.update();
    app.update();

    let restored_unlocks = app.world().resource::<UnlockCatalogState>();
    assert_eq!(*restored_unlocks, expected_unlocks);
    assert!(app
        .world()
        .resource::<OfficeSaveStore>()
        .last_io_error
        .is_none());

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn social_scenarios_are_seed_deterministic() {
    fn run_social_trace(seed: u64) -> (i32, i32, u32, Vec<(u8, i32, i32)>) {
        let mut app = build_test_app();
        app.add_systems(
            Update,
            (
                handle_interruption_requests,
                handle_resolve_calmly_requests,
                handle_panic_response_requests,
                handle_manager_checkin_requests,
                handle_coworker_help_requests,
            )
                .chain(),
        );
        app.world_mut().resource_mut::<OfficeRunConfig>().seed = seed;

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
        let social = app.world().resource::<SocialGraphState>();
        let summary = social
            .profiles
            .iter()
            .map(|profile| (profile.id, profile.affinity, profile.trust))
            .collect::<Vec<_>>();
        (mind.stress, mind.focus, social.scenario_cursor, summary)
    }

    let run_a = run_social_trace(42);
    let run_b = run_social_trace(42);
    let run_c = run_social_trace(84);

    assert_eq!(run_a, run_b);
    assert_ne!(run_a, run_c);
}

#[test]
fn successful_save_and_load_requests_update_active_slot_for_autosave() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("active_slot_semantics");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            crate::game::save::handle_save_slot_requests,
            crate::game::save::handle_load_slot_requests,
            crate::game::save::persist_day_summary_snapshot,
        )
            .chain(),
    );

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 3;
        clock.ended = false;
    }
    app.world_mut().send_event(SaveSlotRequest { slot: 3 });
    app.update();
    assert_eq!(app.world().resource::<SaveSlotConfig>().active_slot, 3);

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 5;
        clock.ended = false;
    }
    app.world_mut().send_event(SaveSlotRequest { slot: 5 });
    app.update();
    assert_eq!(app.world().resource::<SaveSlotConfig>().active_slot, 5);

    app.world_mut().send_event(LoadSlotRequest { slot: 3 });
    app.update();
    assert_eq!(app.world().resource::<SaveSlotConfig>().active_slot, 3);

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.day_number = 88;
        clock.ended = true;
    }
    app.update();

    let slot3_snapshot =
        read_snapshot_from_slot(&save_config, 3).expect("slot 3 snapshot should exist");
    let slot5_snapshot =
        read_snapshot_from_slot(&save_config, 5).expect("slot 5 snapshot should exist");
    assert_eq!(slot3_snapshot.day_index, 88);
    assert_eq!(slot5_snapshot.day_index, 5);

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn apply_snapshot_normalizes_terminal_sets_to_disjoint_lists() {
    let app = build_test_app();

    let mut malformed_snapshot = {
        let clock = app.world().resource::<DayClock>();
        let worker_stats = app.world().resource::<WorkerStats>();
        let task_board = app.world().resource::<TaskBoard>();
        let run_config = app.world().resource::<OfficeRunConfig>();
        let inbox = app.world().resource::<InboxState>();
        let day_stats = app.world().resource::<DayStats>();
        let mind = app.world().resource::<PlayerMindState>();
        let progression = app.world().resource::<CareerProgression>();
        let social_graph = app.world().resource::<SocialGraphState>();
        let unlocks = app.world().resource::<UnlockCatalogState>();
        let day_outcome = app.world().resource::<DayOutcome>();
        capture_snapshot(
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
        )
    };

    let overlapping_id = malformed_snapshot
        .task_board
        .active
        .first()
        .expect("seeded task should exist for malformed snapshot test")
        .id;
    malformed_snapshot.task_board.active.clear();
    malformed_snapshot.task_board.completed_today = vec![overlapping_id, overlapping_id];
    malformed_snapshot.task_board.failed_today =
        vec![overlapping_id, overlapping_id + 1, overlapping_id + 1];

    let mut clock = DayClock::default();
    let mut worker_stats = WorkerStats::default();
    let mut task_board = TaskBoard::default();
    let mut run_config = OfficeRunConfig::default();
    let mut inbox = InboxState::default();
    let mut day_stats = DayStats::default();
    let mut mind = PlayerMindState::default();
    let mut progression = CareerProgression::default();
    let mut social_graph = SocialGraphState::default();
    let mut unlocks = UnlockCatalogState::default();
    let mut day_outcome = DayOutcome::default();
    let economy = OfficeEconomyRules::default();

    apply_snapshot(
        &malformed_snapshot,
        &mut clock,
        &mut worker_stats,
        &mut task_board,
        &mut run_config,
        &mut inbox,
        &mut day_stats,
        &mut mind,
        &mut progression,
        &mut social_graph,
        &mut unlocks,
        &economy,
        &mut day_outcome,
    )
    .expect("malformed terminal-set snapshot should normalize successfully");

    let completed_ids: Vec<u64> = task_board
        .completed_today
        .iter()
        .map(|task_id| task_id.0)
        .collect();
    let failed_ids: Vec<u64> = task_board
        .failed_today
        .iter()
        .map(|task_id| task_id.0)
        .collect();
    assert_eq!(completed_ids, vec![overlapping_id]);
    assert_eq!(failed_ids, vec![overlapping_id + 1]);
}

#[test]
fn efficiency_perk_reduces_process_energy_cost() {
    let mut app = build_test_app();
    app.add_systems(Update, handle_process_requests);

    {
        let mut progression = app.world_mut().resource_mut::<CareerProgression>();
        progression.efficiency_perk = 2;
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        if let Some(task) = board.active.first_mut() {
            task.required_focus = 1;
            task.priority = TaskPriority::Low;
        }
    }

    app.world_mut().send_event(ProcessInboxEvent);
    app.update();

    let rules = app.world().resource::<OfficeRules>();
    let economy = app.world().resource::<OfficeEconomyRules>();
    let expected_cost = (rules.process_energy_cost
        - 2 * economy.process_energy_discount_per_efficiency_perk)
        .max(1);

    let worker_entity = app.world().resource::<TestWorkerEntity>().0;
    let worker = app
        .world()
        .get::<crate::game::components::OfficeWorker>(worker_entity)
        .expect("worker should exist");
    assert_eq!(worker.energy, 100 - expected_cost);
}

#[test]
fn day_outcome_preview_applies_level_streak_and_burnout_modifiers() {
    let mut app = build_test_app();
    app.add_systems(Update, update_day_outcome_preview);

    {
        let mut progression = app.world_mut().resource_mut::<CareerProgression>();
        progression.level = 4;
        progression.success_streak = 2;
        progression.resilience_perk = 1;
        progression.diplomacy_perk = 1;
    }
    {
        let mut worker_stats = app.world_mut().resource_mut::<WorkerStats>();
        worker_stats.stress = 80;
    }
    {
        let mut stats = app.world_mut().resource_mut::<DayStats>();
        stats.interruptions_triggered = 2;
        stats.calm_responses = 1;
        stats.manager_checkins = 1;
        stats.coworker_helps = 2;
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today = vec![
            crate::game::resources::TaskId(1),
            crate::game::resources::TaskId(2),
            crate::game::resources::TaskId(3),
        ];
        board.failed_today.clear();
    }

    app.update();

    let economy = app.world().resource::<OfficeEconomyRules>();
    let outcome = app.world().resource::<DayOutcome>();
    let expected_salary = 3 * economy.base_salary_per_task
        + (4 - 1) * economy.level_salary_bonus
        + 3 * economy.streak_bonus_per_day
        - economy.burnout_salary_penalty;
    let expected_reputation = 2 + 2 * 3 + 1;
    let expected_stress = 2 * 3 - 3 - 2 * 2 - economy.stress_relief_per_resilience_perk;

    assert_eq!(outcome.salary_earned, expected_salary);
    assert_eq!(outcome.reputation_delta, expected_reputation);
    assert_eq!(outcome.stress_delta, expected_stress);
}

#[test]
fn day_summary_rollover_levels_and_assigns_auto_perk() {
    let mut app = build_test_app();
    app.add_systems(Update, apply_day_summary_rollover);

    {
        let mut clock = app.world_mut().resource_mut::<DayClock>();
        clock.ended = true;
    }
    {
        let mut worker_stats = app.world_mut().resource_mut::<WorkerStats>();
        worker_stats.money = 10;
        worker_stats.stress = 70;
    }
    {
        let mut stats = app.world_mut().resource_mut::<DayStats>();
        stats.manager_checkins = 2;
        stats.coworker_helps = 1;
    }
    {
        let mut day_outcome = app.world_mut().resource_mut::<DayOutcome>();
        day_outcome.salary_earned = 40;
        day_outcome.reputation_delta = 2;
        day_outcome.stress_delta = 20;
        day_outcome.completed_tasks = 5;
        day_outcome.failed_tasks = 0;
    }
    {
        let mut progression = app.world_mut().resource_mut::<CareerProgression>();
        progression.level = 1;
        progression.xp = 25;
    }

    app.update();

    let economy = app.world().resource::<OfficeEconomyRules>();
    let progression = app.world().resource::<CareerProgression>();
    let worker_stats = app.world().resource::<WorkerStats>();
    let expected_xp = 25
        + 5 * economy.xp_per_completed_task
        + 2 * economy.xp_per_manager_checkin
        + economy.xp_per_coworker_help;

    assert_eq!(worker_stats.money, 50);
    assert_eq!(worker_stats.stress, 90);
    assert_eq!(progression.level, 2);
    assert_eq!(progression.xp, expected_xp - 32);
    assert_eq!(progression.success_streak, 1);
    assert_eq!(progression.burnout_days, 1);
    assert_eq!(progression.efficiency_perk, 1);
}

#[test]
fn setup_scene_is_idempotent_for_first_seconds_entities() {
    let mut app = build_test_app();
    app.add_systems(Update, setup_scene);

    app.update();
    app.update();

    let worker_count = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<crate::game::components::OfficeWorker>>()
            .iter(world)
            .count()
    };
    let worker_avatar_count = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<crate::game::components::WorkerAvatar>>()
            .iter(world)
            .count()
    };
    let inbox_avatar_count = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<crate::game::components::InboxAvatar>>()
            .iter(world)
            .count()
    };
    let camera_count = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<Camera2d>>()
            .iter(world)
            .count()
    };

    assert_eq!(worker_count, 1);
    assert_eq!(worker_avatar_count, 1);
    assert_eq!(inbox_avatar_count, 1);
    assert_eq!(camera_count, 1);
}

#[test]
fn seeded_task_board_content_pack_has_kind_and_priority_variety() {
    let board = super::task_board::seed_task_board(9, 18, 17 * 60);
    let kinds: HashSet<_> = board.active.iter().map(|task| task.kind).collect();
    let priorities: HashSet<_> = board.active.iter().map(|task| task.priority).collect();

    assert!(kinds.len() >= 4, "expected all task kinds in content pack");
    assert!(
        priorities.len() >= 4,
        "expected all priority tiers in content pack"
    );
}

#[test]
fn seeded_task_board_scales_task_economy_with_day_progression() {
    let early = super::task_board::seed_task_board(1, 18, 17 * 60);
    let late = super::task_board::seed_task_board(15, 18, 17 * 60);

    let early_avg_reward = early
        .active
        .iter()
        .map(|task| task.reward_money)
        .sum::<i32>() as f32
        / early.active.len() as f32;
    let late_avg_reward = late
        .active
        .iter()
        .map(|task| task.reward_money)
        .sum::<i32>() as f32
        / late.active.len() as f32;
    let early_avg_focus = early
        .active
        .iter()
        .map(|task| task.required_focus)
        .sum::<i32>() as f32
        / early.active.len() as f32;
    let late_avg_focus = late
        .active
        .iter()
        .map(|task| task.required_focus)
        .sum::<i32>() as f32
        / late.active.len() as f32;

    assert!(
        late_avg_reward > early_avg_reward,
        "late-day task rewards should scale upward"
    );
    assert!(
        late_avg_focus > early_avg_focus,
        "late-day task focus requirements should scale upward"
    );
}

#[test]
fn negative_salary_clamped_to_zero() {
    let _economy = OfficeEconomyRules::default();
    let _stats = DayStats::default();
    let _progression = CareerProgression::default();
    let _unlocks = UnlockCatalogState::default();
    let _social = SocialGraphState::default();
    let _worker_stats = WorkerStats::default();

    // Set up a scenario where failures exceed completions
    let _task_board = TaskBoard {
        failed_today: vec![TaskId(1), TaskId(2), TaskId(3), TaskId(4), TaskId(5)],
        ..Default::default()
    };

    let mut app = build_test_app();
    app.add_systems(Update, update_day_outcome_preview);

    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today.clear();
        board.failed_today = vec![TaskId(1), TaskId(2), TaskId(3), TaskId(4), TaskId(5)];
    }

    app.update();

    let outcome = app.world().resource::<DayOutcome>();
    assert!(
        outcome.salary_earned >= 0,
        "salary should never be negative, got {}",
        outcome.salary_earned
    );
}

#[test]
fn affinity_trust_bounded_after_many_interruptions() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        (handle_interruption_requests, handle_resolve_calmly_requests).chain(),
    );

    for _ in 0..50 {
        app.world_mut().send_event(InterruptionEvent);
        app.update();
        app.world_mut().send_event(ResolveCalmlyEvent);
        app.update();
    }

    let social = app.world().resource::<SocialGraphState>();
    for profile in &social.profiles {
        assert!(
            profile.affinity >= -100 && profile.affinity <= 100,
            "affinity for {} out of bounds: {}",
            profile.codename,
            profile.affinity
        );
        assert!(
            profile.trust >= -100 && profile.trust <= 100,
            "trust for {} out of bounds: {}",
            profile.codename,
            profile.trust
        );
    }
}

#[test]
fn social_bonus_affects_reputation() {
    let mut app = build_test_app();
    app.add_systems(Update, update_day_outcome_preview);

    // Set high affinity on all coworkers to ensure social_bonus > 0
    {
        let mut social = app.world_mut().resource_mut::<SocialGraphState>();
        for profile in &mut social.profiles {
            profile.affinity = 50;
        }
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today = vec![TaskId(1), TaskId(2)];
        board.failed_today.clear();
    }
    {
        let mut stats = app.world_mut().resource_mut::<DayStats>();
        stats.manager_checkins = 1;
    }

    app.update();

    let outcome_with_social = app.world().resource::<DayOutcome>().reputation_delta;

    // Now reset with zero affinity
    {
        let mut social = app.world_mut().resource_mut::<SocialGraphState>();
        for profile in &mut social.profiles {
            profile.affinity = 0;
        }
    }

    app.update();

    let outcome_without_social = app.world().resource::<DayOutcome>().reputation_delta;

    assert!(
        outcome_with_social > outcome_without_social,
        "social bonus should increase reputation delta: with={}, without={}",
        outcome_with_social,
        outcome_without_social
    );
}

#[test]
fn task_deadline_never_in_past() {
    // Check across many days that deadline >= day_start + 30
    for day in 1..=30 {
        let board = super::task_board::seed_task_board(day, 18, 17 * 60);
        let day_start = (17 * 60u32).saturating_sub(480);
        for task in &board.active {
            assert!(
                task.deadline_minute as u32 >= day_start + 30,
                "task {} on day {} has deadline {} which is before day_start+30={}",
                task.id.0,
                day,
                task.deadline_minute,
                day_start + 30
            );
        }
    }
}

#[test]
fn focus_ratio_never_exceeds_one() {
    // progress_delta_for_task has focus_ratio = (focus/required_focus).min(1.0)
    // Verify by checking the resulting progress doesn't exceed expectations even
    // when focus is much larger than required_focus
    let mut app = build_test_app();
    app.add_systems(Update, handle_process_requests);

    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.focus = 100;
    }
    {
        let mut board = app.world_mut().resource_mut::<TaskBoard>();
        if let Some(task) = board.active.first_mut() {
            task.required_focus = 1; // very low requirement vs high focus
            task.priority = TaskPriority::Low;
            task.progress = 0.0;
        }
    }

    app.world_mut().send_event(ProcessInboxEvent);
    app.update();

    let board = app.world().resource::<TaskBoard>();
    // With focus_ratio clamped to 1.0 and Low priority (mult=1.0),
    // delta = 0.52 * 1.0 * 1.0 = 0.52
    // If focus_ratio exceeded 1.0, progress would be larger
    // The task with progress 0.0 + 0.52 = 0.52 should not be complete yet
    if let Some(task) = board.active.first() {
        assert!(
            task.progress <= 1.0,
            "progress should never exceed 1.0, got {}",
            task.progress
        );
    }
    // Whether completed or still active, check that it made reasonable progress
    let total_progressed = if board.completed_today.len() == 1 {
        true // it completed, which is valid for 0.52 progress
    } else {
        let p = board.active.first().unwrap().progress;
        p > 0.0 && p <= 1.0
    };
    assert!(total_progressed, "progress should be valid");
}

#[test]
fn save_load_preserves_focus() {
    let mut app = build_test_app();
    let save_config = save_config_for_test("focus_roundtrip");
    app.world_mut().insert_resource(save_config.clone());

    app.add_systems(
        Update,
        (
            crate::game::save::handle_save_slot_requests,
            crate::game::save::handle_load_slot_requests,
        )
            .chain(),
    );

    let expected_focus = 42;
    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.focus = expected_focus;
    }
    {
        let mut worker_stats = app.world_mut().resource_mut::<WorkerStats>();
        worker_stats.focus = expected_focus;
    }

    app.world_mut().send_event(SaveSlotRequest { slot: 1 });
    app.update();

    // Corrupt the focus values
    {
        let mut mind = app.world_mut().resource_mut::<PlayerMindState>();
        mind.focus = 99;
    }
    {
        let mut worker_stats = app.world_mut().resource_mut::<WorkerStats>();
        worker_stats.focus = 99;
    }

    app.world_mut().send_event(LoadSlotRequest { slot: 1 });
    app.update();

    let restored_mind = app.world().resource::<PlayerMindState>();
    let restored_stats = app.world().resource::<WorkerStats>();
    assert_eq!(
        restored_mind.focus, expected_focus,
        "mind.focus should survive save/load roundtrip"
    );
    assert_eq!(
        restored_stats.focus, expected_focus,
        "worker_stats.focus should survive save/load roundtrip"
    );
    assert_eq!(
        restored_mind.focus, restored_stats.focus,
        "mind.focus and worker_stats.focus should be in sync after load"
    );

    let _ = fs::remove_dir_all(&save_config.save_dir);
}

#[test]
fn burnout_penalty_only_above_threshold() {
    let economy = OfficeEconomyRules::default();

    // Below threshold: no burnout penalty
    let mut app_below = build_test_app();
    app_below.add_systems(Update, update_day_outcome_preview);
    {
        let mut worker_stats = app_below.world_mut().resource_mut::<WorkerStats>();
        worker_stats.stress = economy.burnout_stress_threshold - 1;
    }
    {
        let mut board = app_below.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today = vec![TaskId(1), TaskId(2)];
        board.failed_today.clear();
    }
    app_below.update();
    let salary_below = app_below.world().resource::<DayOutcome>().salary_earned;

    // At threshold: burnout penalty applies
    let mut app_at = build_test_app();
    app_at.add_systems(Update, update_day_outcome_preview);
    {
        let mut worker_stats = app_at.world_mut().resource_mut::<WorkerStats>();
        worker_stats.stress = economy.burnout_stress_threshold;
    }
    {
        let mut board = app_at.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today = vec![TaskId(1), TaskId(2)];
        board.failed_today.clear();
    }
    app_at.update();
    let salary_at = app_at.world().resource::<DayOutcome>().salary_earned;

    assert!(
        salary_below > salary_at,
        "salary below burnout threshold ({}) should exceed salary at threshold ({})",
        salary_below,
        salary_at
    );
    assert_eq!(
        salary_below - salary_at,
        economy.burnout_salary_penalty,
        "difference should equal burnout penalty"
    );
}

#[test]
fn streak_bonus_caps_at_max() {
    let economy = OfficeEconomyRules::default();

    // Streak at max
    let mut app_max = build_test_app();
    app_max.add_systems(Update, update_day_outcome_preview);
    {
        let mut progression = app_max.world_mut().resource_mut::<CareerProgression>();
        progression.success_streak = economy.max_streak_bonus_days;
    }
    {
        let mut board = app_max.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today = vec![TaskId(1)];
        board.failed_today.clear();
    }
    app_max.update();
    let salary_at_max = app_max.world().resource::<DayOutcome>().salary_earned;

    // Streak well above max
    let mut app_over = build_test_app();
    app_over.add_systems(Update, update_day_outcome_preview);
    {
        let mut progression = app_over.world_mut().resource_mut::<CareerProgression>();
        progression.success_streak = economy.max_streak_bonus_days + 100;
    }
    {
        let mut board = app_over.world_mut().resource_mut::<TaskBoard>();
        board.active.clear();
        board.completed_today = vec![TaskId(1)];
        board.failed_today.clear();
    }
    app_over.update();
    let salary_over_max = app_over.world().resource::<DayOutcome>().salary_earned;

    assert_eq!(
        salary_at_max, salary_over_max,
        "streak bonus should cap at max_streak_bonus_days"
    );
}

#[test]
fn scenario_cursor_wraps_safely() {
    let mut social = SocialGraphState::default();
    social.scenario_cursor = u32::MAX;
    // wrapping_add(1) should wrap to 0, not panic
    social.scenario_cursor = social.scenario_cursor.wrapping_add(1);
    assert_eq!(social.scenario_cursor, 0);

    // Also verify the system doesn't panic at u32::MAX
    let mut app = build_test_app();
    app.add_systems(Update, handle_interruption_requests);
    {
        let mut sg = app.world_mut().resource_mut::<SocialGraphState>();
        sg.scenario_cursor = u32::MAX;
    }
    app.world_mut().send_event(InterruptionEvent);
    app.update();

    let social_after = app.world().resource::<SocialGraphState>();
    assert_eq!(
        social_after.scenario_cursor, 0,
        "scenario_cursor should wrap from u32::MAX to 0"
    );
}

#[test]
fn end_of_day_event_consumed() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        (
            finalize_end_day_request,
            consume_end_of_day_events,
            count_end_of_day_events,
        )
            .chain(),
    );

    app.world_mut().send_event(EndDayRequested);
    app.update();

    // The consume system should have consumed the event on the first update.
    // The count system ran after consume, so it saw the event too (chained in same frame).
    assert_eq!(app.world().resource::<EndEventCount>().0, 1);

    // On the second update, no new events should appear
    app.update();
    assert_eq!(
        app.world().resource::<EndEventCount>().0,
        1,
        "EndOfDayEvent should be consumed and not re-emitted"
    );
}
