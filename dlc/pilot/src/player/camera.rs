//! Camera follow system — smooth lerp, bounds clamping, zoom, shake.

use bevy::prelude::*;
use crate::shared::*;

const CAMERA_LERP_SPEED: f32 = 6.0;
const CAMERA_TRANSITION_SPEED: f32 = 4.0;
const DEFAULT_ZOOM: f32 = 1.0 / PIXEL_SCALE;
const MAP_VIEW_ZOOM: f32 = 1.0 / (PIXEL_SCALE * 2.5);

/// Tracks camera effects that persist across frames.
#[derive(Resource, Default)]
pub struct CameraState {
    pub shake_intensity: f32,
    pub shake_timer: f32,
    pub zoom_target: f32,
    pub transitioning: bool,
    pub transition_target: Vec2,
    pub transition_timer: f32,
}

impl CameraState {
    pub fn start_shake(&mut self, intensity: f32, duration: f32) {
        self.shake_intensity = intensity;
        self.shake_timer = duration;
    }

    pub fn start_transition(&mut self, target: Vec2) {
        self.transitioning = true;
        self.transition_target = target;
        self.transition_timer = 0.0;
    }
}

pub fn follow_camera(
    time: Res<Time>,
    player_q: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    mut camera_q: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    mut cam_state: ResMut<CameraState>,
    world_map: Res<WorldMap>,
    input: Res<PlayerInput>,
    game_state: Res<State<GameState>>,
) {
    let Ok(player_tf) = player_q.get_single() else { return; };
    let Ok(mut camera_tf) = camera_q.get_single_mut() else { return; };
    let dt = time.delta_secs();

    // Zoom handling
    let desired_zoom = if input.map_view && *game_state.get() == GameState::Playing {
        MAP_VIEW_ZOOM
    } else {
        DEFAULT_ZOOM
    };
    if (cam_state.zoom_target - desired_zoom).abs() > 0.001 {
        cam_state.zoom_target = desired_zoom;
    }
    let current_scale = camera_tf.scale.x;
    let new_scale = lerp(current_scale, cam_state.zoom_target, dt * 4.0);
    camera_tf.scale = Vec3::splat(new_scale);

    // Position target
    let target = if cam_state.transitioning {
        cam_state.transition_timer += dt;
        let t = (cam_state.transition_timer * CAMERA_TRANSITION_SPEED).min(1.0);
        if t >= 1.0 { cam_state.transitioning = false; }
        Vec2::new(
            lerp(camera_tf.translation.x, cam_state.transition_target.x, t),
            lerp(camera_tf.translation.y, cam_state.transition_target.y, t),
        )
    } else {
        player_tf.translation.truncate()
    };

    let lerp_factor = (CAMERA_LERP_SPEED * dt).min(1.0);
    let mut new_x = lerp(camera_tf.translation.x, target.x, lerp_factor);
    let mut new_y = lerp(camera_tf.translation.y, target.y, lerp_factor);

    // Bounds clamping
    if world_map.width > 0 && world_map.height > 0 {
        let half_view_w = 160.0 * new_scale;
        let half_view_h = 120.0 * new_scale;
        let map_w = world_map.width as f32 * TILE_SIZE;
        let map_h = world_map.height as f32 * TILE_SIZE;
        let min_x = half_view_w;
        let max_x = map_w - half_view_w;
        let min_y = -(map_h - half_view_h);
        let max_y = -half_view_h;
        if max_x > min_x { new_x = new_x.clamp(min_x, max_x); }
        if max_y > min_y { new_y = new_y.clamp(min_y, max_y); }
    }

    // Screen shake
    let mut shake_offset = Vec2::ZERO;
    if cam_state.shake_timer > 0.0 {
        cam_state.shake_timer -= dt;
        let t = cam_state.shake_timer;
        shake_offset.x = (t * 37.0).sin() * cam_state.shake_intensity;
        shake_offset.y = (t * 53.0).cos() * cam_state.shake_intensity;
        if cam_state.shake_timer <= 0.0 {
            cam_state.shake_timer = 0.0;
            cam_state.shake_intensity = 0.0;
        }
    }

    camera_tf.translation.x = new_x + shake_offset.x;
    camera_tf.translation.y = new_y + shake_offset.y;
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
