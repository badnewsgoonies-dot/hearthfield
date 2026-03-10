//! Visual effects for watering can and scythe tool impacts.
//!
//! Implements:
//!  - WaterSplash: expanding light-blue poof on the watered tile
//!  - WaterDrip: 2-3 small water droplets that linger briefly after splash
//!  - ScytheArc: a short-lived horizontal slash arc on scythe impact
//!  - CutDebris: 2-3 thin green/yellow fragments that float up after scythe hit

use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// WaterSplash — expanding semi-transparent poof on watered tile
// ═══════════════════════════════════════════════════════════════════════════

/// Brief expanding water-splash overlay spawned when the watering can impacts.
#[derive(Component)]
pub struct WaterSplash {
    pub timer: Timer,
}

/// Listen for WateringCan impact events and spawn a water splash poof.
pub fn spawn_water_splash(
    mut commands: Commands,
    mut impact_events: EventReader<ToolImpactEvent>,
) {
    for event in impact_events.read() {
        if event.tool != ToolKind::WateringCan {
            continue;
        }
        let wx = event.grid_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let wy = event.grid_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        // Main splash poof: light blue semi-transparent, expands from 0.3 to 1.2 scale
        commands.spawn((
            WaterSplash {
                timer: Timer::from_seconds(0.25, TimerMode::Once),
            },
            Sprite {
                color: Color::srgba(0.4, 0.6, 1.0, 0.25),
                custom_size: Some(Vec2::new(8.0, 8.0)),
                ..default()
            },
            Transform::from_xyz(wx, wy, Z_FARM_OVERLAY + 2.0).with_scale(Vec3::splat(0.3)),
            Visibility::default(),
        ));
    }
}

