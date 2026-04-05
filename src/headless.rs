//! Headless test telemetry — writes game state to a JSON file each frame
//! so external test drivers can navigate and verify screen states.
//!
//! Enabled by the `HEARTHFIELD_HEADLESS` environment variable.
//! State is written to `/tmp/hearthfield-state.json`.

use crate::player::CollisionMap;
use crate::shared::*;
use crate::world::CurrentMapId;
use bevy::prelude::*;

const STATE_FILE: &str = "/tmp/hearthfield-state.json";
const COLLISION_FILE: &str = "/tmp/hearthfield-collision.json";

/// Resource tracking whether headless telemetry is active.
#[derive(Resource)]
pub struct HeadlessTelemetry {
    pub enabled: bool,
    pub frame: u64,
    /// Only write collision data when the map changes.
    pub last_collision_map: String,
}

impl Default for HeadlessTelemetry {
    fn default() -> Self {
        Self {
            enabled: std::env::var("HEARTHFIELD_HEADLESS").is_ok(),
            frame: 0,
            last_collision_map: String::new(),
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

/// Write current game state + player position to a JSON file.
/// Also writes collision grid when the map changes.
fn write_state_telemetry(
    game_state: Res<State<GameState>>,
    player_state: Res<PlayerState>,
    current_map_id: Res<CurrentMapId>,
    calendar: Res<Calendar>,
    collision_map: Res<CollisionMap>,
    player_query: Query<&GridPosition, With<Player>>,
    mut telemetry: ResMut<HeadlessTelemetry>,
) {
    telemetry.frame += 1;

    if telemetry.frame % 5 != 0 {
        return;
    }

    let state_name = format!("{:?}", game_state.get());
    let map_name = format!("{:?}", current_map_id.map_id);
    let player_map = format!("{:?}", player_state.current_map);

    let (px, py) = player_query
        .get_single()
        .map(|gp| (gp.x, gp.y))
        .unwrap_or((-1, -1));

    let json = format!(
        r#"{{"game_state":"{}","current_map":"{}","player_map":"{}","player_x":{},"player_y":{},"season":"{:?}","day":{},"year":{},"frame":{}}}"#,
        state_name, map_name, player_map, px, py,
        calendar.season, calendar.day, calendar.year, telemetry.frame,
    );

    let _ = std::fs::write(STATE_FILE, json);

    // Write collision grid when map changes
    if collision_map.initialised && telemetry.last_collision_map != player_map {
        telemetry.last_collision_map = player_map;
        let (min_x, max_x, min_y, max_y) = collision_map.bounds;
        let mut solid_list = collision_map
            .solid_tiles
            .iter()
            .map(|&(x, y)| format!("[{},{}]", x, y))
            .collect::<Vec<_>>();
        solid_list.sort();
        let collision_json = format!(
            r#"{{"bounds":[{},{},{},{}],"solid":[{}]}}"#,
            min_x, max_x, min_y, max_y,
            solid_list.join(",")
        );
        let _ = std::fs::write(COLLISION_FILE, collision_json);
    }
}
