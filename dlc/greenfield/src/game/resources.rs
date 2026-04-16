use bevy::prelude::*;

pub const TARGET_FPS: f32 = 60.0;

pub const GAME_TITLE: &str = "Greenfield Demo";

pub const RECORDING_VERSION: u32 = 1;

#[derive(Resource, Debug, Default)]
pub struct AudioManager;

#[derive(Resource, Debug, Default)]
pub struct SettingsCache;

#[derive(Resource, Debug, Default)]
pub struct TurnClock {
    pub turn: u32,
    pub elapsed_secs: f32,
}

#[derive(Resource, Debug, Default)]
pub struct RecordingBuffer {
    pub started_at_secs: f32,
    pub event_count: u32,
}

#[derive(Resource, Debug, Default)]
pub struct GameConfig {
    pub target_fps: f32,
    pub recording_enabled: bool,
}
