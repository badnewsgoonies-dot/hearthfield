use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod resources;
pub mod systems;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OfficeGameState {
    #[default]
    InDay,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfficeSimSet {
    Input,
    ActionResolution,
    DayBoundary,
    Presentation,
}

pub struct CityOfficeWorkerPlugin;

impl Plugin for CityOfficeWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OfficeGameState>()
            .init_resource::<resources::OfficeRules>()
            .init_resource::<resources::InboxState>()
            .init_resource::<resources::DayClock>()
            .init_resource::<resources::PlayerMindState>()
            .init_resource::<resources::PlayerCareerState>()
            .init_resource::<resources::DayStats>()
            .add_event::<events::EndDayRequested>()
            .add_event::<events::DayAdvanced>()
            .add_event::<events::ProcessInboxEvent>()
            .add_event::<events::CoffeeBreakEvent>()
            .add_event::<events::InterruptionEvent>()
            .add_event::<events::ResolveCalmlyEvent>()
            .add_event::<events::PanicResponseEvent>()
            .add_event::<events::ManagerCheckInEvent>()
            .add_event::<events::CoworkerHelpEvent>()
            .add_event::<events::WaitEvent>()
            .add_event::<events::EndOfDayEvent>()
            .add_systems(Startup, systems::setup_scene)
            .configure_sets(
                Update,
                (
                    OfficeSimSet::Input,
                    OfficeSimSet::ActionResolution,
                    OfficeSimSet::DayBoundary,
                    OfficeSimSet::Presentation,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    systems::collect_player_input.in_set(OfficeSimSet::Input),
                    (
                        systems::handle_interruption_requests,
                        systems::handle_resolve_calmly_requests,
                        systems::handle_panic_response_requests,
                        systems::handle_manager_checkin_requests,
                        systems::handle_coworker_help_requests,
                        systems::handle_process_requests,
                        systems::handle_coffee_requests,
                        systems::handle_wait_requests,
                    )
                        .chain()
                        .in_set(OfficeSimSet::ActionResolution),
                    (systems::check_end_of_day, systems::print_end_of_day_summary)
                        .chain()
                        .in_set(OfficeSimSet::DayBoundary),
                    systems::update_visuals.in_set(OfficeSimSet::Presentation),
                )
                    .run_if(in_state(OfficeGameState::InDay)),
            );
    }
}
