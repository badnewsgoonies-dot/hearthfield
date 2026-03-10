//! Visual impact feedback when the player mines rocks with a pickaxe.
//!
//! Provides:
//! - `RockShake` — oscillates the rock sprite ±1.5px in X over 0.12s on hit
//! - `DamageFlash` — flashes the rock sprite white for one frame (0.05s) on hit
//! - Spark particles — 2-3 bright yellow-white sparks on hit
//! - Rock destruction burst — 6-8 stone fragments + dust poof on destroy

use bevy::prelude::*;
use rand::Rng;

use super::components::{InMine, MineFloorEntity};
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Applied to a rock entity on pickaxe hit.
/// Oscillates the rock's translation.x around `original_x` for the shake duration.
#[derive(Component, Debug)]
pub struct RockShake {
    pub timer: Timer,
    pub original_x: f32,
}

/// Applied to a rock entity on pickaxe hit.
/// Sets the sprite colour to a bright overexposed white for 0.05 s then removes itself.
#[derive(Component, Debug)]
pub struct DamageFlash {
    pub timer: Timer,
}

/// A short-lived spark particle spawned on rock impact (metal-on-stone sparks).
#[derive(Component, Debug)]
pub struct RockSpark {
    pub lifetime: Timer,
    pub velocity: Vec2,
}

/// A heavier stone-fragment particle spawned when a rock is fully destroyed.
#[derive(Component, Debug)]
pub struct StoneFragment {
    pub lifetime: Timer,
    pub velocity: Vec2,
    pub gravity: f32,
    pub initial_alpha: f32,
}

/// A gray dust poof spawned when a rock is fully destroyed.
#[derive(Component, Debug)]
pub struct RockDestroyPoof {
    pub timer: Timer,
}

// ═══════════════════════════════════════════════════════════════════════
// EVENTS (internal to mining domain)
// ═══════════════════════════════════════════════════════════════════════

/// Fired by `rock_breaking` when a rock takes damage but is NOT destroyed.
#[derive(Event, Debug, Clone)]
pub struct RockHitEvent {
    pub rock_entity: Entity,
    pub world_x: f32,
    pub world_y: f32,
}

/// Fired by `rock_breaking` when a rock is destroyed.
#[derive(Event, Debug, Clone)]
pub struct RockDestroyedEvent {
    pub world_x: f32,
    pub world_y: f32,
}

// ═══════════════════════════════════════════════════════════════════════
// REACTION SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// System: on `RockHitEvent`, add `RockShake` and `DamageFlash` to the rock entity
/// and spawn 2-3 spark particles at the impact position.
pub fn handle_rock_hit_effects(
    mut commands: Commands,
    mut hit_events: EventReader<RockHitEvent>,
    rocks: Query<&Transform, With<MineRock>>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        hit_events.clear();
        return;
    }

    let mut rng = rand::thread_rng();

    for event in hit_events.read() {
        // Shake + flash on the rock entity
        if let Ok(transform) = rocks.get(event.rock_entity) {
            let original_x = transform.translation.x;
            commands.entity(event.rock_entity).insert((
                RockShake {
                    timer: Timer::from_seconds(0.12, TimerMode::Once),
                    original_x,
                },
                DamageFlash {
                    timer: Timer::from_seconds(0.05, TimerMode::Once),
                },
            ));
        }

        // Spawn 2-3 spark particles
        let count = rng.gen_range(2usize..=3);
        for _ in 0..count {
            let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
            let speed = rng.gen_range(80.0f32..120.0);
            let lifetime_secs = rng.gen_range(0.10f32..0.15);

            commands.spawn((
                RockSpark {
                    lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                    velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                },
                Sprite {
                    color: Color::srgba(1.0, 0.9, 0.5, 0.9),
                    custom_size: Some(Vec2::splat(1.0)),
                    ..default()
                },
                Transform::from_xyz(event.world_x, event.world_y, Z_EFFECTS),
                Visibility::default(),
            ));
        }
    }
}

