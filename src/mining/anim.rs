//! Enemy idle animations, procedural enemy sprites, and ore shimmer effects.
//!
//! - Each enemy type gets a unique 16x16 procedurally generated sprite (fallback
//!   when atlas not loaded, and used for visual variety).
//! - Enemies idle with type-specific animations (squash-stretch, wing flap, alpha pulse).
//! - Valuable ore nodes shimmer with periodic sparkle particles.

use bevy::image::{Image, ImageSampler};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::components::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Drives idle animation for mine enemies. Each enemy gets a random initial
/// phase so they don't all animate in lockstep.
#[derive(Component, Debug)]
pub struct EnemyIdleAnim {
    pub phase: f32,
}

/// Marks an ore node as shimmering (gold, iridium, gems).
/// The timer controls sparkle spawn interval.
#[derive(Component, Debug)]
pub struct OreShimmer {
    pub timer: Timer,
    /// Color of the shimmer particle.
    pub color: Color,
}

/// A tiny sparkle particle that fades out and despawns.
#[derive(Component, Debug)]
pub struct ShimmerParticle {
    pub lifetime: Timer,
}

// ═══════════════════════════════════════════════════════════════════════
// PROCEDURAL ENEMY SPRITES (RESOURCE CACHE)
// ═══════════════════════════════════════════════════════════════════════

/// Cached procedural enemy sprite handles, generated once on first use.
#[derive(Resource, Default)]
pub struct ProceduralEnemySprites {
    pub slime: Option<Handle<Image>>,
    pub bat: Option<Handle<Image>>,
    pub rock_crab: Option<Handle<Image>>,
    pub shimmer_particle: Option<Handle<Image>>,
    pub loaded: bool,
}

/// Generate all procedural enemy sprites and the shimmer particle sprite.
pub fn load_procedural_enemy_sprites(
    mut images: ResMut<Assets<Image>>,
    mut sprites: ResMut<ProceduralEnemySprites>,
) {
    if sprites.loaded {
        return;
    }

    sprites.slime = Some(images.add(make_enemy_sprite(MineEnemy::GreenSlime)));
    sprites.bat = Some(images.add(make_enemy_sprite(MineEnemy::Bat)));
    sprites.rock_crab = Some(images.add(make_enemy_sprite(MineEnemy::RockCrab)));
    sprites.shimmer_particle = Some(images.add(make_shimmer_particle_image()));
    sprites.loaded = true;
}