/// Each frame: scale from 0.3 → 1.2, fade alpha 0.25 → 0, despawn when done.
pub fn update_water_splash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut WaterSplash)>,
) {
    for (entity, mut tf, mut sprite, mut splash) in query.iter_mut() {
        splash.timer.tick(time.delta());

        if splash.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let t = splash.timer.fraction(); // 0.0 → 1.0
        // Scale from 0.3 to 1.2
        let scale = 0.3 + t * 0.9;
        tf.scale = Vec3::splat(scale);

        // Alpha fades from 0.25 to 0
        let alpha = 0.25 * (1.0 - t);
        sprite.color = Color::srgba(0.4, 0.6, 1.0, alpha);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WaterDrip — tiny water droplets that linger briefly on the soil
// ═══════════════════════════════════════════════════════════════════════════

/// A tiny water droplet that lingers on a watered tile after the splash.
#[derive(Component)]
pub struct WaterDrip {
    pub timer: Timer,
    pub initial_alpha: f32,
}

/// Spawn 2-3 tiny water droplets randomly within the watered tile.
pub fn spawn_water_drips(
    mut commands: Commands,
    mut impact_events: EventReader<ToolImpactEvent>,
) {
    let mut rng = rand::thread_rng();

    for event in impact_events.read() {
        if event.tool != ToolKind::WateringCan {
            continue;
        }

        let tile_origin_x = event.grid_x as f32 * TILE_SIZE;
        let tile_origin_y = event.grid_y as f32 * TILE_SIZE;

        let drip_count = rng.gen_range(2..=3usize);
        for _ in 0..drip_count {
            // Random position within the tile
            let offset_x = rng.gen_range(2.0f32..TILE_SIZE - 2.0);
            let offset_y = rng.gen_range(2.0f32..TILE_SIZE - 2.0);
            let wx = tile_origin_x + offset_x;
            let wy = tile_origin_y + offset_y;

            let lifetime = rng.gen_range(0.35f32..0.5);

            commands.spawn((
                WaterDrip {
                    timer: Timer::from_seconds(lifetime, TimerMode::Once),
                    initial_alpha: 0.3,
                },
                Sprite {
                    color: Color::srgba(0.3, 0.5, 0.9, 0.3),
                    custom_size: Some(Vec2::new(1.0, 1.0)),
                    ..default()
                },
                Transform::from_xyz(wx, wy, Z_FARM_OVERLAY + 1.5),
                Visibility::default(),
            ));
        }
    }
}

/// Each frame: fade alpha from 0.3 → 0, no movement, despawn when done.
pub fn update_water_drips(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut WaterDrip)>,
) {
    for (entity, mut sprite, mut drip) in query.iter_mut() {
        drip.timer.tick(time.delta());

        if drip.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let fraction = 1.0 - drip.timer.fraction(); // 1.0 → 0.0
        let alpha = drip.initial_alpha * fraction;
        sprite.color = Color::srgba(0.3, 0.5, 0.9, alpha);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ScytheArc — brief wide slash arc on scythe impact
// ═══════════════════════════════════════════════════════════════════════════

/// A brief slash arc visual spawned when the scythe impacts a tile.
#[derive(Component)]
pub struct ScytheArc {
    pub timer: Timer,
}

/// Listen for Scythe impact events and spawn a slash arc visual.
pub fn spawn_scythe_arc(
    mut commands: Commands,
    mut impact_events: EventReader<ToolImpactEvent>,
    player_query: Query<&PlayerMovement, With<Player>>,
) {
    let facing = player_query
        .get_single()
        .map(|pm| pm.facing)
        .unwrap_or(Facing::Down);

    for event in impact_events.read() {
        if event.tool != ToolKind::Scythe {
            continue;
        }

        let wx = event.grid_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let wy = event.grid_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        // Slight directional offset toward facing direction
        let (offset_x, offset_y) = match facing {
            Facing::Up => (0.0, 2.0),
            Facing::Down => (0.0, -2.0),
            Facing::Left => (-2.0, 0.0),
            Facing::Right => (2.0, 0.0),
        };

        // Rotation: horizontal slash for left/right, vertical for up/down
        let rotation_rad = match facing {
            Facing::Up | Facing::Down => std::f32::consts::FRAC_PI_2, // 90 degrees for vertical
            Facing::Left | Facing::Right => 0.0, // horizontal slash
        };

        commands.spawn((
            ScytheArc {
                timer: Timer::from_seconds(0.1, TimerMode::Once),
            },
            Sprite {
                color: Color::srgba(0.9, 0.95, 1.0, 0.4),
                // Wide horizontal slash: 14x6 pixels
                custom_size: Some(Vec2::new(14.0, 6.0)),
                ..default()
            },
            Transform::from_xyz(wx + offset_x, wy + offset_y, Z_EFFECTS - 1.0)
                .with_scale(Vec3::splat(0.8))
                .with_rotation(Quat::from_rotation_z(rotation_rad)),
            Visibility::default(),
        ));
    }
}

/// Each frame: scale from 0.8 → 1.3, fade alpha 0.4 → 0, despawn when done.
pub fn update_scythe_arc(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut ScytheArc)>,
) {
    for (entity, mut tf, mut sprite, mut arc) in query.iter_mut() {
        arc.timer.tick(time.delta());

        if arc.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let t = arc.timer.fraction(); // 0.0 → 1.0
        // Scale from 0.8 to 1.3
        let scale = 0.8 + t * 0.5;
        // Preserve Z rotation while scaling uniformly
        let current_rot = tf.rotation;
        tf.scale = Vec3::new(scale, scale, 1.0);
        tf.rotation = current_rot;

        // Alpha fades from 0.4 to 0 rapidly
        let alpha = 0.4 * (1.0 - t);
        sprite.color = Color::srgba(0.9, 0.95, 1.0, alpha);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CutDebris — thin cut fragments floating up after scythe hit
// ═══════════════════════════════════════════════════════════════════════════

/// A tiny cut debris fragment that floats upward gently after a scythe cut.
#[derive(Component)]
pub struct CutDebris {
    pub timer: Timer,
    pub velocity: Vec2,
    pub initial_alpha: f32,
}

/// Spawn 2-3 cut debris fragments when the scythe impacts a tile with grass/crop.
pub fn spawn_cut_debris(
    mut commands: Commands,
    mut impact_events: EventReader<ToolImpactEvent>,
    farm_state: Res<FarmState>,
) {
    let mut rng = rand::thread_rng();

    for event in impact_events.read() {
        if event.tool != ToolKind::Scythe {
            continue;
        }

        // Only spawn debris if there was something to cut (crop or no soil = grass area)
        let pos = (event.grid_x, event.grid_y);
        let has_crop = farm_state.crops.contains_key(&pos);
        // If there's no soil record at all, assume it's a grass/foliage tile
        let is_grass = !farm_state.soil.contains_key(&pos);

        if !has_crop && !is_grass {
            continue;
        }

        let wx = event.grid_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let wy = event.grid_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        let debris_count = rng.gen_range(2..=3usize);
        for _ in 0..debris_count {
            // Random upward + sideways drift: slow, gentle
            let vx = rng.gen_range(-15.0f32..15.0);
            let vy = rng.gen_range(10.0f32..25.0); // upward drift

            // Green or yellow-green fragment color
            let green_mix = rng.gen_range(0.0f32..1.0);
            let (r, g, b) = if green_mix > 0.5 {
                // Vivid green
                (0.25, 0.65, 0.2)
            } else {
                // Yellow-green
                (0.55, 0.70, 0.2)
            };

            let lifetime = rng.gen_range(0.6f32..0.8);

            commands.spawn((
                CutDebris {
                    timer: Timer::from_seconds(lifetime, TimerMode::Once),
                    velocity: Vec2::new(vx, vy),
                    initial_alpha: 0.85,
                },
                Sprite {
                    color: Color::srgba(r, g, b, 0.85),
                    // Thin cut pieces: 2x1 pixels
                    custom_size: Some(Vec2::new(2.0, 1.0)),
                    ..default()
                },
                Transform::from_xyz(wx, wy, Z_EFFECTS - 2.0),
                Visibility::default(),
            ));
        }
    }
}

/// Each frame: move fragments upward/sideways, fade alpha, despawn when done.
pub fn update_cut_debris(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut CutDebris)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut debris) in query.iter_mut() {
        debris.timer.tick(time.delta());

        if debris.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        // Move gently
        tf.translation.x += debris.velocity.x * dt;
        tf.translation.y += debris.velocity.y * dt;

        // Very gentle deceleration — not gravity, just air resistance
        debris.velocity.y -= 8.0 * dt;

        // Fade alpha proportional to remaining lifetime
        let fraction = 1.0 - debris.timer.fraction();
        let alpha = debris.initial_alpha * fraction;
        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(base.red, base.green, base.blue, alpha);
    }
}
