mod bootstrap;
mod day_cycle;
mod input;
mod interruptions;
mod task_board;
mod tasks;
mod visuals;

pub use bootstrap::{boot_to_main_menu, setup_scene, toggle_pause};
pub use day_cycle::{
    apply_day_summary_rollover, check_end_of_day, check_relationship_milestones,
    consume_end_of_day_events, finalize_end_day_request, sync_taskboard_bridge,
    transition_day_summary_to_inday, update_day_outcome_preview,
};
pub use input::collect_player_input;
pub use interruptions::{
    auto_trigger_interruptions, handle_coworker_help_requests, handle_interruption_requests,
    handle_manager_checkin_requests, handle_panic_response_requests, handle_resolve_calmly_requests,
};
pub use tasks::{
    enforce_task_deadlines, handle_coffee_requests, handle_process_requests, handle_wait_requests,
};
pub use visuals::update_visuals;

#[cfg(test)]
mod tests;
