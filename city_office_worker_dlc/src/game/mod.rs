use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

pub struct CityOfficeWorkerPlugin;

impl Plugin for CityOfficeWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<resources::OfficeRules>()
            .init_resource::<resources::InboxState>()
            .init_resource::<resources::DayClock>()
            .init_resource::<resources::PlayerMindState>()
            .init_resource::<resources::DayStats>()
            .add_event::<events::ProcessInboxEvent>()
            .add_event::<events::CoffeeBreakEvent>()
            .add_event::<events::InterruptionEvent>()
            .add_event::<events::ResolveCalmlyEvent>()
            .add_event::<events::PanicResponseEvent>()
            .add_event::<events::WaitEvent>()
            .add_event::<events::EndOfDayEvent>()
            .add_systems(Startup, systems::setup_scene)
            .add_systems(
                Update,
                (
                    systems::collect_player_input,
                    systems::handle_interruption_requests,
                    systems::handle_resolve_calmly_requests,
                    systems::handle_panic_response_requests,
                    systems::handle_process_requests,
                    systems::handle_coffee_requests,
                    systems::handle_wait_requests,
                    systems::check_end_of_day,
                    systems::print_end_of_day_summary,
                    systems::update_visuals,
                )
                    .chain(),
            );
    }
}
