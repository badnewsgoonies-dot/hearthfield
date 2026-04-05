//! Headless test telemetry — writes game state to a JSON file each frame
//! so external test drivers can navigate and verify screen states.
//!
//! Enabled by the `HEARTHFIELD_HEADLESS` environment variable.
//! State is written to `/tmp/hearthfield-state.json`.

use crate::shared::*;
use crate::world::CurrentMapId;
use bevy::prelude::*;

const STATE_FILE: &str = "/tmp/hearthfield-state.json";

/// Resource tracking whether headless telemetry is active.
#[derive(Resource)]
pub struct HeadlessTelemetry {
    pub enabled: bool,
    pub frame: u64,
}

impl Default for HeadlessTelemetry {
    fn default() -> Self {
        Self {
            enabled: std::env::var("HEARTHFIELD_HEADLESS").is_ok(),
            frame: 0,
        }
    }
}

pub struct HeadlessPlugin;

impl Plugin for HeadlessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HeadlessTelemetry>().add_systems(
            PostUpdate,
            write_state_telemetry.run_if(|t: Res<HeadlessTelemetry>| t.enabled),
        );
    }
}

/// Write current game state to a JSON file for the test driver to read.
fn write_state_telemetry(
    game_state: Res<State<GameState>>,
    player_state: Res<PlayerState>,
    current_map_id: Res<CurrentMapId>,
    calendar: Res<Calendar>,
    mut telemetry: ResMut<HeadlessTelemetry>,
) {
    telemetry.frame += 1;

    // Only write every 5 frames to reduce I/O
    if telemetry.frame % 5 != 0 {
        return;
    }

    let state_name = format!("{:?}", game_state.get());
    let map_name = format!("{:?}", current_map_id.map_id);
    let player_map = format!("{:?}", player_state.current_map);

    let json = format!(
        r#"{{"game_state":"{}","current_map":"{}","player_map":"{}","season":"{:?}","day":{},"year":{},"frame":{}}}"#,
        state_name,
        map_name,
        player_map,
        calendar.season,
        calendar.day,
        calendar.year,
        telemetry.frame,
    );

    // Best-effort write — don't crash if it fails
    let _ = std::fs::write(STATE_FILE, json);
}
