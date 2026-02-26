use bevy::prelude::*;
use crate::shared::*;

/// Smoothly follow the player with the camera using a lerp.
/// The camera's Transform is interpolated towards the player's position
/// each frame so the motion feels fluid rather than locked-on.
pub fn camera_follow_player(
    time: Res<Time>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let Ok(mut cam_tf) = camera_query.get_single_mut() else {
        return;
    };

    // Lerp speed — higher = snappier. 5.0 gives a nice smooth follow
    // that keeps the player roughly centered without feeling jarring.
    let lerp_speed = 5.0;
    let t = (lerp_speed * time.delta_secs()).min(1.0);

    cam_tf.translation.x = cam_tf.translation.x + (player_tf.translation.x - cam_tf.translation.x) * t;
    cam_tf.translation.y = cam_tf.translation.y + (player_tf.translation.y - cam_tf.translation.y) * t;
    // Keep camera Z unchanged (main.rs sets it via scale).
}
