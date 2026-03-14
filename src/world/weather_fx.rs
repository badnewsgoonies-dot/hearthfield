//! Weather particle effects: rain, snow, and storm visuals.
//!
//! Spawns world-space particle entities (Sprite + Transform) that simulate
//! rain drops, snowflakes, and storm effects. Particles are spawned above the
//! camera viewport and despawned when they fall below it.
//!
//! Weather particles use procedurally generated sprite images (cached in a
//! Resource) rather than plain colored rectangles, giving rain a tapered
//! raindrop shape and snow a cross/star pattern.

use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
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

/// Tracks live weather particle totals so spawn logic can enforce hard caps
/// without full-query counting every frame.
#[derive(Resource, Debug, Default)]
pub struct WeatherParticleCounts {
    pub rain: usize,
    pub snow: usize,
}

// ═══════════════════════════════════════════════════════════════════════
// PROCEDURAL WEATHER SPRITES — cached resource
// ═══════════════════════════════════════════════════════════════════════

/// Cached procedural sprite images for weather particles.
/// Created once on first use, then reused for all particles.
#[derive(Resource, Default)]
pub struct WeatherSprites {
    pub loaded: bool,
    /// 2x6 raindrop sprite handle.
    pub rain_image: Handle<Image>,
    /// 3x8 storm raindrop sprite handle.
    pub storm_image: Handle<Image>,
    /// 4x4 snowflake sprite handle.
    pub snow_image: Handle<Image>,
}

/// Generate a raindrop Image (w x h pixels).
/// Tapered top (1px wide), wider bottom (w px), light blue with alpha gradient.
fn make_raindrop_image(w: u32, h: u32, r: f32, g: f32, b: f32) -> Image {
    let mut data = vec![0u8; (w * h * 4) as usize];
    for py in 0..h {
        // Progress from top (0) to bottom (1)
        let progress = py as f32 / (h - 1).max(1) as f32;
        // Width at this row: 1px at top, w px at bottom
        let row_width = (1.0 + progress * (w as f32 - 1.0)).round() as u32;
        let start = (w - row_width) / 2;
        // Alpha ramps from 0.3 at top to 0.9 at bottom
        let alpha = 0.3 + progress * 0.6;
        for px in start..(start + row_width) {
            let idx = ((py * w + px) * 4) as usize;
            data[idx] = (r * 255.0) as u8;
            data[idx + 1] = (g * 255.0) as u8;
            data[idx + 2] = (b * 255.0) as u8;
            data[idx + 3] = (alpha * 255.0) as u8;
        }
    }
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    )
}

/// Generate a snowflake Image (4x4 cross/star pattern).
fn make_snowflake_image() -> Image {
    // 4x4 cross/star pattern
    let w = 4u32;
    let h = 4u32;
    let mut data = vec![0u8; (w * h * 4) as usize];
    // Cross pattern: center column and center row
    // Plus diagonal corners for star effect
    let pattern: [(u32, u32, f32); 12] = [
        // Vertical bar (center column)
        (1, 0, 0.5),
        (2, 0, 0.5),
        (1, 1, 0.9),
        (2, 1, 0.9),
        (1, 2, 0.9),
        (2, 2, 0.9),
        (1, 3, 0.5),
        (2, 3, 0.5),
        // Horizontal bar (center row)
        (0, 1, 0.6),
        (3, 1, 0.6),
        (0, 2, 0.6),
        (3, 2, 0.6),
    ];
    for (px, py, alpha) in pattern {
        let idx = ((py * w + px) * 4) as usize;
        data[idx] = 255; // white
        data[idx + 1] = 255;
        data[idx + 2] = 255;
        data[idx + 3] = (alpha * 255.0) as u8;
    }
    Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    )
}

