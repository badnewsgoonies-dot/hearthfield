use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct OfficeRules {
    pub max_energy: i32,
    pub max_stress: i32,
    pub max_focus: i32,
    pub process_energy_cost: i32,
    pub process_minutes: u32,
    pub coffee_restore: i32,
    pub coffee_minutes: u32,
    pub wait_minutes: u32,
    pub interruption_minutes: u32,
    pub interruption_stress_increase: i32,
    pub interruption_focus_loss: i32,
    pub calm_focus_restore: i32,
    pub calm_stress_relief: i32,
    pub panic_stress_increase: i32,
    pub panic_focus_loss: i32,
    pub starting_stress: i32,
    pub starting_focus: i32,
    pub day_start_minute: u32,
    pub day_end_minute: u32,
    pub starting_inbox_items: u32,
}

impl Default for OfficeRules {
    fn default() -> Self {
        Self {
            max_energy: 100,
            max_stress: 100,
            max_focus: 100,
            process_energy_cost: 12,
            process_minutes: 15,
            coffee_restore: 25,
            coffee_minutes: 20,
            wait_minutes: 10,
            interruption_minutes: 12,
            interruption_stress_increase: 14,
            interruption_focus_loss: 10,
            calm_focus_restore: 12,
            calm_stress_relief: 9,
            panic_stress_increase: 11,
            panic_focus_loss: 7,
            starting_stress: 18,
            starting_focus: 76,
            day_start_minute: 9 * 60,
            day_end_minute: 17 * 60,
            starting_inbox_items: 18,
        }
    }
}

#[derive(Resource, Debug)]
pub struct InboxState {
    pub remaining_items: u32,
}

impl Default for InboxState {
    fn default() -> Self {
        Self {
            remaining_items: 18,
        }
    }
}

#[derive(Resource, Debug)]
pub struct DayClock {
    pub day_number: u32,
    pub current_minute: u32,
    pub ended: bool,
}

impl Default for DayClock {
    fn default() -> Self {
        Self {
            day_number: 1,
            current_minute: 9 * 60,
            ended: false,
        }
    }
}

impl DayClock {
    pub fn advance(&mut self, minutes: u32) {
        self.current_minute = self.current_minute.saturating_add(minutes);
    }
}

#[derive(Resource, Debug)]
pub struct PlayerMindState {
    pub stress: i32,
    pub focus: i32,
    pub pending_interruptions: u32,
}

impl Default for PlayerMindState {
    fn default() -> Self {
        Self {
            stress: 18,
            focus: 76,
            pending_interruptions: 0,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct DayStats {
    pub processed_items: u32,
    pub coffee_breaks: u32,
    pub wait_actions: u32,
    pub failed_process_attempts: u32,
    pub interruptions_triggered: u32,
    pub calm_responses: u32,
    pub panic_responses: u32,
}

pub fn format_clock(total_minutes: u32) -> String {
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    format!("{hours:02}:{minutes:02}")
}
