//! Seasonal visual effects: terrain/tree tinting and autumn leaf particles.
//!
//! This module owns:
//! - `SeasonalTintApplied` — resource tracking the last season we tinted for
//! - `apply_seasonal_tint` — queries `MapTile` / `WorldObject` entities and
//!   multiplies their sprite colour by a season-specific tint when the season
//!   changes.
//! - `FallingLeaf` — component for autumn leaf particle entities
//! - `spawn_falling_leaves` — spawns one leaf per ~60 frames during Fall
//! - `update_falling_leaves` — drifts leaves downward with sine oscillation
//!   and despawns them when they fall off-screen or live too long.

use bevy::prelude::*;
use rand::Rng;

use crate::shared::*;

use super::MapTile;
use super::objects::{WorldObject, WorldObjectData};
use super::maps::WorldObjectKind;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Tracks which season the last tint pass was applied for.
/// Initialised to `None` so the first Playing frame always triggers a tint.
#[derive(Resource, Debug, Clone)]
pub struct SeasonalTintApplied {
    pub season: Option<Season>,
}

impl Default for SeasonalTintApplied {
    fn default() -> Self {
        Self { season: None }
    }
}

/// Accumulates fractional frames so we spawn exactly 1 leaf per 60 frames on
/// average, regardless of frame rate.
#[derive(Resource, Default)]
pub struct LeafSpawnAccumulator {
    pub frames: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marks a falling-leaf particle entity spawned during autumn.
#[derive(Component, Debug)]
pub struct FallingLeaf {
    /// Seconds this leaf has been alive.
    pub age: f32,
    /// Base horizontal position used for sine oscillation.
    pub base_x: f32,
    /// Oscillation frequency (radians per second).
    pub frequency: f32,
    /// Oscillation amplitude in pixels.
    pub amplitude: f32,
    /// Downward speed in pixels per second.
    pub fall_speed: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// COLOUR HELPERS
// ═══════════════════════════════════════════════════════════════════════

/// Returns the season tint multiplier colour for terrain tiles.
///
/// The tint is applied as a multiplicative colour: White = no change.
fn terrain_tint(season: Season) -> Color {
    match season {
        Season::Spring => Color::WHITE, // lush — no modification needed
        Season::Summer => Color::srgb(1.0, 0.95, 0.85), // warm golden haze
        Season::Fall   => Color::srgb(1.0, 0.80, 0.55), // orange warmth
        Season::Winter => Color::srgb(0.85, 0.90, 1.00), // cool blue-white
    }
}

/// Returns the tint colour for tree/bush objects based on season.
/// Each tree gets either the "a" or "b" variant depending on a hash of its
/// grid position, so the same map looks varied across adjacent trees.
fn tree_tint(season: Season, variant_b: bool) -> Color {
    match season {
        Season::Spring => {
            if variant_b {
                Color::srgb(1.0, 0.7, 0.8) // cherry blossom pink
            } else {
                Color::srgb(0.7, 1.0, 0.6) // light spring green
            }
        }
        Season::Summer => Color::srgb(0.6, 0.9, 0.5), // deep summer green
        Season::Fall => {
            if variant_b {
                Color::srgb(1.0, 0.85, 0.30) // golden yellow
            } else {
                Color::srgb(1.0, 0.60, 0.30) // burnt orange
            }
        }
        Season::Winter => Color::srgb(0.6, 0.5, 0.4), // bare brown-grey
    }
}

/// Returns the tint for non-tree world objects (rocks, stumps, bushes, logs).
/// Rocks and stone objects should stay mostly neutral; only get a slight
/// season shift for consistency.
fn object_tint(season: Season) -> Color {
    match season {
        Season::Spring => Color::WHITE,
        Season::Summer => Color::srgb(1.0, 0.97, 0.90),
        Season::Fall   => Color::srgb(0.95, 0.90, 0.80),
        Season::Winter => Color::srgb(0.90, 0.93, 1.00),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Apply a seasonal colour tint to all terrain tiles and world objects.
///
/// This system runs in the Playing state every frame, but the heavy work only
/// triggers when the season tracked in `SeasonalTintApplied` differs from the
/// current `Calendar` season.  That way we re-tint on: game start, season
/// change events, and map transitions (when the resource is not reset — the
/// new season is still in the Calendar so the first frame of the new map
/// will pick it up automatically).
pub fn apply_seasonal_tint(
    calendar: Res<Calendar>,
    mut tint_applied: ResMut<SeasonalTintApplied>,
    mut tile_query: Query<&mut Sprite, (With<MapTile>, Without<WorldObject>)>,
    mut object_query: Query<(&mut Sprite, &WorldObjectData), (With<WorldObject>, Without<MapTile>)>,
) {
    let current_season = calendar.season;

    // Only do work when the season has changed (or on first run when it's None).
    if tint_applied.season == Some(current_season) {
        return;
    }

    // ── Terrain tiles ─────────────────────────────────────────────────────────
    let t = terrain_tint(current_season);
    for mut sprite in tile_query.iter_mut() {
        // Only tint textured tiles; skip plain-colored Void tiles
        if sprite.texture_atlas.is_some() {
            sprite.color = t;
        }
    }

    // ── World objects (trees, rocks, bushes, stumps, logs) ────────────────────
    for (mut sprite, obj_data) in object_query.iter_mut() {
        let tint = match obj_data.kind {
            WorldObjectKind::Tree => {
                // Use grid position hash to choose variant so adjacent trees differ.
                let variant_b = (obj_data.grid_x.wrapping_add(obj_data.grid_y * 3)) % 2 == 1;
                tree_tint(current_season, variant_b)
            }
            WorldObjectKind::Bush => {
                // Bushes get the same tree tint in autumn/winter for foliage variety.
                tree_tint(current_season, false)
            }
            _ => object_tint(current_season),
        };
        sprite.color = tint;
    }

    // Record that we've applied this season.
    tint_applied.season = Some(current_season);
}

// ─────────────────────────────────────────────────────────────────────────────
// Falling leaf particles
// ─────────────────────────────────────────────────────────────────────────────

/// Spawn a single falling leaf above the camera viewport during autumn.
///
/// Rate: approximately one leaf every 60 frames (tracked via `LeafSpawnAccumulator`).
pub fn spawn_falling_leaves(
    mut commands: Commands,
    calendar: Res<Calendar>,
    mut accum: ResMut<LeafSpawnAccumulator>,
    time: Res<Time>,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    // Only active in Fall.
    if calendar.season != Season::Fall {
        return;
    }

    // Accumulate time and emit at ~60fps cadence.
    accum.frames += time.delta_secs() * 60.0;

    if accum.frames < 60.0 {
        return;
    }
    accum.frames -= 60.0;

    let camera_pos = camera_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let mut rng = rand::thread_rng();

    // Spawn position: random x across the visible width, just above the top edge.
    let half_w = SCREEN_WIDTH * 0.5 + 32.0; // slightly wider than screen
    let spawn_x = camera_pos.x + rng.gen_range(-half_w..half_w);
    let spawn_y = camera_pos.y + SCREEN_HEIGHT * 0.5 + 16.0; // above top edge

    // Pick a leaf colour: orange or deep red.
    let color = if rng.gen_bool(0.5) {
        Color::srgb(1.0, 0.55, 0.15) // vivid orange
    } else {
        Color::srgb(0.85, 0.25, 0.10) // deep red
    };

    let fall_speed = rng.gen_range(20.0_f32..40.0);
    let frequency = rng.gen_range(1.5_f32..3.5);
    let amplitude = rng.gen_range(8.0_f32..20.0);

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(4.0, 4.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(spawn_x, spawn_y, Z_SEASONAL)),
        FallingLeaf {
            age: 0.0,
            base_x: spawn_x,
            frequency,
            amplitude,
            fall_speed,
        },
    ));
}

/// Animate and cull falling leaf particles.
///
/// Each leaf drifts downward at `fall_speed` and oscillates horizontally using
/// a sine wave, giving a gentle fluttering appearance.
/// Leaves are despawned when they have fallen below the camera's lower edge or
/// have been alive for more than 10 seconds.
pub fn update_falling_leaves(
    mut commands: Commands,
    mut leaves: Query<(Entity, &mut Transform, &mut FallingLeaf)>,
    camera_query: Query<&Transform, (With<Camera2d>, Without<FallingLeaf>)>,
    time: Res<Time>,
    calendar: Res<Calendar>,
) {
    let camera_y = camera_query
        .iter()
        .next()
        .map(|t| t.translation.y)
        .unwrap_or(0.0);

    let despawn_y = camera_y - SCREEN_HEIGHT * 0.5 - 16.0;
    let dt = time.delta_secs();

    for (entity, mut transform, mut leaf) in leaves.iter_mut() {
        leaf.age += dt;

        // Despawn if too old or off the bottom of the screen.
        if leaf.age > 10.0 || transform.translation.y < despawn_y {
            commands.entity(entity).despawn();
            continue;
        }

        // Drift downward.
        transform.translation.y -= leaf.fall_speed * dt;

        // Sine-wave horizontal oscillation.
        let x_offset = leaf.amplitude * (leaf.age * leaf.frequency).sin();
        transform.translation.x = leaf.base_x + x_offset;
    }

    // If we're no longer in Fall, despawn all remaining leaves.
    if calendar.season != Season::Fall {
        for (entity, _, _) in leaves.iter() {
            commands.entity(entity).despawn();
        }
    }
}
