use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod systems;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GreenfieldState {
    #[default]
    Boot,
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Loading,
    Settings,
    Credits,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GreenfieldSet {
    Input,
    Simulation,
    Render,
}

pub struct GreenfieldPlugin;

impl Plugin for GreenfieldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GreenfieldState>()
            .add_systems(Update, systems::update_hud_sys::update_hud_system)
            .add_systems(Update, systems::play_audio_sys::play_audio_system)
            .add_systems(Update, systems::handle_config_sys::handle_config_system)
            .add_systems(Update, systems::record_tick_sys::record_tick_system)
            .add_systems(Update, systems::tick_clock_sys::tick_clock_system)
            .add_event::<events::PlayerMovedEvent>()
            .add_event::<events::ScoreChangedEvent>()
            .add_event::<events::TimerElapsedEvent>()
            .add_event::<events::ButtonPressedEvent>()
            .add_event::<events::ConfigurationChangedEvent>()
            .add_event::<events::GameUnloadedEvent>()
            .add_event::<events::GameLoadedEvent>()
            .init_resource::<resources::GameConfig>()
            .init_resource::<resources::RecordingBuffer>()
            .init_resource::<resources::TurnClock>()
            .init_resource::<resources::SettingsCache>()
            .init_resource::<resources::AudioManager>()
            .add_systems(Update, systems::render_tick::render_tick)
            .add_systems(Update, systems::sim_tick::sim_tick)
            .add_systems(Update, systems::input_tick::input_tick)
            .add_systems(Update, systems::boot_tick::boot_tick)
            .add_event::<events::RenderTickedEvent>()
            .add_event::<events::SimulationTickedEvent>()
            .add_event::<events::InputTickedEvent>()
            .add_event::<events::TurnRecordedEvent>()
            .add_event::<events::TurnEndedEvent>()
            .add_event::<events::TurnBeganEvent>()
            .add_event::<events::GameEndedEvent>()
            .add_event::<events::GameResumedEvent>()
            .add_event::<events::GamePausedEvent>()
            .add_event::<events::GameStartedEvent>()
            .add_event::<events::MainMenuExitedEvent>()
            .add_event::<events::MainMenuEnteredEvent>()
            .add_event::<events::BootCompletedEvent>()
            .add_event::<events::BootStartedEvent>()
            ;
    }
}
pub mod scene;
pub mod audio;
pub mod input;
