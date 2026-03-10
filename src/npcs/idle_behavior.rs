//! NPC idle behaviors: subtle breathing animation, occasional look-around,
//! and grounding shadows beneath each NPC.

use super::animation::NpcAnimationTimer;
use super::spawning::NpcMovement;
use crate::shared::*;
use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

// ═══════════════════════════════════════════════════════════════════════
// IDLE BEHAVIOR COMPONENT
// ═══════════════════════════════════════════════════════════════════════

/// Drives subtle idle animations for stationary NPCs.
///
/// When an NPC is not moving, two behaviors occur:
/// - **Breathing**: gentle scale-Y oscillation (0.98–1.02 at ~0.4 Hz)
/// - **Look-around**: every 3–8 seconds, briefly face a random direction
#[derive(Component)]
pub struct NpcIdleBehavior {
    /// Accumulator for the breathing sine wave.
    pub breath_phase: f32,
    /// Countdown until the next look-around.
    pub look_timer: Timer,
    /// Duration the NPC holds its glance direction.
    pub glance_timer: Timer,
    /// Whether the NPC is currently glancing away.
    pub is_glancing: bool,
    /// The atlas base row to glance toward (0/4/8/12).
    pub glance_base: usize,
    /// The original base row to restore after glancing.
    pub original_base: usize,
}

impl Default for NpcIdleBehavior {
    fn default() -> Self {
        Self {
            breath_phase: 0.0,
            look_timer: Timer::from_seconds(5.0, TimerMode::Once),
            glance_timer: Timer::from_seconds(0.7, TimerMode::Once),
            is_glancing: false,
            glance_base: 0,
            original_base: 0,
        }
    }
}

/// Simple deterministic hash for per-NPC variation (avoids rand dependency).
fn simple_hash(id: &str) -> u32 {
    let mut h: u32 = 5381;
    for byte in id.bytes() {
        h = h.wrapping_mul(33).wrapping_add(byte as u32);
    }
    h
}

// ═══════════════════════════════════════════════════════════════════════
// SHADOW COMPONENT & SPRITE CACHE
// ═══════════════════════════════════════════════════════════════════════

/// Marker component for the shadow child entity beneath an NPC.
#[derive(Component)]
pub struct NpcShadow;

/// Cached shadow sprite image handle (generated once, reused for all NPCs).
#[derive(Resource, Default)]
pub struct ShadowSpriteCache {
    pub handle: Option<Handle<Image>>,
}

