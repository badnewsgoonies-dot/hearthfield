//! Visual feedback effects when the player chops trees with an axe.
//!
//! Systems here are pure presentation-layer effects — they do NOT modify
//! `WorldObjectData` health or any simulation state. They only touch
//! `Transform`, `Sprite`, and spawned VFX entities.
//!
//! Effects provided:
//! - **TreeShake**: oscillates a tree's X position on axe impact
//! - **TreeFlash**: flashes the tree sprite white for one frame on impact
//! - **LeafParticle**: 3-5 leaf burst particles on axe impact
//! - **TreeDestructionPoof**: expanding dust cloud when a tree is destroyed

use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

use super::maps::WorldObjectKind;
use super::objects::WorldObject;
use super::objects::WorldObjectData;

// ─────────────────────────────────────────────────────────────────────────────
// Components
// ─────────────────────────────────────────────────────────────────────────────

/// Attached to a tree entity when it takes an axe hit.
/// Drives the shake oscillation in `update_tree_shake`.
#[derive(Component)]
pub struct TreeShake {
    pub timer: Timer,
    pub original_x: f32,
}

/// Attached to a tree entity for exactly one frame after an axe hit.
/// Stores the original sprite color so `update_tree_flash` can restore it.
#[derive(Component)]
pub struct TreeFlash {
    /// Elapsed seconds since the flash was applied (restored after first update).
    pub elapsed: f32,
    /// The sprite color before the flash.
    pub original_color: Color,
}

/// A leaf particle emitted upward from a tree top on axe impact.
#[derive(Component)]
pub struct LeafParticle {
    pub lifetime: f32,
    pub elapsed: f32,
    pub velocity: Vec2,
    /// Horizontal flutter: amplitude in pixels.
    pub flutter_amp: f32,
    /// Flutter frequency in radians/second.
    pub flutter_freq: f32,
    /// Phase offset for sinusoidal flutter (avoids all leaves fluttering in sync).
    pub flutter_phase: f32,
    pub initial_alpha: f32,
}