/// Ensure weather sprite images are generated and cached.
pub fn ensure_weather_sprites_loaded(images: &mut Assets<Image>, sprites: &mut WeatherSprites) {
    if sprites.loaded {
        return;
    }
    // Rain: 2x6, light blue
    sprites.rain_image = images.add(make_raindrop_image(2, 6, 0.5, 0.6, 1.0));
    // Storm: 3x8, brighter blue
    sprites.storm_image = images.add(make_raindrop_image(3, 8, 0.4, 0.5, 0.95));
    // Snow: 4x4 cross/star
    sprites.snow_image = images.add(make_snowflake_image());
    sprites.loaded = true;
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Maximum number of weather particles alive at once to prevent performance issues.
const MAX_WEATHER_PARTICLES: usize = 600;

/// Returns true if the given map is indoors (no weather particles).
fn is_indoor_map(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PlayerHouse
            | MapId::TownHouseWest
            | MapId::TownHouseEast
            | MapId::GeneralStore
            | MapId::AnimalShop
            | MapId::Blacksmith
            | MapId::Library
            | MapId::Tavern
    )
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn weather particles each frame based on the current weather.
///
/// Rain drops use a procedural tapered raindrop sprite.
/// Snowflakes use a procedural cross/star pattern sprite.
/// Particles are spawned at random X positions above the camera's visible area.
pub fn spawn_weather_particles(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut counts: ResMut<WeatherParticleCounts>,
    mut images: ResMut<Assets<Image>>,
    mut weather_sprites: ResMut<WeatherSprites>,
) {
    // Don't spawn weather particles on indoor maps.
    if is_indoor_map(player_state.current_map) {
        return;
    }

    let Ok(cam_tf) = camera_query.get_single() else {
        return;
    };

    // Enforce hard cap using tracked totals.
    let existing = counts.rain + counts.snow;
    if existing >= MAX_WEATHER_PARTICLES {
        return;
    }

    // Ensure procedural sprites are generated
    ensure_weather_sprites_loaded(&mut images, &mut weather_sprites);

    let mut rng = rand::thread_rng();

    let cam_scale = cam_tf.scale.x;
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
                let speed = rng.gen_range(170.0..310.0);

                let mut sprite = Sprite::from_image(weather_sprites.rain_image.clone());
                sprite.custom_size = Some(Vec2::new(2.5, 10.0));
                sprite.color = Color::srgba(0.43, 0.5, 0.72, 0.82);

                commands.spawn((
                    RainDrop { speed },
                    sprite,
                    Transform::from_translation(Vec3::new(x, y, Z_WEATHER)),
                ));
                counts.rain += 1;
            }
        }
        Weather::Stormy => {
            let count = 5.min(budget);
            for _ in 0..count {
                let x = rng.gen_range(spawn_left..spawn_right);
                let y = spawn_top + rng.gen_range(0.0..20.0);
                let speed = rng.gen_range(280.0..480.0);

                let mut sprite = Sprite::from_image(weather_sprites.storm_image.clone());
                sprite.custom_size = Some(Vec2::new(3.5, 10.0));
                sprite.color = Color::srgba(0.38, 0.45, 0.72, 0.84);

                commands.spawn((
                    RainDrop { speed },
                    sprite,
                    Transform::from_translation(Vec3::new(x, y, Z_WEATHER)),
                ));
                counts.rain += 1;
            }
        }
        Weather::Snowy => {
            let count = 2.min(budget);
            for _ in 0..count {
                let x = rng.gen_range(spawn_left..spawn_right);
                let y = spawn_top + rng.gen_range(0.0..20.0);
                let speed = rng.gen_range(14.0..28.0);
                let drift_freq = rng.gen_range(0.7..1.8);
                let drift_amp = rng.gen_range(7.0..18.0);
                let drift_phase = rng.gen_range(0.0..std::f32::consts::TAU);

                let alpha = rng.gen_range(0.72_f32..0.92);
                let brightness = rng.gen_range(0.94_f32..1.0);

                let mut sprite = Sprite::from_image(weather_sprites.snow_image.clone());
                sprite.custom_size = Some(Vec2::new(6.5, 6.5));
                sprite.color = Color::srgba(brightness, brightness * 0.985, 0.97, alpha);

                commands.spawn((
                    SnowFlake {
                        speed,
                        drift_freq,
                        drift_amp,
                        drift_phase,
                        elapsed: 0.0,
                        origin_x: x,
                    },
                    sprite,
                    Transform::from_translation(Vec3::new(x, y, Z_WEATHER)),
                ));
                counts.snow += 1;
            }
        }
        Weather::Sunny => {
            // No particles for sunny weather.
        }
    }
}

/// Move weather particles each frame and despawn those that fall below the camera.
#[allow(clippy::type_complexity)]
pub fn update_weather_particles(
    mut commands: Commands,
    time: Res<Time>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut counts: ResMut<WeatherParticleCounts>,
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
            commands.entity(entity).despawn();
            counts.rain = counts.rain.saturating_sub(1);
        }
    }

    // Update snowflakes
    for (entity, mut flake, mut transform) in snow_query.iter_mut() {
        flake.elapsed += dt;
        transform.translation.y -= flake.speed * dt;
        // Lateral sine-wave drift
        transform.translation.x = flake.origin_x
            + (flake.elapsed * flake.drift_freq + flake.drift_phase).sin() * flake.drift_amp;
        if transform.translation.y < despawn_y {
            commands.entity(entity).despawn();
            counts.snow = counts.snow.saturating_sub(1);
        }
    }
}

/// When weather changes or when on an indoor map, despawn all weather particles.
pub fn cleanup_weather_on_change(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    mut prev_weather: ResMut<PreviousWeather>,
    mut counts: ResMut<WeatherParticleCounts>,
    rain_query: Query<Entity, With<RainDrop>>,
    snow_query: Query<Entity, With<SnowFlake>>,
) {
    let should_cleanup =
        calendar.weather != prev_weather.weather || is_indoor_map(player_state.current_map);

    if should_cleanup {
        prev_weather.weather = calendar.weather;
        for entity in rain_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in snow_query.iter() {
            commands.entity(entity).despawn();
        }
        counts.rain = 0;
        counts.snow = 0;
    }
}

/// Send a toast notification when the weather changes (rain starts, stops, etc).
pub fn weather_change_notification(
    calendar: Res<Calendar>,
    mut toast_events: EventWriter<ToastEvent>,
    mut prev_weather: Local<Option<Weather>>,
) {
    let current = calendar.weather;
    if Some(current) != *prev_weather {
        if prev_weather.is_some() {
            let msg = match current {
                Weather::Rainy => "It started raining.",
                Weather::Stormy => "A storm is rolling in!",
                Weather::Snowy => "It's starting to snow.",
                Weather::Sunny => "The skies have cleared up.",
            };
            toast_events.send(ToastEvent {
                message: msg.into(),
                duration_secs: 3.0,
            });
        }
        *prev_weather = Some(current);
    }
}

/// Despawn all weather particles unconditionally (used on state exit).
pub fn cleanup_all_weather_particles(
    mut commands: Commands,
    mut counts: ResMut<WeatherParticleCounts>,
    rain_query: Query<Entity, With<RainDrop>>,
    snow_query: Query<Entity, With<SnowFlake>>,
) {
    for entity in rain_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in snow_query.iter() {
        commands.entity(entity).despawn();
    }
    counts.rain = 0;
    counts.snow = 0;
}
