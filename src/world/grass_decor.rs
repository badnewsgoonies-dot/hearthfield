//! Grass decoration system: spawn small decorative sprites on grass tiles.
//!
//! Currently DISABLED — the grass_biome.png sprites are from an old art pack
//! and don't match the modern farm terrain. The base terrain already provides
//! 4 grass tile variants for visual variety.
//!
//! To re-enable: replace grass_biome.png with matching art, then uncomment
//! the system registration in mod.rs.

#![allow(dead_code)]

use bevy::prelude::*;

use crate::shared::*;

use super::objects::{ObjectAtlases, WindSway};
use super::{MapTile, WorldMap};

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker for grass decoration sprite entities (despawned with the map).
#[derive(Component, Debug)]
pub struct GrassDecoration;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCE
// ═══════════════════════════════════════════════════════════════════════

/// Tracks whether grass decorations have been spawned for the current map.
#[derive(Resource, Default)]
pub struct GrassDecorState {
    pub spawned_for_map: Option<MapId>,
    pub spawned_for_season: Option<Season>,
}

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Z layer for grass decorations — between ground (0.0) and farm overlay (10.0).
const Z_GRASS_DECOR: f32 = 5.0;

// ═══════════════════════════════════════════════════════════════════════
// POSITIONAL HASH
// ═══════════════════════════════════════════════════════════════════════

/// Deterministic hash of tile position for consistent decoration placement.
/// Returns a value in 0..1000.
fn tile_hash(x: usize, y: usize) -> u32 {
    // Simple but effective hash for visual variety.
    let h = (x as u32)
        .wrapping_mul(2654435761)
        .wrapping_add((y as u32).wrapping_mul(2246822519));
    h % 1000
}

/// Secondary hash for choosing decoration variant.
fn variant_hash(x: usize, y: usize) -> u32 {
    (x as u32)
        .wrapping_mul(1103515245)
        .wrapping_add((y as u32).wrapping_mul(12345))
}

// ═══════════════════════════════════════════════════════════════════════
// ATLAS INDICES
// ═══════════════════════════════════════════════════════════════════════

/// Returns (atlas_index, should_spawn) for a grass decoration at (x,y).
/// `threshold` controls what % of tiles get decorations (lower = more).
fn decoration_for_season(season: Season, x: usize, y: usize) -> Option<usize> {
    let h = tile_hash(x, y);
    let v = variant_hash(x, y);

    // grass_biome.png: 9 cols x 5 rows of 16x16 tiles
    // Row 0 indices 0..8: grass tufts, small flowers, tiny decorations
    //   0: tall grass tuft (green)
    //   1: small bush/grass clump
    //   2: tiny flower (red)
    //   3: small leaf/clover
    //   4: purple flower / weeds
    //   5: grass patch
    //   6: small green plant
    //   7: darker plant
    //   8: another grass variant

    match season {
        Season::Spring => {
            // ~18% of tiles: lots of flowers and small plants
            if h < 180 {
                // Favor flowers in spring
                let choices: &[usize] = &[0, 2, 3, 4, 5, 6];
                Some(choices[(v as usize) % choices.len()])
            } else {
                None
            }
        }
        Season::Summer => {
            // ~15% of tiles: dry tufts, fewer flowers
            if h < 150 {
                let choices: &[usize] = &[0, 1, 5, 7, 8];
                Some(choices[(v as usize) % choices.len()])
            } else {
                None
            }
        }
        Season::Fall => {
            // ~10% of tiles: sparse, mostly just grass tufts
            if h < 100 {
                let choices: &[usize] = &[0, 1, 5];
                Some(choices[(v as usize) % choices.len()])
            } else {
                None
            }
        }
        Season::Winter => {
            // ~8% of tiles: very sparse, bare look
            if h < 80 {
                let choices: &[usize] = &[0, 5];
                Some(choices[(v as usize) % choices.len()])
            } else {
                None
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn grass decorations when a map is loaded.
/// Runs every frame but only does work when the map or season changes.
pub fn spawn_grass_decorations(
    mut commands: Commands,
    world_map: Res<WorldMap>,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    object_atlases: Res<ObjectAtlases>,
    mut state: ResMut<GrassDecorState>,
    existing: Query<Entity, With<GrassDecoration>>,
) {
    let map_id = player_state.current_map;
    let season = calendar.season;

    // Only do work when map or season changes.
    if state.spawned_for_map == Some(map_id) && state.spawned_for_season == Some(season) {
        return;
    }

    // Despawn existing decorations first.
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    // Need the map def to know which tiles are grass.
    let Some(ref map_def) = world_map.map_def else {
        return;
    };

    // Don't spawn decorations on indoor maps.
    if matches!(
        map_id,
        MapId::PlayerHouse
            | MapId::TownHouseWest
            | MapId::TownHouseEast
            | MapId::GeneralStore
            | MapId::AnimalShop
            | MapId::Blacksmith
    ) {
        state.spawned_for_map = Some(map_id);
        state.spawned_for_season = Some(season);
        return;
    }

    if !object_atlases.loaded {
        return;
    }

    // Iterate over all tiles and spawn decorations on eligible grass tiles.
    let width = map_def.width;
    let height = map_def.height;

    for y in 0..height {
        for x in 0..width {
            let tile = map_def.tiles[y * width + x];
            if tile != TileKind::Grass {
                continue;
            }

            // Check if this tile should get a decoration.
            if let Some(atlas_idx) = decoration_for_season(season, x, y) {
                // Offset position slightly from tile center using the variant hash
                // so decorations don't sit perfectly centered on every tile.
                let vh = variant_hash(x, y);
                let offset_x = ((vh % 7) as f32 - 3.0) * 1.5; // -4.5 to +4.5 px
                let offset_y = (((vh / 7) % 7) as f32 - 3.0) * 1.5;

                let wx = x as f32 * TILE_SIZE + TILE_SIZE * 0.5 + offset_x;
                let wy = y as f32 * TILE_SIZE + TILE_SIZE * 0.5 + offset_y;

                let mut sprite = Sprite::from_atlas_image(
                    object_atlases.grass_biome_image.clone(),
                    TextureAtlas {
                        layout: object_atlases.grass_biome_layout.clone(),
                        index: atlas_idx,
                    },
                );
                sprite.custom_size = Some(Vec2::new(12.0, 12.0));

                // Apply seasonal tint to decorations.
                sprite.color = match season {
                    Season::Spring => Color::WHITE,
                    Season::Summer => Color::srgb(1.0, 0.95, 0.85),
                    Season::Fall => Color::srgb(0.9, 0.75, 0.55),
                    Season::Winter => Color::srgb(0.85, 0.90, 1.0),
                };

                // Grass sway: sinusoidal rotation ±4-8° at 0.8-1.2 Hz.
                // Phase offset from positional hash for visual variety.
                let phase_offset = (vh as f32).sin() * std::f32::consts::TAU;
                let sway_speed = 0.8 + (vh % 5) as f32 * 0.1; // 0.8-1.2 Hz
                let sway_amount = 0.07 + (vh % 4) as f32 * 0.007; // ~4-8 degrees in radians

                commands.spawn((
                    sprite,
                    Transform::from_translation(Vec3::new(wx, wy, Z_GRASS_DECOR)),
                    MapTile, // will despawn with the map
                    GrassDecoration,
                    WindSway {
                        offset: phase_offset,
                        speed: sway_speed,
                        amount: sway_amount,
                    },
                ));
            }
        }
    }

    state.spawned_for_map = Some(map_id);
    state.spawned_for_season = Some(season);
}
