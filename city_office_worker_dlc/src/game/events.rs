use crate::game::resources::TaskId;
use bevy::prelude::*;

#[derive(Event, Debug, Default)]
pub struct ProcessInboxEvent;

#[derive(Event, Debug, Default)]
pub struct CoffeeBreakEvent;

#[derive(Event, Debug, Default)]
pub struct InterruptionEvent;

#[derive(Event, Debug, Default)]
pub struct ResolveCalmlyEvent;

#[derive(Event, Debug, Default)]
pub struct PanicResponseEvent;

#[derive(Event, Debug, Default)]
pub struct ManagerCheckInEvent;

#[derive(Event, Debug, Default)]
pub struct CoworkerHelpEvent;

#[derive(Event, Debug)]
pub struct WaitEvent {
    pub minutes: u32,
}

#[derive(Event, Debug, Default)]
pub struct EndDayRequested;

#[derive(Event, Debug, Clone, Copy)]
pub struct DayAdvanced {
    pub new_day_index: u32,
}

#[derive(Event, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TaskAccepted {
    pub task_id: TaskId,
}

#[derive(Event, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TaskProgressed {
    pub task_id: TaskId,
    pub delta: f32,
}

#[derive(Event, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TaskCompleted {
    pub task_id: TaskId,
}

#[derive(Event, Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct TaskFailed {
    pub task_id: TaskId,
}

#[derive(Event, Debug)]
pub struct EndOfDayEvent {
    pub day_number: u32,
    pub finished_minute: u32,
    pub processed_items: u32,
    pub remaining_items: u32,
    pub coffee_breaks: u32,
    pub wait_actions: u32,
    pub failed_process_attempts: u32,
    pub interruptions_triggered: u32,
    pub calm_responses: u32,
    pub panic_responses: u32,
    pub unresolved_interruptions: u32,
    pub manager_checkins: u32,
    pub coworker_helps: u32,
    pub final_energy: i32,
    pub final_stress: i32,
    pub final_focus: i32,
    pub final_reputation: i32,
}
