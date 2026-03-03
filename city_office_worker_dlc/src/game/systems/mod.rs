mod bootstrap;
mod day_cycle;
mod input;
mod interruptions;
mod task_board;
mod tasks;
mod visuals;

pub use bootstrap::{boot_to_main_menu, main_menu_to_in_day, setup_scene, toggle_pause};
pub use day_cycle::{
    apply_day_summary_rollover, check_end_of_day, finalize_end_day_request, sync_taskboard_bridge,
    transition_day_summary_to_inday, update_day_outcome_preview,
};
pub use input::collect_player_input;
pub use interruptions::{
    handle_coworker_help_requests, handle_interruption_requests, handle_manager_checkin_requests,
    handle_panic_response_requests, handle_resolve_calmly_requests,
};
pub use tasks::{handle_coffee_requests, handle_process_requests, handle_wait_requests};
pub use visuals::update_visuals;

#[cfg(test)]
mod tests;
