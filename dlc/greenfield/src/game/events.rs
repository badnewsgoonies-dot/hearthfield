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