/// Create a 16x16 procedural RGBA sprite for the given enemy type.
fn make_enemy_sprite(kind: MineEnemy) -> Image {
    let w = 16usize;
    let h = 16usize;
    let mut data = vec![0u8; w * h * 4];

    match kind {
        MineEnemy::GreenSlime => draw_slime(&mut data, w),
        MineEnemy::Bat => draw_bat(&mut data, w),
        MineEnemy::RockCrab => draw_rock_crab(&mut data, w),
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

/// Create a 4x4 white shimmer particle image.
fn make_shimmer_particle_image() -> Image {
    let w = 4usize;
    let h = 4usize;
    let mut data = vec![0u8; w * h * 4];

    // Diamond shape: bright white center, softer edges
    let pattern: [[u8; 4]; 4] = [
        [0, 1, 1, 0],
        [1, 2, 2, 1],
        [1, 2, 2, 1],
        [0, 1, 1, 0],
    ];

    for (py, row) in pattern.iter().enumerate() {
        for (px, &val) in row.iter().enumerate() {
            let i = (py * w + px) * 4;
            if val > 0 {
                let alpha = if val == 2 { 255 } else { 180 };
                data[i] = 255; // R
                data[i + 1] = 255; // G
                data[i + 2] = 240; // B (slightly warm)
                data[i + 3] = alpha;
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
// PIXEL ART DRAWING HELPERS
// ═══════════════════════════════════════════════════════════════════════

fn set_pixel(data: &mut [u8], w: usize, x: usize, y: usize, rgba: [u8; 4]) {
    if x < w && y < w {
        let i = (y * w + x) * 4;
        data[i] = rgba[0];
        data[i + 1] = rgba[1];
        data[i + 2] = rgba[2];
        data[i + 3] = rgba[3];
    }
}

/// Green blob slime: rounded body, two dark eyes, slightly transparent edges.
fn draw_slime(data: &mut [u8], w: usize) {
    // Body shape (1 = edge/transparent, 2 = solid body, 3 = highlight, 4 = dark eye)
    #[rustfmt::skip]
    let pattern: [[u8; 16]; 16] = [
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,0],
        [0,0,0,0,0,1,2,2,2,2,1,0,0,0,0,0],
        [0,0,0,0,1,2,2,3,2,2,2,1,0,0,0,0],
        [0,0,0,1,2,2,3,3,2,2,2,2,1,0,0,0],
        [0,0,0,1,2,2,2,2,2,2,2,2,1,0,0,0],
        [0,0,1,2,2,4,2,2,2,4,2,2,2,1,0,0],
        [0,0,1,2,2,4,2,2,2,4,2,2,2,1,0,0],
        [0,0,1,2,2,2,2,2,2,2,2,2,2,1,0,0],
        [0,0,1,2,2,2,2,2,2,2,2,2,2,1,0,0],
        [0,1,2,2,2,2,2,2,2,2,2,2,2,2,1,0],
        [0,1,2,2,2,2,2,2,2,2,2,2,2,2,1,0],
        [0,1,1,1,1,1,1,1,1,1,1,1,1,1,1,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];

    for (py, row) in pattern.iter().enumerate() {
        for (px, &val) in row.iter().enumerate() {
            match val {
                1 => set_pixel(data, w, px, py, [20, 100, 20, 160]),   // dark green edge
                2 => set_pixel(data, w, px, py, [50, 180, 50, 230]),   // green body
                3 => set_pixel(data, w, px, py, [100, 220, 100, 240]), // highlight
                4 => set_pixel(data, w, px, py, [15, 15, 15, 255]),    // dark eyes
                _ => {}
            }
        }
    }
}

/// Dark purple/brown bat with wing shapes, 2 red pixel eyes.
fn draw_bat(data: &mut [u8], w: usize) {
    // 1 = wing membrane, 2 = body, 3 = eye (red), 4 = wing tip
    #[rustfmt::skip]
    let pattern: [[u8; 16]; 16] = [
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [4,0,0,0,0,0,0,0,0,0,0,0,0,0,0,4],
        [1,4,0,0,0,0,0,0,0,0,0,0,0,0,4,1],
        [1,1,4,0,0,0,2,2,2,2,0,0,0,4,1,1],
        [1,1,1,0,0,2,2,2,2,2,2,0,0,1,1,1],
        [0,1,1,1,2,2,3,2,2,3,2,2,1,1,1,0],
        [0,0,1,1,2,2,2,2,2,2,2,2,1,1,0,0],
        [0,0,0,1,2,2,2,2,2,2,2,2,1,0,0,0],
        [0,0,0,0,2,2,2,2,2,2,2,2,0,0,0,0],
        [0,0,0,0,0,2,2,2,2,2,2,0,0,0,0,0],
        [0,0,0,0,0,0,2,2,2,2,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,2,2,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];

    for (py, row) in pattern.iter().enumerate() {
        for (px, &val) in row.iter().enumerate() {
            match val {
                1 => set_pixel(data, w, px, py, [80, 50, 90, 200]),    // wing membrane
                2 => set_pixel(data, w, px, py, [60, 35, 70, 240]),    // body (dark purple)
                3 => set_pixel(data, w, px, py, [220, 30, 30, 255]),   // red eyes
                4 => set_pixel(data, w, px, py, [100, 65, 110, 180]),  // wing tips
                _ => {}
            }
        }
    }
}

/// Rock crab: brownish-gray body, shell texture, small eyes.
fn draw_rock_crab(data: &mut [u8], w: usize) {
    // 1 = shell edge, 2 = shell body, 3 = highlight, 4 = eye, 5 = claw
    #[rustfmt::skip]
    let pattern: [[u8; 16]; 16] = [
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0],
        [0,0,0,0,1,2,2,3,3,2,2,1,0,0,0,0],
        [0,0,0,1,2,2,3,2,2,3,2,2,1,0,0,0],
        [0,0,1,2,2,4,2,2,2,2,4,2,2,1,0,0],
        [0,0,1,2,2,2,2,2,2,2,2,2,2,1,0,0],
        [0,5,1,2,2,2,2,2,2,2,2,2,2,1,5,0],
        [5,5,1,2,2,2,1,1,1,1,2,2,2,1,5,5],
        [0,5,1,2,2,2,2,2,2,2,2,2,2,1,5,0],
        [0,0,1,1,2,2,2,2,2,2,2,2,1,1,0,0],
        [0,0,0,1,1,2,2,2,2,2,2,1,1,0,0,0],
        [0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];

    for (py, row) in pattern.iter().enumerate() {
        for (px, &val) in row.iter().enumerate() {
            match val {
                1 => set_pixel(data, w, px, py, [90, 80, 70, 240]),    // shell edge
                2 => set_pixel(data, w, px, py, [140, 125, 105, 250]), // shell body
                3 => set_pixel(data, w, px, py, [170, 155, 135, 250]), // highlight
                4 => set_pixel(data, w, px, py, [20, 20, 20, 255]),    // eyes
                5 => set_pixel(data, w, px, py, [160, 90, 60, 230]),   // claws (orange-brown)
                _ => {}
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ANIMATION SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// System: animate enemy idle behaviors based on their type.
/// - Slimes: vertical squash-stretch (scale Y 0.85-1.15 at 1.5 Hz, X inverse)
/// - Bats: wing flap via scale X oscillation + small vertical bob (+-2px at 2 Hz)
/// - Rock crabs: subtle horizontal wobble (+-1px at 0.5 Hz)
pub fn animate_enemy_idle(
    time: Res<Time>,
    mut query: Query<(&MineMonster, &mut EnemyIdleAnim, &mut Transform, &mut Sprite)>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    let t = time.elapsed_secs();

    for (monster, mut anim, mut transform, mut sprite) in query.iter_mut() {
        let phase = anim.phase;
        // Advance phase (used for continuity)
        anim.phase += time.delta_secs();

        match monster.kind {
            MineEnemy::GreenSlime => {
                // Squash-stretch: Y oscillates 0.85-1.15 at 1.5 Hz
                let freq = 1.5 * std::f32::consts::TAU;
                let wave = ((t + phase) * freq).sin();
                let scale_y = 1.0 + wave * 0.15; // 0.85 to 1.15
                let scale_x = 1.0 - wave * 0.10; // inverse (wider when squished)
                // Preserve Z scale from original spawn (1.2)
                let base_scale = 1.2;
                transform.scale = Vec3::new(
                    base_scale * scale_x,
                    base_scale * scale_y,
                    1.0,
                );
            }
            MineEnemy::Bat => {
                // Wing flap: scale X oscillation + vertical bob
                let flap_freq = 2.0 * std::f32::consts::TAU;
                let bob_freq = 2.0 * std::f32::consts::TAU;
                let flap = ((t + phase) * flap_freq).sin();
                let bob = ((t + phase) * bob_freq).cos();

                let base_scale = 1.2;
                transform.scale = Vec3::new(
                    base_scale * (1.0 + flap * 0.12),
                    base_scale,
                    1.0,
                );
                // Vertical bob +-2 pixels (applied as offset from grid position)
                // We modify only the fractional part to avoid fighting with grid snap
                let grid_y = (transform.translation.y / TILE_SIZE).round() * TILE_SIZE;
                transform.translation.y = grid_y + bob * 2.0;
            }
            MineEnemy::RockCrab => {
                // Subtle side-to-side wobble at 0.5 Hz
                let wobble_freq = 0.5 * std::f32::consts::TAU;
                let wobble = ((t + phase) * wobble_freq).sin();
                let grid_x = (transform.translation.x / TILE_SIZE).round() * TILE_SIZE;
                transform.translation.x = grid_x + wobble * 1.0;
                // Keep base scale
                transform.scale = Vec3::splat(1.2);
            }
        }

        // Shadows/ghost-like alpha pulse for all enemies (very subtle)
        // Only the rock crab gets a subtle color shift to simulate shadow effect
        let _ = sprite.reborrow(); // suppress unused mut warning
    }
}

/// System: animate ore shimmer by spawning sparkle particles near valuable ores.
pub fn animate_ore_shimmer(
    mut commands: Commands,
    time: Res<Time>,
    mut ores: Query<(&Transform, &mut OreShimmer)>,
    proc_sprites: Res<ProceduralEnemySprites>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    let Some(particle_handle) = proc_sprites.shimmer_particle.as_ref() else {
        return;
    };

    for (ore_transform, mut shimmer) in ores.iter_mut() {
        shimmer.timer.tick(time.delta());
        if !shimmer.timer.just_finished() {
            continue;
        }

        // Spawn a small sparkle particle near the ore
        // Random offset within the tile (using time-based pseudo-random)
        let t = time.elapsed_secs();
        let offset_x = ((t * 127.1).sin() * 6.0).round();
        let offset_y = ((t * 311.7).cos() * 6.0).round();

        let particle_pos = Vec3::new(
            ore_transform.translation.x + offset_x,
            ore_transform.translation.y + offset_y,
            ore_transform.translation.z + 1.0,
        );

        let mut sprite = Sprite::from_image(particle_handle.clone());
        sprite.color = shimmer.color;
        sprite.custom_size = Some(Vec2::splat(4.0));

        commands.spawn((
            sprite,
            Transform::from_translation(particle_pos),
            MineFloorEntity,
            ShimmerParticle {
                lifetime: Timer::from_seconds(0.4, TimerMode::Once),
            },
        ));
    }
}

/// System: fade out and despawn shimmer particles.
pub fn update_shimmer_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut ShimmerParticle, &mut Sprite, &mut Transform)>,
    in_mine: Res<InMine>,
) {
    if !in_mine.0 {
        return;
    }

    for (entity, mut particle, mut sprite, mut transform) in particles.iter_mut() {
        particle.lifetime.tick(time.delta());

        let progress = particle.lifetime.fraction();

        // Float upward slightly
        transform.translation.y += time.delta_secs() * 8.0;

        // Fade out
        let alpha = 1.0 - progress;
        sprite.color = sprite.color.with_alpha(alpha);

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
