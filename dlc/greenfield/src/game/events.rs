use bevy::prelude::*;

#[derive(Event, Debug, Default)]
pub struct BootStartedEvent;

#[derive(Event, Debug, Default)]
pub struct BootCompletedEvent;

#[derive(Event, Debug, Default)]
pub struct MainMenuEnteredEvent;

#[derive(Event, Debug, Default)]
pub struct MainMenuExitedEvent;

#[derive(Event, Debug, Default)]
pub struct GameStartedEvent;

#[derive(Event, Debug, Default)]
pub struct GamePausedEvent;

#[derive(Event, Debug, Default)]
pub struct GameResumedEvent;

#[derive(Event, Debug, Default)]
pub struct GameEndedEvent;

#[derive(Event, Debug, Default)]
pub struct TurnBeganEvent;

#[derive(Event, Debug, Default)]
pub struct TurnEndedEvent;

#[derive(Event, Debug, Default)]
pub struct TurnRecordedEvent;

#[derive(Event, Debug, Default)]
pub struct InputTickedEvent;

#[derive(Event, Debug, Default)]
pub struct SimulationTickedEvent;

#[derive(Event, Debug, Default)]
pub struct RenderTickedEvent;

#[derive(Event, Debug, Default)]
pub struct GameLoadedEvent;

#[derive(Event, Debug, Default)]
pub struct GameUnloadedEvent;

#[derive(Event, Debug, Default)]
pub struct ConfigurationChangedEvent;

#[derive(Event, Debug, Clone)]
pub struct ButtonPressedEvent {
    pub button_id: u32,
}

#[derive(Event, Debug, Clone)]
pub struct TimerElapsedEvent {
    pub timer_id: u32,
    pub elapsed_secs: f32,
}

#[derive(Event, Debug, Clone)]
pub struct ScoreChangedEvent {
    pub old_score: i32,
    pub new_score: i32,
}

#[derive(Event, Debug, Clone)]
pub struct PlayerMovedEvent {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
