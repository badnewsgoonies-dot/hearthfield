//! Weather particle effects: rain, snow, and storm visuals.
//!
//! Spawns world-space particle entities (Sprite + Transform) that simulate
//! rain drops, snowflakes, and storm effects. Particles are spawned above the
//! camera viewport and despawned when they fall below it.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker for rain drop particle entities.
#[derive(Component, Debug)]
pub struct RainDrop {
    /// Downward speed in pixels per second.
    pub speed: f32,
}

/// Marker for snowflake particle entities.
#[derive(Component, Debug)]
pub struct SnowFlake {
    /// Downward speed in pixels per second.
    pub speed: f32,
    /// Lateral drift frequency (radians per second).
    pub drift_freq: f32,
    /// Lateral drift amplitude in pixels.
    pub drift_amp: f32,
    /// Phase offset for the sine wave so snowflakes don't all drift in sync.
    pub drift_phase: f32,
    /// Accumulated time for the sine wave calculation.
    pub elapsed: f32,
    /// The X position at spawn (center of drift).
    pub origin_x: f32,
}

/// Resource that tracks the previous weather so we can detect changes.
#[derive(Resource, Debug)]
pub struct PreviousWeather {
    pub weather: Weather,
}

impl Default for PreviousWeather {
    fn default() -> Self {
        Self {
            weather: Weather::Sunny,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum number of weather particles alive at once to prevent performance issues.
const MAX_WEATHER_PARTICLES: usize = 600;

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn weather particles each frame based on the current weather.
///
/// Rain drops are thin blue rectangles that fall fast.
/// Snowflakes are small white squares that drift laterally while falling slowly.
/// Particles are spawned at random X positions above the camera's visible area.
pub fn spawn_weather_particles(
    mut commands: Commands,
    calendar: Res<Calendar>,
    camera_query: Query<&Transform, With<Camera2d>>,
    rain_query: Query<(), With<RainDrop>>,
    snow_query: Query<(), With<SnowFlake>>,
) {
    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };

    // Count existing particles to enforce the cap.
    let existing = rain_query.iter().count() + snow_query.iter().count();
    if existing >= MAX_WEATHER_PARTICLES {
        return;
    }

    let mut rng = rand::thread_rng();

    // The camera has a scale of 1/PIXEL_SCALE (set in main.rs), meaning
    // the visible area in world units is:
    //   visible_width  = SCREEN_WIDTH  * (1 / PIXEL_SCALE) = SCREEN_WIDTH / PIXEL_SCALE
    //   visible_height = SCREEN_HEIGHT * (1 / PIXEL_SCALE) = SCREEN_HEIGHT / PIXEL_SCALE
    //
    // But the camera scale is applied via Transform::from_scale(Vec3::splat(1.0 / PIXEL_SCALE)),
    // which means the orthographic projection sees a region of size:
    //   half_width  = (SCREEN_WIDTH / 2) * (1 / PIXEL_SCALE)
    //   half_height = (SCREEN_HEIGHT / 2) * (1 / PIXEL_SCALE)
    let cam_scale = cam_tf.scale.x; // 1.0 / PIXEL_SCALE
    let half_w = (SCREEN_WIDTH / 2.0) * cam_scale;
    let half_h = (SCREEN_HEIGHT / 2.0) * cam_scale;

    let cam_x = cam_tf.translation.x;
    let cam_y = cam_tf.translation.y;

    // Spawn area: slightly wider than the visible area to avoid visible pop-in.
    let spawn_left = cam_x - half_w - 20.0;
    let spawn_right = cam_x + half_w + 20.0;
    let spawn_top = cam_y + half_h + 10.0;

    let budget = MAX_WEATHER_PARTICLES - existing;

    match calendar.weather {
        Weather::Rainy => {
            let count = 3.min(budget);
            for _ in 0..count {
                let x = rng.gen_range(spawn_left..spawn_right);
                let y = spawn_top + rng.gen_range(0.0..20.0);
                let speed = rng.gen_range(200.0..400.0);

                commands.spawn((
                    RainDrop { speed },
                    Sprite {
                        color: Color::srgba(0.5, 0.6, 1.0, 0.6),
                        custom_size: Some(Vec2::new(1.0, 6.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(x, y, Z_WEATHER)),
                ));
            }
        }
        Weather::Stormy => {
            let count = 5.min(budget);
            for _ in 0..count {
                let x = rng.gen_range(spawn_left..spawn_right);
                let y = spawn_top + rng.gen_range(0.0..20.0);
                let speed = rng.gen_range(250.0..450.0);

                commands.spawn((
                    RainDrop { speed },
                    Sprite {
                        color: Color::srgba(0.4, 0.5, 0.9, 0.7),
                        custom_size: Some(Vec2::new(1.0, 6.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(x, y, Z_WEATHER)),
                ));
            }
        }
        Weather::Snowy => {
            let count = 2.min(budget);
            for _ in 0..count {
                let x = rng.gen_range(spawn_left..spawn_right);
                let y = spawn_top + rng.gen_range(0.0..20.0);
                let speed = rng.gen_range(30.0..60.0);
                let drift_freq = rng.gen_range(1.0..3.0);
                let drift_amp = rng.gen_range(5.0..15.0);
                let drift_phase = rng.gen_range(0.0..std::f32::consts::TAU);

                commands.spawn((
                    SnowFlake {
                        speed,
                        drift_freq,
                        drift_amp,
                        drift_phase,
                        elapsed: 0.0,
                        origin_x: x,
                    },
                    Sprite {
                        color: Color::srgba(1.0, 1.0, 1.0, 0.7),
                        custom_size: Some(Vec2::new(3.0, 3.0)),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(x, y, Z_WEATHER)),
                ));
            }
        }
        Weather::Sunny => {
            // No particles for sunny weather.
        }
    }
}

/// Move weather particles each frame and despawn those that fall below the camera.
pub fn update_weather_particles(
    mut commands: Commands,
    time: Res<Time>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut rain_query: Query<(Entity, &RainDrop, &mut Transform), Without<Camera2d>>,
    mut snow_query: Query<
        (Entity, &mut SnowFlake, &mut Transform),
        (Without<Camera2d>, Without<RainDrop>),
    >,
) {
    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };

    let dt = time.delta_secs();
    let cam_scale = cam_tf.scale.x;
    let half_h = (SCREEN_HEIGHT / 2.0) * cam_scale;
    let despawn_y = cam_tf.translation.y - half_h - 20.0;

    // Update rain drops
    for (entity, drop, mut transform) in rain_query.iter_mut() {
        transform.translation.y -= drop.speed * dt;
        if transform.translation.y < despawn_y {
            if let Some(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }

    // Update snowflakes
    for (entity, mut flake, mut transform) in snow_query.iter_mut() {
        flake.elapsed += dt;
        transform.translation.y -= flake.speed * dt;
        // Lateral sine-wave drift
        transform.translation.x =
            flake.origin_x + (flake.elapsed * flake.drift_freq + flake.drift_phase).sin() * flake.drift_amp;
        if transform.translation.y < despawn_y {
            if let Some(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// When weather changes or when leaving Playing state, despawn all weather particles.
pub fn cleanup_weather_on_change(
    mut commands: Commands,
    calendar: Res<Calendar>,
    mut prev_weather: ResMut<PreviousWeather>,
    rain_query: Query<Entity, With<RainDrop>>,
    snow_query: Query<Entity, With<SnowFlake>>,
) {
    if calendar.weather != prev_weather.weather {
        prev_weather.weather = calendar.weather;
        for entity in rain_query.iter() {
            if let Some(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
        for entity in snow_query.iter() {
            if let Some(mut ec) = commands.get_entity(entity) {
                ec.despawn();
            }
        }
    }
}

/// Despawn all weather particles unconditionally (used on state exit).
pub fn cleanup_all_weather_particles(
    mut commands: Commands,
    rain_query: Query<Entity, With<RainDrop>>,
    snow_query: Query<Entity, With<SnowFlake>>,
) {
    for entity in rain_query.iter() {
        if let Some(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
    for entity in snow_query.iter() {
        if let Some(mut ec) = commands.get_entity(entity) {
            ec.despawn();
        }
    }
}
