//! Weather visual and gameplay effects — screen tinting, turbulence shake, crosswind, icing.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

/// Marker for the weather overlay sprite used for screen-wide tinting.
#[derive(Component)]
pub struct WeatherOverlay;

/// Marker for weather particle entities (rain drops, snowflakes, fog wisps).
#[derive(Component)]
pub struct WeatherParticle {
    pub lifetime: f32,
    pub velocity: Vec2,
}

/// Tracks ice accumulation on the aircraft.
#[derive(Resource, Clone, Debug, Default)]
pub struct IcingState {
    pub accumulation: f32, // 0.0–100.0
    pub de_icing_active: bool,
}

impl IcingState {
    pub fn performance_penalty(&self) -> f32 {
        (self.accumulation / 100.0) * 0.3 // up to 30% performance loss
    }

    pub fn is_dangerous(&self) -> bool {
        self.accumulation > 70.0
    }
}

/// Lightning flash timer.
#[derive(Resource, Clone, Debug, Default)]
pub struct LightningTimer {
    pub next_flash_secs: f32,
    pub flash_active: bool,
    pub flash_timer: f32,
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Apply screen-wide visual tinting based on current weather.
pub fn apply_weather_visual_effects(
    weather_state: Res<WeatherState>,
    mut overlay_query: Query<&mut Sprite, With<WeatherOverlay>>,
) {
    let tint = match weather_state.current {
        Weather::Clear => Color::srgba(1.0, 1.0, 0.95, 0.0), // no overlay
        Weather::Cloudy => Color::srgba(0.7, 0.7, 0.75, 0.15),
        Weather::Rain => Color::srgba(0.4, 0.45, 0.55, 0.25),
        Weather::Storm => Color::srgba(0.2, 0.2, 0.3, 0.4),
        Weather::Fog => Color::srgba(0.85, 0.85, 0.85, 0.5),
        Weather::Snow => Color::srgba(0.9, 0.9, 0.95, 0.3),
        Weather::Windy => Color::srgba(0.8, 0.75, 0.65, 0.1),
    };

    for mut sprite in &mut overlay_query {
        sprite.color = tint;
    }
}

/// Screen shake during turbulence while in flight.
pub fn apply_turbulence_effects(
    weather_state: Res<WeatherState>,
    flight_state: Res<FlightState>,
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    if flight_state.phase == FlightPhase::Idle || flight_state.phase == FlightPhase::Arrived {
        return;
    }

    let intensity = match weather_state.turbulence_level {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 0.5,
        TurbulenceLevel::Moderate => 1.5,
        TurbulenceLevel::Severe => 3.5,
    };

    if intensity == 0.0 {
        return;
    }

    let t = time.elapsed_secs();
    let shake_x = (t * 15.3).sin() * intensity;
    let shake_y = (t * 19.7).cos() * intensity;

    for mut transform in &mut camera_query {
        transform.translation.x += shake_x;
        transform.translation.y += shake_y;
    }
}

/// Crosswind affects takeoff and landing difficulty.
pub fn apply_wind_effects(
    weather_state: Res<WeatherState>,
    mut flight_state: ResMut<FlightState>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Takeoff | FlightPhase::Landing | FlightPhase::Approach
    ) {
        return;
    }

    // Crosswind component based on wind speed and relative heading
    let relative_angle = (weather_state.wind_direction_deg - flight_state.heading_deg).abs();
    let crosswind_factor = (relative_angle.to_radians().sin()).abs();
    let crosswind_knots = weather_state.wind_speed_knots * crosswind_factor;

    // Heading drift from crosswind (subtle effect)
    if crosswind_knots > 5.0 {
        let drift = crosswind_knots * 0.02;
        if weather_state.wind_direction_deg > flight_state.heading_deg {
            flight_state.heading_deg += drift;
        } else {
            flight_state.heading_deg -= drift;
        }
        flight_state.heading_deg = flight_state.heading_deg.rem_euclid(360.0);
    }
}

/// Ice accumulation in cold weather at altitude — reduces performance.
pub fn apply_icing_effects(
    weather_state: Res<WeatherState>,
    flight_state: Res<FlightState>,
    mut icing: ResMut<IcingState>,
    time: Res<Time>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase == FlightPhase::Idle {
        icing.accumulation = 0.0;
        return;
    }

    let icing_conditions = matches!(weather_state.current, Weather::Snow | Weather::Storm)
        && flight_state.altitude_ft > 5000.0;

    if icing_conditions && !icing.de_icing_active {
        let rate = match weather_state.current {
            Weather::Snow => 3.0,
            Weather::Storm => 5.0,
            _ => 0.0,
        };
        icing.accumulation = (icing.accumulation + rate * time.delta_secs()).min(100.0);

        if icing.is_dangerous() {
            toast_events.send(ToastEvent {
                message: "⚠ SEVERE ICING — activate de-icing!".to_string(),
                duration_secs: 3.0,
            });
        }
    } else if icing.de_icing_active {
        icing.accumulation = (icing.accumulation - 10.0 * time.delta_secs()).max(0.0);
    } else {
        // Gradual sublimation at lower altitudes
        icing.accumulation = (icing.accumulation - 1.0 * time.delta_secs()).max(0.0);
    }
}

/// Lightning flash effects during thunderstorms.
pub fn lightning_flash_effects(
    weather_state: Res<WeatherState>,
    time: Res<Time>,
    mut lightning: ResMut<LightningTimer>,
    mut overlay_query: Query<&mut Sprite, With<WeatherOverlay>>,
) {
    if weather_state.current != Weather::Storm {
        lightning.flash_active = false;
        return;
    }

    lightning.next_flash_secs -= time.delta_secs();

    if lightning.next_flash_secs <= 0.0 && !lightning.flash_active {
        lightning.flash_active = true;
        lightning.flash_timer = 0.1; // flash duration
        let mut rng = rand::thread_rng();
        use rand::Rng;
        lightning.next_flash_secs = rng.gen_range(3.0..12.0);
    }

    if lightning.flash_active {
        lightning.flash_timer -= time.delta_secs();
        if lightning.flash_timer <= 0.0 {
            lightning.flash_active = false;
        } else {
            // Bright white flash
            for mut sprite in &mut overlay_query {
                sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.7);
            }
        }
    }
}

/// Animate weather particles (rain, snow).
pub fn animate_weather_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut WeatherParticle)>,
) {
    for (entity, mut transform, mut particle) in &mut particles {
        transform.translation.x += particle.velocity.x * time.delta_secs();
        transform.translation.y += particle.velocity.y * time.delta_secs();
        particle.lifetime -= time.delta_secs();
        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
