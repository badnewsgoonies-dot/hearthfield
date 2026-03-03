//! Weather visual effects — rain, snow, fog, wind streaks, lightning flashes.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS & RESOURCES
// ═══════════════════════════════════════════════════════════════════════════

/// A weather particle (rain drop, snowflake, wind streak).
#[derive(Component)]
pub struct WeatherFxParticle {
    pub kind: FxParticleKind,
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FxParticleKind {
    RainDrop,
    SnowFlake,
    WindStreak,
}

/// Fog overlay entity.
#[derive(Component)]
pub struct FogOverlay;

/// Lightning flash overlay.
#[derive(Component)]
pub struct LightningFlash {
    pub timer: f32,
    pub flash_duration: f32,
}

/// Timer controlling particle spawn rate.
#[derive(Resource)]
pub struct WeatherFxTimer {
    pub spawn_timer: f32,
    pub spawn_interval: f32,
    pub particles_per_spawn: u32,
}

impl Default for WeatherFxTimer {
    fn default() -> Self {
        Self {
            spawn_timer: 0.0,
            spawn_interval: 0.08,
            particles_per_spawn: 3,
        }
    }
}

/// Timer for periodic lightning in storms.
#[derive(Resource)]
pub struct StormLightningTimer {
    pub timer: f32,
    pub next_flash: f32,
}

impl Default for StormLightningTimer {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            timer: 0.0,
            next_flash: rng.gen_range(4.0..12.0),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Spawn the fog overlay entity at startup (hidden by default).
pub fn spawn_fog_overlay(mut commands: Commands) {
    commands.spawn((
        FogOverlay,
        Sprite::from_color(
            Color::srgba(0.8, 0.8, 0.82, 0.0),
            Vec2::new(2000.0, 2000.0),
        ),
        Transform::from_xyz(0.0, 0.0, Z_WEATHER),
        Visibility::Hidden,
    ));
}

/// Update fog visibility and opacity based on current weather.
pub fn update_fog(
    weather: Res<WeatherState>,
    location: Res<PlayerLocation>,
    mut fog_query: Query<(&mut Sprite, &mut Visibility), With<FogOverlay>>,
) {
    let is_outdoor = !location.zone.is_indoor();
    let fog_opacity = if is_outdoor && weather.current == Weather::Fog {
        0.55
    } else if is_outdoor && weather.current == Weather::Rain {
        0.1
    } else if is_outdoor && weather.current == Weather::Storm {
        0.2
    } else {
        0.0
    };

    for (mut sprite, mut vis) in &mut fog_query {
        if fog_opacity > 0.0 {
            *vis = Visibility::Inherited;
            sprite.color = Color::srgba(0.8, 0.8, 0.82, fog_opacity);
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

/// Spawn weather particles (rain, snow, wind streaks) on a timer.
pub fn spawn_weather_particles(
    mut commands: Commands,
    time: Res<Time>,
    weather: Res<WeatherState>,
    location: Res<PlayerLocation>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut fx_timer: ResMut<WeatherFxTimer>,
) {
    // Only spawn particles outdoors
    if location.zone.is_indoor() {
        return;
    }

    let should_spawn = matches!(
        weather.current,
        Weather::Rain | Weather::Snow | Weather::Storm | Weather::Windy
    );
    if !should_spawn {
        return;
    }

    fx_timer.spawn_timer += time.delta_secs();
    if fx_timer.spawn_timer < fx_timer.spawn_interval {
        return;
    }
    fx_timer.spawn_timer = 0.0;

    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };
    let cam_pos = cam_tf.translation.truncate();
    let mut rng = rand::thread_rng();

    let count = match weather.current {
        Weather::Storm => fx_timer.particles_per_spawn * 3,
        Weather::Rain => fx_timer.particles_per_spawn * 2,
        _ => fx_timer.particles_per_spawn,
    };

    for _ in 0..count {
        let offset_x = rng.gen_range(-200.0..200.0);
        let spawn_y = cam_pos.y + 150.0;
        let spawn_x = cam_pos.x + offset_x;

        let (kind, velocity, color, size, lifetime) = match weather.current {
            Weather::Rain | Weather::Storm => {
                let wind_x = weather.wind_speed_knots * 0.5;
                (
                    FxParticleKind::RainDrop,
                    Vec2::new(wind_x, -300.0),
                    Color::srgba(0.5, 0.6, 0.9, 0.6),
                    Vec2::new(2.0, 8.0),
                    1.2,
                )
            }
            Weather::Snow => (
                FxParticleKind::SnowFlake,
                Vec2::new(rng.gen_range(-20.0..20.0), -60.0),
                Color::srgba(0.95, 0.95, 1.0, 0.8),
                Vec2::new(4.0, 4.0),
                4.0,
            ),
            Weather::Windy => (
                FxParticleKind::WindStreak,
                Vec2::new(weather.wind_speed_knots * 3.0, -10.0),
                Color::srgba(0.8, 0.8, 0.8, 0.3),
                Vec2::new(16.0, 1.0),
                0.8,
            ),
            _ => continue,
        };

        commands.spawn((
            WeatherFxParticle {
                kind,
                velocity,
                lifetime: 0.0,
                max_lifetime: lifetime,
            },
            Sprite::from_color(color, size),
            Transform::from_xyz(spawn_x, spawn_y, Z_WEATHER + 0.5),
        ));
    }
}

/// Move and despawn weather particles.
pub fn update_weather_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut WeatherFxParticle, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut tf) in &mut particles {
        particle.lifetime += dt;
        tf.translation.x += particle.velocity.x * dt;
        tf.translation.y += particle.velocity.y * dt;

        if particle.lifetime >= particle.max_lifetime {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Handle lightning flashes during storms.
pub fn update_storm_lightning(
    mut commands: Commands,
    time: Res<Time>,
    weather: Res<WeatherState>,
    location: Res<PlayerLocation>,
    mut storm_timer: ResMut<StormLightningTimer>,
    mut flash_query: Query<(Entity, &mut LightningFlash, &mut Sprite)>,
) {
    // Tick existing flashes
    for (entity, mut flash, mut sprite) in &mut flash_query {
        flash.timer += time.delta_secs();
        let t = flash.timer / flash.flash_duration;
        if t >= 1.0 {
            commands.entity(entity).despawn_recursive();
        } else {
            let alpha = (1.0 - t) * 0.7;
            sprite.color = Color::srgba(1.0, 1.0, 0.95, alpha);
        }
    }

    // Only generate new lightning during storms, outdoors
    if weather.current != Weather::Storm || location.zone.is_indoor() {
        return;
    }

    storm_timer.timer += time.delta_secs();
    if storm_timer.timer >= storm_timer.next_flash {
        storm_timer.timer = 0.0;
        let mut rng = rand::thread_rng();
        storm_timer.next_flash = rng.gen_range(3.0..10.0);

        // Spawn a full-screen flash
        commands.spawn((
            LightningFlash {
                timer: 0.0,
                flash_duration: 0.15,
            },
            Sprite::from_color(
                Color::srgba(1.0, 1.0, 0.95, 0.7),
                Vec2::new(2000.0, 2000.0),
            ),
            Transform::from_xyz(0.0, 0.0, Z_WEATHER + 2.0),
        ));
    }
}

/// Despawn all weather particles when moving indoors or weather clears.
pub fn cleanup_weather_particles(
    mut commands: Commands,
    weather: Res<WeatherState>,
    location: Res<PlayerLocation>,
    particles: Query<Entity, With<WeatherFxParticle>>,
) {
    let should_clear = location.zone.is_indoor()
        || matches!(weather.current, Weather::Clear | Weather::Cloudy);

    if should_clear {
        for entity in &particles {
            commands.entity(entity).despawn_recursive();
        }
    }
}
