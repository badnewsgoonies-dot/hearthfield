use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod resources;
pub mod save;
pub mod systems;
pub mod ui;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OfficeGameState {
    #[default]
    Boot,
    MainMenu,
    InDay,
    DaySummary,
    Paused,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OfficeSimSet {
    Input,
    Time,
    TaskGeneration,
    TaskResolution,
    Interruptions,
    Economy,
    StateTransitions,
    Ui,
}

pub struct CityOfficeWorkerPlugin;

impl Plugin for CityOfficeWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<OfficeGameState>()
            .init_resource::<resources::OfficeRules>()
            .init_resource::<resources::OfficeRunConfig>()
            .init_resource::<resources::WorkerStats>()
            .init_resource::<resources::InboxState>()
            .init_resource::<resources::DayClock>()
            .init_resource::<resources::PlayerMindState>()
            .init_resource::<resources::PlayerCareerState>()
            .init_resource::<resources::SocialGraphState>()
            .init_resource::<resources::CoworkerDialogue>()
            .init_resource::<resources::OfficeEconomyRules>()
            .init_resource::<resources::CareerProgression>()
            .init_resource::<resources::UnlockCatalogState>()
            .init_resource::<resources::TaskBoard>()
            .init_resource::<resources::DayOutcome>()
            .init_resource::<resources::DayStats>()
            .init_resource::<resources::ActiveInterruptionContext>()
            .init_resource::<resources::FiredMilestones>()
            .init_resource::<resources::ToastState>()
            .init_resource::<save::SaveSlotConfig>()
            .init_resource::<save::OfficeSaveStore>()
            .init_resource::<resources::WorkerSpriteData>()
            .init_resource::<resources::OfficeFontHandle>()
            .init_resource::<ui::day_summary::DaySummarySnapshot>()
            .add_event::<events::EndDayRequested>()
            .add_event::<events::DayAdvanced>()
            .add_event::<events::TaskAccepted>()
            .add_event::<events::TaskProgressed>()
            .add_event::<events::TaskCompleted>()
            .add_event::<events::TaskFailed>()
            .add_event::<events::ProcessInboxEvent>()
            .add_event::<events::CoffeeBreakEvent>()
            .add_event::<events::InterruptionEvent>()
            .add_event::<events::ResolveCalmlyEvent>()
            .add_event::<events::PanicResponseEvent>()
            .add_event::<events::ManagerCheckInEvent>()
            .add_event::<events::CoworkerHelpEvent>()
            .add_event::<events::WaitEvent>()
            .add_event::<events::EndOfDayEvent>()
            .add_event::<events::RelationshipMilestone>()
            .add_event::<save::SaveSlotRequest>()
            .add_event::<save::LoadSlotRequest>()
            .add_systems(Startup, (resources::load_office_font, systems::setup_scene))
            .add_systems(
                Update,
                (
                    save::handle_save_slot_requests,
                    save::handle_load_slot_requests,
                )
                    .chain(),
            )
            // Boot -> MainMenu (one-shot transition)
            .add_systems(
                Update,
                systems::boot_to_main_menu.run_if(in_state(OfficeGameState::Boot)),
            )
            // Main Menu UI
            .add_systems(
                OnEnter(OfficeGameState::MainMenu),
                ui::main_menu::spawn_main_menu,
            )
            .add_systems(
                OnExit(OfficeGameState::MainMenu),
                ui::main_menu::despawn_main_menu,
            )
            .add_systems(
                Update,
                ui::main_menu::handle_main_menu_input.run_if(in_state(OfficeGameState::MainMenu)),
            )
            // Pause toggle (keyboard Esc)
            .add_systems(
                Update,
                systems::toggle_pause
                    .run_if(in_state(OfficeGameState::InDay).or(in_state(OfficeGameState::Paused))),
            )
            // Pause Menu UI
            .add_systems(
                OnEnter(OfficeGameState::Paused),
                ui::pause_menu::spawn_pause_menu,
            )
            .add_systems(
                OnExit(OfficeGameState::Paused),
                ui::pause_menu::despawn_pause_menu,
            )
            .add_systems(
                Update,
                ui::pause_menu::handle_pause_input.run_if(in_state(OfficeGameState::Paused)),
            )
            // HUD + Task Board + Interruption (InDay state)
            .add_systems(
                OnEnter(OfficeGameState::InDay),
                (
                    ui::hud::spawn_hud,
                    ui::task_board::spawn_task_board,
                    ui::interruption::spawn_interruption_popup,
                    ui::hud::spawn_toast,
                ),
            )
            .add_systems(
                OnExit(OfficeGameState::InDay),
                (
                    ui::hud::despawn_hud,
                    ui::task_board::despawn_task_board,
                    ui::interruption::despawn_interruption_popup,
                    ui::hud::despawn_toast,
                ),
            )
            // Day Summary UI
            .add_systems(
                OnEnter(OfficeGameState::DaySummary),
                ui::day_summary::spawn_day_summary,
            )
            .add_systems(
                OnExit(OfficeGameState::DaySummary),
                ui::day_summary::despawn_day_summary,
            )
            .add_systems(
                Update,
                ui::day_summary::handle_day_summary_input
                    .run_if(in_state(OfficeGameState::DaySummary)),
            )
            // System sets for InDay simulation
            .configure_sets(
                Update,
                (
                    OfficeSimSet::Input,
                    OfficeSimSet::Time,
                    OfficeSimSet::TaskGeneration,
                    OfficeSimSet::TaskResolution,
                    OfficeSimSet::Interruptions,
                    OfficeSimSet::Economy,
                    OfficeSimSet::StateTransitions,
                    OfficeSimSet::Ui,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    systems::collect_player_input.in_set(OfficeSimSet::Input),
                    systems::handle_wait_requests.in_set(OfficeSimSet::Time),
                    systems::sync_taskboard_bridge.in_set(OfficeSimSet::TaskGeneration),
                    (
                        systems::handle_process_requests,
                        systems::handle_coffee_requests,
                    )
                        .chain()
                        .in_set(OfficeSimSet::TaskResolution),
                    (
                        systems::auto_trigger_interruptions,
                        systems::handle_interruption_requests,
                        systems::handle_resolve_calmly_requests,
                        systems::handle_panic_response_requests,
                        systems::handle_manager_checkin_requests,
                        systems::handle_coworker_help_requests,
                    )
                        .chain()
                        .in_set(OfficeSimSet::Interruptions),
                    (
                        systems::update_day_outcome_preview,
                        systems::check_relationship_milestones,
                    )
                        .chain()
                        .in_set(OfficeSimSet::Economy),
                    (
                        systems::enforce_task_deadlines,
                        systems::check_end_of_day,
                        systems::finalize_end_day_request,
                    )
                        .chain()
                        .in_set(OfficeSimSet::StateTransitions),
                    (
                        systems::update_visuals,
                        ui::hud::update_hud,
                        ui::hud::update_career_hud,
                        ui::hud::update_unlocks_hud,
                        ui::hud::update_coworker_panel,
                        ui::hud::update_toast,
                        ui::task_board::update_task_board,
                        ui::interruption::update_interruption_visibility,
                    )
                        .in_set(OfficeSimSet::Ui),
                )
                    .run_if(in_state(OfficeGameState::InDay)),
            )
            .add_systems(
                Update,
                (
                    systems::consume_end_of_day_events,
                    systems::apply_day_summary_rollover,
                    save::persist_day_summary_snapshot,
                    systems::transition_day_summary_to_inday,
                )
                    .chain()
                    .run_if(in_state(OfficeGameState::DaySummary)),
            );
    }
}
