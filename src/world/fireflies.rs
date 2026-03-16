//! Ambient dusk firefly particles for outdoor maps.
//!
//! Fireflies appear only on outdoor maps during the early evening window,
//! drift gently within the camera view, pulse their alpha softly, and expire
//! after a short lifetime so the swarm feels organic rather than static.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

use super::lighting::is_indoor_map;

const MIN_FIREFLIES: usize = 8;
const MAX_FIREFLIES: usize = 12;
const FIREFLY_Z: f32 = Z_WEATHER - 5.0;

/// A single ambient firefly particle.
#[derive(Component, Debug)]
pub struct Firefly {
    /// Lifetime timer; the particle despawns when it finishes.
    pub timer: Timer,
    /// Current drift velocity in world units per second.
    pub drift_direction: Vec2,
    /// Baseline alpha before the pulse is applied.
    pub base_alpha: f32,
    /// Phase offset so nearby particles do not pulse in sync.
    pub pulse_phase: f32,
    /// Pulse frequency in radians per second.
    pub pulse_speed: f32,
}

/// Tracks the desired swarm size for the active dusk window.
#[derive(Resource, Debug, Default)]
pub struct FireflySwarmState {
    pub target_count: Option<usize>,
}

fn fireflies_should_be_active(calendar: &Calendar, map_id: MapId) -> bool {
    let time = calendar.time_float();
    (18.0..22.0).contains(&time) && !is_indoor_map(map_id)
}

/// Spawn enough fireflies to reach the active swarm target.
pub fn spawn_fireflies(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    camera_query: Query<&Transform, With<Camera2d>>,
    fireflies: Query<Entity, With<Firefly>>,
    mut swarm_state: ResMut<FireflySwarmState>,
) {
    if !fireflies_should_be_active(&calendar, player_state.current_map) {
        return;
    }

    let Ok(cam_tf) = camera_query.single() else {
        return;
    };

    let target_count = *swarm_state
        .target_count
        .get_or_insert_with(|| rand::thread_rng().gen_range(MIN_FIREFLIES..=MAX_FIREFLIES));

    let existing = fireflies.iter().count();
    if existing >= target_count {
        return;
    }

    let mut rng = rand::thread_rng();
    let cam_scale = cam_tf.scale.x;
    let half_w = (SCREEN_WIDTH * 0.5) * cam_scale;
    let half_h = (SCREEN_HEIGHT * 0.5) * cam_scale;
    let cam_pos = cam_tf.translation.truncate();

    for _ in existing..target_count {
        let x = cam_pos.x + rng.gen_range((-half_w * 0.9)..(half_w * 0.9));
        let y = cam_pos.y + rng.gen_range((-half_h * 0.75)..(half_h * 0.75));
        let size = rng.gen_range(2.0_f32..4.0);
        let drift_angle = rng.gen_range(0.0_f32..std::f32::consts::TAU);
        let drift_speed = rng.gen_range(5.0_f32..12.0);
        let drift_direction = Vec2::from_angle(drift_angle) * drift_speed;
        let base_alpha = rng.gen_range(0.35_f32..0.65);
        let pulse_phase = rng.gen_range(0.0_f32..std::f32::consts::TAU);
        let pulse_speed = rng.gen_range(1.1_f32..2.1);
        let lifetime_secs = rng.gen_range(8.0_f32..14.0);

        commands.spawn((
            Firefly {
                timer: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                drift_direction,
                base_alpha,
                pulse_phase,
                pulse_speed,
            },
            Sprite {
                color: Color::srgba(1.0, 0.92, 0.45, base_alpha),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, FIREFLY_Z),
        ));
    }
}

/// Animate fireflies with a mild drift and a slow alpha pulse.
pub fn update_fireflies(
    mut commands: Commands,
    time: Res<Time>,
    mut fireflies: Query<(Entity, &mut Firefly, &mut Sprite, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let mut rng = rand::thread_rng();

    for (entity, mut firefly, mut sprite, mut transform) in &mut fireflies {
        firefly.timer.tick(time.delta());
        if firefly.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let drift_rotation = rng.gen_range(-0.45_f32..0.45) * dt;
        firefly.drift_direction = Mat2::from_angle(drift_rotation) * firefly.drift_direction;
        transform.translation.x += firefly.drift_direction.x * dt;
        transform.translation.y += firefly.drift_direction.y * dt;

        let age = firefly.timer.elapsed_secs();
        let pulse = (age * firefly.pulse_speed + firefly.pulse_phase).sin();
        let alpha = (firefly.base_alpha + pulse * 0.22).clamp(0.08, 0.95);
        sprite.color = Color::srgba(1.0, 0.92, 0.45, alpha);
    }
}

/// Remove all fireflies when dusk ends or the player enters an indoor map.
pub fn cleanup_fireflies(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    mut swarm_state: ResMut<FireflySwarmState>,
    fireflies: Query<Entity, With<Firefly>>,
) {
    if fireflies_should_be_active(&calendar, player_state.current_map) {
        return;
    }

    for entity in &fireflies {
        commands.entity(entity).despawn();
    }
    swarm_state.target_count = None;
}

/// Despawn all fireflies unconditionally, used when leaving the playing state.
pub fn cleanup_all_fireflies(
    mut commands: Commands,
    mut swarm_state: ResMut<FireflySwarmState>,
    fireflies: Query<Entity, With<Firefly>>,
) {
    for entity in &fireflies {
        commands.entity(entity).despawn();
    }
    swarm_state.target_count = None;
}
