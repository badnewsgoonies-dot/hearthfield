use bevy::prelude::*;
use crate::shared::*;
use crate::world::WorldMap;
use super::CameraSnap;

/// Smoothly follow the player with the camera using a lerp, clamped to map bounds.
/// On map transitions, snaps instantly for 2 frames (ensures WorldMap bounds are
/// updated before the final clamp).
pub fn camera_follow_player(
    time: Res<Time>,
    player_query: Query<&LogicalPosition, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), (With<Camera2d>, Without<Player>)>,
    world_map: Res<WorldMap>,
    mut snap: ResMut<CameraSnap>,
) {
    let Ok(logical_pos) = player_query.get_single() else {
        return;
    };
    let Ok((mut cam_tf, projection)) = camera_query.get_single_mut() else {
        return;
    };

    let target_x = logical_pos.0.x.round();
    let target_y = logical_pos.0.y.round();

    // Snap if countdown active or if camera is very far from target (teleport)
    let dx = (target_x - cam_tf.translation.x).abs();
    let dy = (target_y - cam_tf.translation.y).abs();
    let should_snap = snap.frames_remaining > 0 || dx > TILE_SIZE * 4.0 || dy > TILE_SIZE * 4.0;

    let (smooth_x, smooth_y) = if should_snap {
        if snap.frames_remaining > 0 {
            snap.frames_remaining -= 1;
        }
        (target_x, target_y)
    } else {
        let lerp_speed = 5.0;
        let t = (lerp_speed * time.delta_secs()).min(1.0);
        (
            cam_tf.translation.x + (target_x - cam_tf.translation.x) * t,
            cam_tf.translation.y + (target_y - cam_tf.translation.y) * t,
        )
    };

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
