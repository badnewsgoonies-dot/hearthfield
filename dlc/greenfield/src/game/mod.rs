use bevy::prelude::*;

pub mod components;
pub mod events;
pub mod systems;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GreenfieldState {
    #[default]
    Boot,
    MainMenu,
    Playing,
    Paused,
    GameOver,
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
