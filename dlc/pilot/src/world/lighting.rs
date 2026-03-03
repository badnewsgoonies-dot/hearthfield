//! Lighting system — time-of-day ambient lighting, indoor/outdoor detection,
//! weather modifiers, runway lights at night.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS & RESOURCES
// ═══════════════════════════════════════════════════════════════════════════

/// Global ambient light state computed each frame.
#[derive(Resource, Clone, Debug)]
pub struct AmbientLighting {
    pub color: Color,
    pub intensity: f32,
}

impl Default for AmbientLighting {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            intensity: 1.0,
        }
    }
}

/// Marker for the ambient light overlay sprite.
#[derive(Component)]
pub struct LightOverlay;

/// Marker for runway light entities that blink at night.
#[derive(Component)]
pub struct RunwayLight {
    pub blink_timer: f32,
    pub blink_speed: f32,
    pub on: bool,
}

impl Default for RunwayLight {
    fn default() -> Self {
        Self {
            blink_timer: 0.0,
            blink_speed: 1.2,
            on: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TIME-OF-DAY LIGHTING TABLES
// ═══════════════════════════════════════════════════════════════════════════

/// Compute the base ambient color and intensity for a given hour.
fn time_of_day_lighting(hour: u32) -> (Color, f32) {
    match hour {
        // Night
        0..=4 => (Color::srgb(0.15, 0.15, 0.35), 0.25),
        // Dawn
        5 => (Color::srgb(0.6, 0.45, 0.3), 0.5),
        6 => (Color::srgb(0.85, 0.7, 0.45), 0.7),
        // Morning
        7..=9 => (Color::srgb(1.0, 0.95, 0.85), 0.9),
        // Midday
        10..=14 => (Color::srgb(1.0, 1.0, 1.0), 1.0),
        // Afternoon
        15..=16 => (Color::srgb(1.0, 0.97, 0.9), 0.95),
        // Evening
        17 => (Color::srgb(0.95, 0.8, 0.55), 0.8),
        18 => (Color::srgb(0.85, 0.55, 0.35), 0.65),
        // Dusk
        19 => (Color::srgb(0.5, 0.35, 0.45), 0.45),
        // Night
        20..=23 => (Color::srgb(0.15, 0.15, 0.35), 0.25),
        _ => (Color::WHITE, 1.0),
    }
}

/// Weather-based lighting modifier.
fn weather_light_modifier(weather: &Weather) -> (Color, f32) {
    match weather {
        Weather::Clear => (Color::WHITE, 1.0),
        Weather::Cloudy => (Color::srgb(0.85, 0.85, 0.9), 0.85),
        Weather::Rain => (Color::srgb(0.7, 0.7, 0.75), 0.7),
        Weather::Storm => (Color::srgb(0.4, 0.4, 0.5), 0.45),
        Weather::Fog => (Color::srgb(0.8, 0.8, 0.82), 0.65),
        Weather::Snow => (Color::srgb(0.9, 0.9, 0.95), 0.75),
        Weather::Windy => (Color::srgb(0.9, 0.9, 0.85), 0.9),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Spawn the ambient light overlay at startup.
pub fn spawn_light_overlay(mut commands: Commands) {
    commands.spawn((
        LightOverlay,
        Sprite::from_color(
            Color::srgba(0.0, 0.0, 0.0, 0.0),
            Vec2::new(2000.0, 2000.0),
        ),
        Transform::from_xyz(0.0, 0.0, Z_WEATHER - 1.0),
    ));
}

/// Update ambient lighting based on time of day, indoor/outdoor, and weather.
pub fn update_ambient_lighting(
    calendar: Res<Calendar>,
    weather: Res<WeatherState>,
    location: Res<PlayerLocation>,
    mut ambient: ResMut<AmbientLighting>,
    mut overlay_query: Query<&mut Sprite, With<LightOverlay>>,
) {
    // Indoor zones get constant bright lighting
    if location.zone.is_indoor() {
        ambient.color = Color::srgb(0.95, 0.92, 0.85);
        ambient.intensity = 0.9;

        // Apply a very subtle overlay
        for mut sprite in &mut overlay_query {
            sprite.color = Color::srgba(0.0, 0.0, 0.0, 0.05);
        }
        return;
    }

    // Outdoor: combine time-of-day + weather
    let (tod_color, tod_intensity) = time_of_day_lighting(calendar.hour);
    let (wx_color, wx_intensity) = weather_light_modifier(&weather.current);

    let r = tod_color.to_srgba().red * wx_color.to_srgba().red;
    let g = tod_color.to_srgba().green * wx_color.to_srgba().green;
    let b = tod_color.to_srgba().blue * wx_color.to_srgba().blue;

    ambient.color = Color::srgb(r, g, b);
    ambient.intensity = tod_intensity * wx_intensity;

    // Set overlay darkness (inverse of intensity)
    let darkness = 1.0 - ambient.intensity;
    for mut sprite in &mut overlay_query {
        let srgba = ambient.color.to_srgba();
        sprite.color = Color::srgba(
            1.0 - srgba.red,
            1.0 - srgba.green,
            1.0 - srgba.blue,
            darkness * 0.6,
        );
    }
}

/// Animate runway lights: blink at night, solid during day.
pub fn animate_runway_lights(
    time: Res<Time>,
    calendar: Res<Calendar>,
    location: Res<PlayerLocation>,
    mut lights: Query<(&mut RunwayLight, &mut Visibility)>,
) {
    let is_runway = location.zone == MapZone::Runway;
    let night = calendar.is_night();

    for (mut light, mut vis) in &mut lights {
        if !is_runway {
            *vis = Visibility::Hidden;
            continue;
        }

        if night {
            light.blink_timer += time.delta_secs();
            if light.blink_timer >= light.blink_speed {
                light.blink_timer = 0.0;
                light.on = !light.on;
            }
            *vis = if light.on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            *vis = Visibility::Inherited;
            light.on = true;
        }
    }
}

/// Spawn runway edge lights along the runway.
pub fn spawn_runway_lights(
    mut commands: Commands,
    mut transition_events: EventReader<ZoneTransitionEvent>,
    existing: Query<Entity, With<RunwayLight>>,
) {
    for evt in transition_events.read() {
        // Despawn old lights
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }

        if evt.to_zone != MapZone::Runway {
            continue;
        }

        // Place lights along both sides of the runway
        for i in 0..20 {
            let x_left = 2;
            let x_right = 18;
            let y = i + 1;
            let speed = 1.0 + (i as f32 * 0.05);

            for gx in [x_left, x_right] {
                let pos = grid_to_world_center(gx, y);
                commands.spawn((
                    RunwayLight {
                        blink_speed: speed,
                        ..default()
                    },
                    Sprite::from_color(Color::srgb(1.0, 0.9, 0.3), Vec2::new(4.0, 4.0)),
                    Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR + 0.5),
                ));
            }
        }
    }
}