/// System: on `RockDestroyedEvent`, spawn 6-8 stone fragments + a gray dust poof.
pub fn handle_rock_destroyed_effects(
    mut commands: Commands,
    mut destroy_events: EventReader<RockDestroyedEvent>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        destroy_events.clear();
        return;
    }

    let mut rng = rand::thread_rng();

    for event in destroy_events.read() {
        let wx = event.world_x;
        let wy = event.world_y;

        // 6-8 stone fragments
        let count = rng.gen_range(6usize..=8);
        for _ in 0..count {
            let angle = rng.gen_range(0.0f32..std::f32::consts::TAU);
            let speed = rng.gen_range(60.0f32..120.0);
            let lifetime_secs = rng.gen_range(0.40f32..0.60);
            let initial_alpha = 0.85f32;

            commands.spawn((
                StoneFragment {
                    lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
                    velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                    gravity: 120.0,
                    initial_alpha,
                },
                Sprite {
                    color: Color::srgba(0.55, 0.55, 0.55, initial_alpha),
                    custom_size: Some(Vec2::splat(3.0)),
                    ..default()
                },
                Transform::from_xyz(wx, wy, Z_EFFECTS),
                Visibility::default(),
                MineFloorEntity,
            ));
        }

        // Gray dust poof (same pattern as TillPoof but gray, 10x10 px)
        commands.spawn((
            RockDestroyPoof {
                timer: Timer::from_seconds(0.25, TimerMode::Once),
            },
            Sprite {
                color: Color::srgba(0.55, 0.55, 0.55, 0.35),
                custom_size: Some(Vec2::splat(10.0)),
                ..default()
            },
            Transform::from_xyz(wx, wy, Z_EFFECTS - 1.0).with_scale(Vec3::splat(0.5)),
            Visibility::default(),
            MineFloorEntity,
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE SYSTEMS (per-frame)
// ═══════════════════════════════════════════════════════════════════════

/// System: oscillate shaking rocks and clean up when timer expires.
pub fn update_rock_shake(
    mut commands: Commands,
    time: Res<Time>,
    mut rocks: Query<(Entity, &mut Transform, &mut RockShake)>,
) {
    for (entity, mut transform, mut shake) in rocks.iter_mut() {
        shake.timer.tick(time.delta());

        if shake.timer.finished() {
            // Restore exact original X and remove the component
            transform.translation.x = shake.original_x;
            commands.entity(entity).remove::<RockShake>();
        } else {
            // Oscillate: sin over [0, π] produces a smooth out-and-back motion
            let t = shake.timer.fraction(); // 0..1
            let offset = (t * std::f32::consts::PI * 4.0).sin() * 1.5;
            transform.translation.x = shake.original_x + offset;
        }
    }
}

/// System: set overexposed white colour during flash, reset to white when done.
pub fn update_damage_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut rocks: Query<(Entity, &mut Sprite, &mut DamageFlash)>,
) {
    for (entity, mut sprite, mut flash) in rocks.iter_mut() {
        flash.timer.tick(time.delta());

        if flash.timer.finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<DamageFlash>();
        } else {
            // Overexposed white (HDR-style: values > 1.0 bloom in bright scenes)
            sprite.color = Color::srgb(2.0, 2.0, 2.0);
        }
    }
}

/// System: move sparks, fade alpha, despawn on expiry.
pub fn update_rock_sparks(
    mut commands: Commands,
    time: Res<Time>,
    mut sparks: Query<(Entity, &mut Transform, &mut Sprite, &mut RockSpark)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut spark) in sparks.iter_mut() {
        spark.lifetime.tick(time.delta());

        if spark.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        tf.translation.x += spark.velocity.x * dt;
        tf.translation.y += spark.velocity.y * dt;

        // Fade out over lifetime
        let alpha = 0.9 * (1.0 - spark.lifetime.fraction());
        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(base.red, base.green, base.blue, alpha);
    }
}

/// System: move stone fragments with gravity, fade alpha, despawn on expiry.
pub fn update_stone_fragments(
    mut commands: Commands,
    time: Res<Time>,
    mut frags: Query<(Entity, &mut Transform, &mut Sprite, &mut StoneFragment)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut sprite, mut frag) in frags.iter_mut() {
        frag.lifetime.tick(time.delta());

        if frag.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        tf.translation.x += frag.velocity.x * dt;
        tf.translation.y += frag.velocity.y * dt;
        frag.velocity.y -= frag.gravity * dt;

        let alpha = frag.initial_alpha * (1.0 - frag.lifetime.fraction());
        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(base.red, base.green, base.blue, alpha);
    }
}

/// System: expand and fade the rock destroy dust poof, despawn on expiry.
pub fn update_rock_destroy_poof(
    mut commands: Commands,
    time: Res<Time>,
    mut poofs: Query<(Entity, &mut Transform, &mut Sprite, &mut RockDestroyPoof)>,
) {
    for (entity, mut tf, mut sprite, mut poof) in poofs.iter_mut() {
        poof.timer.tick(time.delta());

        if poof.timer.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let t = poof.timer.fraction(); // 0..1
        let scale = 0.5 + t * 1.2;   // 0.5 → 1.7
        tf.scale = Vec3::splat(scale);

        let alpha = 0.35 * (1.0 - t);
        sprite.color = Color::srgba(0.55, 0.55, 0.55, alpha);
    }
}