/// Expanding dust cloud spawned when a tree is fully destroyed.
/// Same pattern as `TillPoof` in `tool_anim.rs`, but larger.
#[derive(Component)]
pub struct TreeDestructionPoof {
    pub timer: Timer,
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: seasonal leaf color
// ─────────────────────────────────────────────────────────────────────────────

fn leaf_color(season: Season) -> Color {
    match season {
        Season::Spring => Color::srgba(0.35, 0.75, 0.25, 0.9),  // bright green
        Season::Summer => Color::srgba(0.20, 0.60, 0.20, 0.9),  // deep green
        Season::Fall => Color::srgba(0.80, 0.45, 0.10, 0.9),    // orange-brown
        Season::Winter => Color::srgba(0.70, 0.70, 0.70, 0.7),  // pale grey (sparse)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: on_axe_tree_impact
// ─────────────────────────────────────────────────────────────────────────────

/// Reads `ToolImpactEvent` for axe impacts, finds any tree entity at the target
/// grid position, then:
/// 1. Inserts/resets `TreeShake` on the tree entity.
/// 2. Inserts `TreeFlash` so the sprite turns white for one frame.
/// 3. Spawns 3-5 `LeafParticle` entities bursting upward from the tree top.
#[allow(clippy::too_many_arguments)]
pub fn on_axe_tree_impact(
    mut commands: Commands,
    mut impact_events: EventReader<ToolImpactEvent>,
    mut tree_query: Query<(Entity, &WorldObjectData, &Transform, &mut Sprite), With<WorldObject>>,
    calendar: Res<Calendar>,
) {
    let mut rng = rand::thread_rng();

    for event in impact_events.read() {
        // Only care about axe impacts.
        if event.tool != ToolKind::Axe {
            continue;
        }

        // Find a tree entity whose grid position matches the impact tile.
        // Trees and pines both count.
        let tree_entity = tree_query
            .iter()
            .find(|(_, data, _, _)| {
                data.grid_x == event.grid_x
                    && data.grid_y == event.grid_y
                    && matches!(data.kind, WorldObjectKind::Tree | WorldObjectKind::Pine | WorldObjectKind::PalmTree)
            })
            .map(|(entity, _, _, _)| entity);

        let Some(entity) = tree_entity else {
            continue;
        };

        // Retrieve transform and sprite for the found entity.
        let Ok((_, _data, transform, mut sprite)) = tree_query.get_mut(entity) else {
            continue;
        };

        let original_x = transform.translation.x;
        let original_color = sprite.color;

        // 1. Insert (or replace) TreeShake.
        commands.entity(entity).insert(TreeShake {
            timer: Timer::from_seconds(0.15, TimerMode::Once),
            original_x,
        });

        // 2. Flash sprite to bright white.
        sprite.color = Color::srgb(2.0, 2.0, 2.0);
        commands.entity(entity).insert(TreeFlash {
            elapsed: 0.0,
            original_color,
        });

        // 3. Spawn 3-5 leaf particles from slightly above the tree centre.
        //    Tree sprites are 32x48 pixels (2×3 tiles). The "top" is about
        //    20px above the entity's centre (which sits at the base tile centre
        //    + y_offset from spawn_world_objects).
        let leaf_origin_x = transform.translation.x;
        let leaf_origin_y = transform.translation.y + 20.0;
        let color = leaf_color(calendar.season);
        let count = rng.gen_range(3..=5usize);

        for i in 0..count {
            // Burst upward and outward.
            let angle = std::f32::consts::FRAC_PI_2                  // base: straight up
                + rng.gen_range(-0.9_f32..0.9);                       // ±52° spread
            let speed = rng.gen_range(35.0_f32..70.0);
            let vx = angle.cos() * speed;
            let vy = angle.sin() * speed;
            let lifetime = rng.gen_range(0.35_f32..0.55);
            let flutter_amp = rng.gen_range(2.0_f32..5.0);
            let flutter_freq = rng.gen_range(4.0_f32..8.0);
            let flutter_phase = i as f32 * 1.1; // spread phases

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_xyz(leaf_origin_x, leaf_origin_y, Z_EFFECTS),
                LeafParticle {
                    lifetime,
                    elapsed: 0.0,
                    velocity: Vec2::new(vx, vy),
                    flutter_amp,
                    flutter_freq,
                    flutter_phase,
                    initial_alpha: color.to_srgba().alpha,
                },
            ));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: update_tree_shake
// ─────────────────────────────────────────────────────────────────────────────

/// Each frame, oscillate tree X ±2 pixels over the shake duration.
/// Removes `TreeShake` and restores original X when the timer finishes.
pub fn update_tree_shake(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TreeShake, &mut Transform)>,
) {
    let dt = time.delta();
    let elapsed_total = time.elapsed_secs();

    for (entity, mut shake, mut transform) in query.iter_mut() {
        shake.timer.tick(dt);

        if shake.timer.finished() {
            transform.translation.x = shake.original_x;
            commands.entity(entity).remove::<TreeShake>();
        } else {
            // 3 oscillations in 0.15s → frequency = 3/0.15 = 20 Hz = 2π·20 rad/s ≈ 125.7
            let freq = 20.0_f32 * std::f32::consts::TAU;
            let amplitude = 2.0_f32; // pixels
            // Use elapsed_total so the oscillation phase is continuous and frame-rate independent.
            let offset = (elapsed_total * freq).sin() * amplitude;
            transform.translation.x = shake.original_x + offset;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: update_tree_flash
// ─────────────────────────────────────────────────────────────────────────────

/// Restores a tree's sprite color one frame after `TreeFlash` was inserted.
/// Runs every frame. After the first tick (elapsed > 0.0 before tick, which
/// means this is at least the second frame), the original color is restored
/// and the component is removed.
pub fn update_tree_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut TreeFlash, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mut sprite) in query.iter_mut() {
        flash.elapsed += dt;
        if flash.elapsed >= 0.016 {
            // One frame (~16ms) has elapsed — restore original color.
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<TreeFlash>();
        }
        // On the very first tick (elapsed just set to dt), we leave the white
        // color in place so the player actually sees the flash frame.
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: update_leaf_particles
// ─────────────────────────────────────────────────────────────────────────────

/// Each frame: move leaf particles, apply gentle gravity, flutter X sinusoidally,
/// fade alpha, and despawn when expired.
pub fn update_leaf_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut LeafParticle)>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    const GRAVITY: f32 = 30.0; // px/s² — gentle drift down

    for (entity, mut tf, mut sprite, mut leaf) in query.iter_mut() {
        leaf.elapsed += dt;

        if leaf.elapsed >= leaf.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        // Gravity
        leaf.velocity.y -= GRAVITY * dt;

        // Move
        tf.translation.x += leaf.velocity.x * dt;
        tf.translation.y += leaf.velocity.y * dt;

        // Sinusoidal flutter on X (independent of velocity)
        let flutter_x = (elapsed * leaf.flutter_freq + leaf.flutter_phase).sin() * leaf.flutter_amp;
        tf.translation.x += flutter_x * dt;

        // Fade alpha
        let frac = 1.0 - (leaf.elapsed / leaf.lifetime);
        let alpha = leaf.initial_alpha * frac;
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: on_tree_destruction
// ─────────────────────────────────────────────────────────────────────────────

/// Spawns a `TreeDestructionPoof` when the axe kills a tree (health hits 0).
/// Reads `ToolUseEvent` and checks if any tree at that position has 1 health
/// remaining — meaning the current hit will destroy it.
///
/// NOTE: `handle_tool_use_on_objects` also reads `ToolUseEvent` and actually
/// destroys the entity. Because both systems run in `UpdatePhase::Simulation`,
/// the order is non-deterministic, so we spawn the poof proactively based on
/// the "would die" condition (health == damage amount), then let the other
/// system actually despawn the entity. The poof entity is independent and
/// survives the tree despawn.
pub fn on_tree_destruction(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    tree_query: Query<&WorldObjectData, With<WorldObject>>,
) {
    for event in tool_events.read() {
        if event.tool != ToolKind::Axe {
            continue;
        }

        // Find a tree at the target position that would be destroyed this hit.
        let should_poof = tree_query.iter().any(|data| {
            data.grid_x == event.target_x
                && data.grid_y == event.target_y
                && matches!(data.kind, WorldObjectKind::Tree | WorldObjectKind::Pine | WorldObjectKind::PalmTree)
                && {
                    let damage = data.kind.tool_damage(event.tier);
                    data.health <= damage
                }
        });

        if !should_poof {
            continue;
        }

        // Convert grid to world for poof spawn position.
        let wx = event.target_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
        let wy = event.target_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

        // Spawn poof: 12x12 pixels, brown-green tint, scale 0.5→2.0 over 0.3s.
        commands.spawn((
            TreeDestructionPoof {
                timer: Timer::from_seconds(0.3, TimerMode::Once),
            },
            Sprite {
                color: Color::srgba(0.45, 0.50, 0.30, 0.45),
                custom_size: Some(Vec2::new(12.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(wx, wy, Z_EFFECTS).with_scale(Vec3::splat(0.5)),
        ));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: update_tree_destruction_poof
// ─────────────────────────────────────────────────────────────────────────────

/// Each frame: scale the poof 0.5→2.0, fade alpha to 0, despawn when done.
pub fn update_tree_destruction_poof(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Sprite, &mut TreeDestructionPoof)>,
) {
    for (entity, mut tf, mut sprite, mut poof) in query.iter_mut() {
        poof.timer.tick(time.delta());

        if poof.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let t = poof.timer.fraction(); // 0.0 → 1.0
        tf.scale = Vec3::splat(0.5 + t * 1.5); // 0.5 → 2.0

        let alpha = 0.45 * (1.0 - t);
        sprite.color = Color::srgba(0.45, 0.50, 0.30, alpha);
    }
}
