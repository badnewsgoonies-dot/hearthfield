use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Write current game state + player position to a JSON file.
/// Also writes collision grid when the map changes.
pub fn trial_a4_system(
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

    // Write collision grid when the map changes AND collision has been rebuilt.
    // Include map name so the driver can verify it's reading the right map.
    let map_id_str = format!("{:?}", current_map_id.map_id);
    let collision_fresh = collision_map.initialised
        && (telemetry.last_collision_map != map_id_str);
    if collision_fresh {
        telemetry.last_collision_map = map_id_str.clone();
        let (min_x, max_x, min_y, max_y) = collision_map.bounds;
        let mut solid_list = collision_map
            .solid_tiles
            .iter()
            .map(|&(x, y)| format!("[{},{}]", x, y))
            .collect::<Vec<_>>();
        solid_list.sort();
        let collision_json = format!(
            r#"{{"map":"{}","bounds":[{},{},{},{}],"solid":[{}]}}"#,
            map_id_str, min_x, max_x, min_y, max_y,
            solid_list.join(",")
        );
        let _ = std::fs::write(COLLISION_FILE, collision_json);
    }
}