/// Generate an 8x4 semi-transparent elliptical shadow image.
fn make_shadow_image() -> Image {
    let w = 8usize;
    let h = 4usize;
    let mut data = vec![0u8; w * h * 4];

    // Elliptical shadow pattern (8x4):
    // Center at (3.5, 1.5), rx=3.5, ry=1.5
    // Pixels inside the ellipse get dark gray with low alpha.
    let pattern: [[u8; 8]; 4] = [
        [0, 0, 1, 1, 1, 1, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 1, 1, 1, 1, 0, 0],
    ];

    for (py, row) in pattern.iter().enumerate() {
        for (px, &pixel) in row.iter().enumerate() {
            let i = (py * w + px) * 4;
            if pixel == 1 {
                data[i] = 20;     // R - very dark
                data[i + 1] = 15; // G
                data[i + 2] = 15; // B
                data[i + 3] = 64; // A - ~0.25 alpha
            }
        }
    }

    let mut img = Image::new(
        Extent3d {
            width: w as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// System: apply idle behaviors (breathing + look-around) to stationary NPCs.
pub fn npc_idle_behavior_system(
    time: Res<Time>,
    mut query: Query<
        (
            &Npc,
            &NpcMovement,
            &mut Transform,
            &mut NpcAnimationTimer,
            &mut NpcIdleBehavior,
        ),
        With<Npc>,
    >,
) {
    let dt = time.delta_secs();

    for (npc, movement, mut transform, mut anim, mut idle) in query.iter_mut() {
        if movement.is_moving {
            // Reset breathing and cancel any active glance when moving.
            transform.scale.y = 1.0;
            if idle.is_glancing {
                idle.is_glancing = false;
                anim.last_base = idle.original_base;
            }
            // Randomize next look timer based on NPC id for variety.
            let hash = simple_hash(&npc.id);
            let next_interval = 3.0 + (hash % 500) as f32 / 100.0; // 3.0–8.0s
            idle.look_timer = Timer::from_seconds(next_interval, TimerMode::Once);
            continue;
        }

        // ── Breathing: gentle scale-Y oscillation ──
        // 0.4 Hz → period = 2.5s → angular freq = 2π * 0.4
        idle.breath_phase += dt * std::f32::consts::TAU * 0.4;
        if idle.breath_phase > std::f32::consts::TAU {
            idle.breath_phase -= std::f32::consts::TAU;
        }
        // Oscillate between 0.98 and 1.02
        transform.scale.y = 1.0 + 0.02 * idle.breath_phase.sin();

        // ── Look-around: occasional brief facing change ──
        if idle.is_glancing {
            idle.glance_timer.tick(time.delta());
            if idle.glance_timer.finished() {
                // Restore original facing
                idle.is_glancing = false;
                anim.last_base = idle.original_base;
            }
        } else {
            idle.look_timer.tick(time.delta());
            if idle.look_timer.finished() {
                // Start a glance
                idle.is_glancing = true;
                idle.original_base = anim.last_base;

                // Pick a different direction using a simple deterministic variation.
                // Use the breath_phase (which varies over time) for pseudo-randomness.
                let direction_seed =
                    (idle.breath_phase * 100.0) as u32 + simple_hash(&npc.id);
                let directions = [0usize, 4, 8, 12]; // down, left, right, up
                // Pick one that differs from current
                let mut pick = directions[(direction_seed as usize) % 4];
                if pick == anim.last_base {
                    pick = directions[((direction_seed as usize) + 1) % 4];
                }
                idle.glance_base = pick;
                anim.last_base = pick;

                // Random glance duration: 0.5–1.0s
                let glance_dur = 0.5 + (direction_seed % 50) as f32 / 100.0;
                idle.glance_timer = Timer::from_seconds(glance_dur, TimerMode::Once);

                // Random next look interval: 3–8s
                let next_interval = 3.0 + (direction_seed % 500) as f32 / 100.0;
                idle.look_timer = Timer::from_seconds(next_interval, TimerMode::Once);
            }
        }
    }
}

/// System: attach shadow child entities to NPCs that don't have one yet.
pub fn attach_npc_shadows(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut shadow_cache: ResMut<ShadowSpriteCache>,
    npc_query: Query<(Entity, &Children), With<Npc>>,
    shadow_check: Query<&NpcShadow>,
    npcs_without_children: Query<Entity, (With<Npc>, Without<Children>)>,
) {
    // Generate the shadow image once.
    let shadow_handle = shadow_cache.handle.get_or_insert_with(|| {
        images.add(make_shadow_image())
    }).clone();

    // Attach shadows to NPCs that already have children but no shadow child.
    for (entity, children) in npc_query.iter() {
        let has_shadow = children.iter().any(|c| shadow_check.get(*c).is_ok());
        if !has_shadow {
            let mut shadow_sprite = Sprite::from_image(shadow_handle.clone());
            shadow_sprite.custom_size = Some(Vec2::new(12.0, 6.0));

            commands.entity(entity).with_children(|parent| {
                parent.spawn((
                    NpcShadow,
                    shadow_sprite,
                    // Position at feet: anchored below the NPC sprite.
                    // NPC uses BottomCenter anchor, so (0, 0) is feet.
                    // Shadow sits just below, at slightly lower Z.
                    Transform::from_xyz(0.0, 0.0, -0.1),
                ));
            });
        }
    }

    // Also handle NPCs that have no children at all yet.
    for entity in npcs_without_children.iter() {
        let mut shadow_sprite = Sprite::from_image(shadow_handle.clone());
        shadow_sprite.custom_size = Some(Vec2::new(12.0, 6.0));

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                NpcShadow,
                shadow_sprite,
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
        });
    }
}
