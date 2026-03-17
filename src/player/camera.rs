use super::CameraSnap;
use crate::shared::*;
use crate::world::{AdjacentMapCache, WorldMap};
use bevy::prelude::*;

/// Smoothly follow the player with the camera using a lerp, clamped to map bounds.
/// On map transitions, snaps instantly for 3 frames (ensures WorldMap bounds are
/// updated before the final clamp).
#[allow(clippy::type_complexity)]
pub fn camera_follow_player(
    time: Res<Time>,
    player_query: Query<(&LogicalPosition, &PlayerMovement), (With<Player>, Without<Camera2d>)>,
    mut camera_query: Query<
        (&mut Transform, &OrthographicProjection),
        (With<Camera2d>, Without<Player>),
    >,
    world_map: Res<WorldMap>,
    adjacent_cache: Res<AdjacentMapCache>,
    mut snap: ResMut<CameraSnap>,
) {
    let Ok((logical_pos, movement)) = player_query.get_single() else {
        return;
    };
    let Ok((mut cam_tf, projection)) = camera_query.get_single_mut() else {
        return;
    };

    let target_x = logical_pos.0.x
        + match movement.facing {
            Facing::Left => -18.0,
            Facing::Right => 18.0,
            _ => 0.0,
        };
    // Offset camera upward by slightly less than half the player sprite height
    // so the player sits a bit lower on screen. LogicalPosition is at the feet
    // because player sprite uses BottomCenter anchor.
    let target_y = logical_pos.0.y
        + match movement.facing {
            Facing::Up => 34.0,
            Facing::Down => 22.0,
            Facing::Left | Facing::Right => 28.0,
        };

    // Snap if countdown active or if camera is very far from target (teleport)
    let dx = (target_x - cam_tf.translation.x).abs();
    let dy = (target_y - cam_tf.translation.y).abs();
    let should_snap = snap.frames_remaining > 0 || dx > TILE_SIZE * 3.0 || dy > TILE_SIZE * 3.0;

    let (smooth_x, smooth_y) = if should_snap {
        if snap.frames_remaining > 0 {
            snap.frames_remaining -= 1;
        }
        (target_x, target_y)
    } else {
        let lerp_speed = 6.0;
        let t = (lerp_speed * time.delta_secs()).min(1.0);
        (
            cam_tf.translation.x + (target_x - cam_tf.translation.x) * t,
            cam_tf.translation.y + (target_y - cam_tf.translation.y) * t,
        )
    };

    // Clamp camera to map bounds extended by border tile area from adjacent maps.
    // Guard: if WorldMap hasn't loaded yet (width/height 0), skip clamping.
    let map_w = (world_map.width as f32) * TILE_SIZE;
    let map_h = (world_map.height as f32) * TILE_SIZE;

    // Extend bounds for seamless border tiles (up to 12 tiles beyond each edge)
    let mut extend_neg_x: f32 = 0.0;
    let mut extend_pos_x: f32 = 0.0;
    let mut extend_neg_y: f32 = 0.0;
    let mut extend_pos_y: f32 = 0.0;
    for entry in &adjacent_cache.entries {
        let border_depth = 12.0_f32.min(match entry.direction {
            crate::world::map_data::CardinalDir::North | crate::world::map_data::CardinalDir::South => entry.map_def.height as f32,
            crate::world::map_data::CardinalDir::East | crate::world::map_data::CardinalDir::West => entry.map_def.width as f32,
        }) * TILE_SIZE;
        match entry.direction {
            crate::world::map_data::CardinalDir::North => extend_pos_y = extend_pos_y.max(border_depth),
            crate::world::map_data::CardinalDir::South => extend_neg_y = extend_neg_y.max(border_depth),
            crate::world::map_data::CardinalDir::East => extend_pos_x = extend_pos_x.max(border_depth),
            crate::world::map_data::CardinalDir::West => extend_neg_x = extend_neg_x.max(border_depth),
        }
    }
    let map_w = map_w + extend_pos_x;
    let map_h = map_h + extend_pos_y;

    if map_w <= 0.0 || map_h <= 0.0 {
        cam_tf.translation.x = smooth_x.round();
        cam_tf.translation.y = smooth_y.round();
        return;
    }

    let half_vw = projection.area.width() / 2.0 * cam_tf.scale.x;
    let half_vh = projection.area.height() / 2.0 * cam_tf.scale.y;

    // When the map is smaller than the viewport, center the camera on the
    // map instead of clamping to an edge (avoids bottom-left anchoring).
    cam_tf.translation.x = if map_w + extend_neg_x <= half_vw * 2.0 {
        (map_w - extend_neg_x) / 2.0
    } else {
        smooth_x.round().clamp(half_vw - extend_neg_x, map_w - half_vw)
    };

    cam_tf.translation.y = if map_h + extend_neg_y <= half_vh * 2.0 {
        (map_h - extend_neg_y) / 2.0
    } else {
        smooth_y.round().clamp(half_vh - extend_neg_y, map_h - half_vh)
    };
}
