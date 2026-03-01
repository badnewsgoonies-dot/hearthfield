use bevy::prelude::*;
use crate::shared::*;
use crate::world::WorldMap;

/// Smoothly follow the player with the camera using a lerp, clamped to map bounds.
/// Reads LogicalPosition (rounded) so the camera doesn't sub-pixel drift.
pub fn camera_follow_player(
    time: Res<Time>,
    player_query: Query<&LogicalPosition, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), (With<Camera2d>, Without<Player>)>,
    world_map: Res<WorldMap>,
) {
    let Ok(logical_pos) = player_query.get_single() else {
        return;
    };
    let Ok((mut cam_tf, projection)) = camera_query.get_single_mut() else {
        return;
    };

    let lerp_speed = 5.0;
    let t = (lerp_speed * time.delta_secs()).min(1.0);

    let target_x = logical_pos.0.x.round();
    let target_y = logical_pos.0.y.round();

    // Lerp in float space (smooth)
    let smooth_x = cam_tf.translation.x + (target_x - cam_tf.translation.x) * t;
    let smooth_y = cam_tf.translation.y + (target_y - cam_tf.translation.y) * t;

    // Clamp camera to map bounds so the viewport never shows past the edge
    let map_w = (world_map.width as f32) * TILE_SIZE;
    let map_h = (world_map.height as f32) * TILE_SIZE;

    let half_vw = projection.area.width() / 2.0 * cam_tf.scale.x;
    let half_vh = projection.area.height() / 2.0 * cam_tf.scale.y;

    let min_x = half_vw;
    let max_x = (map_w - half_vw).max(min_x);
    let min_y = half_vh;
    let max_y = (map_h - half_vh).max(min_y);

    cam_tf.translation.x = smooth_x.round().clamp(min_x, max_x);
    cam_tf.translation.y = smooth_y.round().clamp(min_y, max_y);
}
