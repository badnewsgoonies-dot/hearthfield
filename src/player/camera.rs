use bevy::prelude::*;
use crate::shared::*;

/// Smoothly follow the player with the camera using a lerp.
/// Reads LogicalPosition (rounded) so the camera doesn't sub-pixel drift.
pub fn camera_follow_player(
    time: Res<Time>,
    player_query: Query<&LogicalPosition, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(logical_pos) = player_query.get_single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_query.get_single_mut() else {
        return;
    };

    let lerp_speed = 5.0;
    let t = (lerp_speed * time.delta_secs()).min(1.0);

    let target_x = logical_pos.0.x.round();
    let target_y = logical_pos.0.y.round();

    // Lerp in float space (smooth)
    let smooth_x = cam_tf.translation.x + (target_x - cam_tf.translation.x) * t;
    let smooth_y = cam_tf.translation.y + (target_y - cam_tf.translation.y) * t;

    // Snap to integer pixels for the actual transform (prevents sub-pixel blur)
    cam_tf.translation.x = smooth_x.round();
    cam_tf.translation.y = smooth_y.round();
}
