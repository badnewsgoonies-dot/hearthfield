//! Visual synchronisation systems — keep sprite appearances in sync with game state.
//!
//! When `FarmingAtlases` are loaded, soil and crop sprites are rendered using
//! texture atlas slices from `tilled_dirt.png` and `plants.png` respectively.
//! If the atlases are not yet available (e.g. first frame before OnEnter fires),
//! coloured placeholder sprites are used as a fallback.

use bevy::prelude::*;
use crate::shared::*;
use super::{
    FarmEntities, FarmingAtlases, SoilTileEntity, CropTileEntity,
    crop_stage_color,
    soil::soil_color,
};

// ─────────────────────────────────────────────────────────────────────────────
// Atlas index helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Map SoilState to an atlas index in tilled_dirt.png (11 cols × 7 rows).
///
/// Index 5  — plain tilled dirt tile (row 0, col 5)
/// Index 16 — darker / wetter dirt tile (row 1, col 5)
fn soil_atlas_index(state: SoilState) -> usize {
    match state {
        SoilState::Untilled => 0,  // shouldn't normally be rendered
        SoilState::Tilled   => 5,
        SoilState::Watered  => 16,
    }
}

/// Map a crop growth stage to a plants.png atlas index (row 0: indices 0-5).
///
/// Uses the formula:
///   `let atlas_idx = (stage * 5 / total_stages.max(1)).min(5)`
/// so every crop maps smoothly onto the 6 available growth frames regardless
/// of how many growth days are defined.
fn crop_atlas_index(stage: u8, total_stages: u8) -> usize {
    let total = total_stages.max(1) as usize;
    ((stage as usize * 5) / total).min(5)
}

// ─────────────────────────────────────────────────────────────────────────────
// Soil sprite sync
// ─────────────────────────────────────────────────────────────────────────────

/// Update the appearance of every soil sprite to match the current SoilState in
/// FarmState.  Also spawns missing entities and despawns stale ones.
///
/// When `FarmingAtlases` are loaded, newly spawned soil entities use a texture
/// atlas slice from `tilled_dirt.png`.  For sprites that were already spawned
/// (either with an atlas or as a colour placeholder) the atlas index / colour is
/// updated in-place.
pub fn sync_soil_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    atlases: Res<FarmingAtlases>,
    mut soil_query: Query<(&SoilTileEntity, &mut Sprite)>,
) {
    // ── Update existing entities ──────────────────────────────────────────────
    for (tile, mut sprite) in soil_query.iter_mut() {
        let pos = (tile.grid_x, tile.grid_y);
        if let Some(&state) = farm_state.soil.get(&pos) {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Atlas sprite: update the slice index.
                atlas.index = soil_atlas_index(state);
                // Apply color tint: watered soil gets a dark brown multiply,
                // tilled soil stays white (no tint).
                sprite.color = match state {
                    SoilState::Watered => Color::srgb(0.6, 0.5, 0.4),
                    SoilState::Tilled  => Color::WHITE,
                    SoilState::Untilled => Color::WHITE,
                };
            } else {
                // Fallback colour sprite (spawned before atlases were ready).
                sprite.color = soil_color(state);
            }
        }
    }

    // ── Spawn missing entities ────────────────────────────────────────────────
    let missing: Vec<((i32, i32), SoilState)> = farm_state
        .soil
        .iter()
        .filter(|(&pos, _)| !farm_entities.soil_entities.contains_key(&pos))
        .map(|(&pos, &state)| (pos, state))
        .collect();

    for (pos, state) in missing {
        // Soil overlays are area fills — use corner-origin to match ground tiles
        let translation = Vec3::new(pos.0 as f32 * TILE_SIZE, pos.1 as f32 * TILE_SIZE, Z_FARM_OVERLAY);

        let entity = if atlases.loaded {
            // Preferred path: use a texture atlas sprite.
            commands.spawn((
                Sprite::from_atlas_image(
                    atlases.dirt_image.clone(),
                    TextureAtlas {
                        layout: atlases.dirt_layout.clone(),
                        index: soil_atlas_index(state),
                    },
                ),
                Transform::from_translation(translation),
                SoilTileEntity { grid_x: pos.0, grid_y: pos.1 },
                SoilTile { state, grid_x: pos.0, grid_y: pos.1 },
            )).id()
        } else {
            // Fallback path: coloured rectangle until atlases are ready.
            commands.spawn((
                Sprite {
                    color: soil_color(state),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_translation(translation),
                SoilTileEntity { grid_x: pos.0, grid_y: pos.1 },
                SoilTile { state, grid_x: pos.0, grid_y: pos.1 },
            )).id()
        };

        farm_entities.soil_entities.insert(pos, entity);
    }

    // ── Despawn stale entities ────────────────────────────────────────────────
    let stale: Vec<(i32, i32)> = farm_entities
        .soil_entities
        .keys()
        .filter(|pos| !farm_state.soil.contains_key(pos))
        .cloned()
        .collect();

    for pos in stale {
        if let Some(entity) = farm_entities.soil_entities.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Crop sprite sync
// ─────────────────────────────────────────────────────────────────────────────

/// Update crop sprite appearances to reflect current stage / wilting / dead state.
///
/// When `FarmingAtlases` are loaded, newly spawned crop entities use a texture
/// atlas slice from `plants.png`.  Dead crops always use a brown-tinted colour
/// sprite regardless of atlas availability.  For existing sprites with an atlas,
/// the slice index is updated based on `current_stage`.
pub fn sync_crop_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    crop_registry: Res<CropRegistry>,
    atlases: Res<FarmingAtlases>,
    mut crop_query: Query<(&CropTileEntity, &mut Sprite, &mut CropTile)>,
) {
    // ── Update existing entities ──────────────────────────────────────────────
    for (tile, mut sprite, mut crop_component) in crop_query.iter_mut() {
        let pos = (tile.grid_x, tile.grid_y);
        if let Some(crop) = farm_state.crops.get(&pos) {
            // Sync the component data.
            *crop_component = crop.clone();

            let total_stages = crop_registry
                .crops
                .get(&crop.crop_id)
                .map(|d| d.growth_days.len() as u8)
                .unwrap_or(4);

            if crop.dead {
                // Dead crops: dark brown tint regardless of atlas availability.
                // Remove any atlas reference so the colour shows through.
                sprite.texture_atlas = None;
                sprite.color = Color::srgb(0.4, 0.3, 0.2);
                sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 0.8));
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                // Atlas sprite: update slice index for current stage.
                atlas.index = crop_atlas_index(crop.current_stage, total_stages);
                // Apply dehydration tint on top of the atlas image.
                // Freshly watered or healthy crops get no tint (WHITE).
                sprite.color = if crop.days_without_water >= 2 {
                    Color::srgb(0.85, 0.70, 0.30) // severely dehydrated — deep yellow
                } else if crop.days_without_water >= 1 {
                    Color::srgb(0.90, 0.85, 0.50) // mildly dehydrated — light yellow
                } else {
                    Color::WHITE // healthy / watered today
                };
            } else {
                // Fallback: colour placeholder, update tint.
                let color = if crop.days_without_water >= 2 {
                    Color::srgb(0.85, 0.70, 0.30) // severely dehydrated
                } else if crop.days_without_water >= 1 {
                    Color::srgb(0.90, 0.85, 0.50) // mildly dehydrated
                } else {
                    crop_stage_color(crop.current_stage, total_stages, crop.dead)
                };
                sprite.color = color;
            }

            // Scale: slightly larger when mature.
            let is_mature = crop_registry
                .crops
                .get(&crop.crop_id)
                .map(|d| crop.current_stage >= d.growth_days.len() as u8)
                .unwrap_or(false);

            sprite.custom_size = Some(Vec2::splat(if is_mature {
                TILE_SIZE * 0.95
            } else {
                TILE_SIZE * 0.8
            }));
        }
    }

    // ── Spawn missing entities ────────────────────────────────────────────────
    let missing: Vec<((i32, i32), CropTile)> = farm_state
        .crops
        .iter()
        .filter(|(&pos, _)| !farm_entities.crop_entities.contains_key(&pos))
        .map(|(&pos, crop)| (pos, crop.clone()))
        .collect();

    for (pos, crop) in missing {
        let total_stages = crop_registry
            .crops
            .get(&crop.crop_id)
            .map(|d| d.growth_days.len() as u8)
            .unwrap_or(4);

        let translation = grid_to_world_center(pos.0, pos.1).extend(Z_FARM_OVERLAY + 1.0);

        let entity = if atlases.loaded && !crop.dead {
            // Preferred path: texture atlas sprite.
            let atlas_index = crop_atlas_index(crop.current_stage, total_stages);
            commands.spawn((
                Sprite::from_atlas_image(
                    atlases.plants_image.clone(),
                    TextureAtlas {
                        layout: atlases.plants_layout.clone(),
                        index: atlas_index,
                    },
                ),
                Transform::from_translation(translation),
                CropTileEntity { grid_x: pos.0, grid_y: pos.1 },
                crop,
            )).id()
        } else {
            // Fallback: coloured rectangle (also used for dead crops).
            let color = if crop.dead {
                Color::srgb(0.4, 0.3, 0.2) // dark brown — withered/dead
            } else if crop.days_without_water >= 2 {
                Color::srgb(0.85, 0.70, 0.30) // severely dehydrated
            } else if crop.days_without_water >= 1 {
                Color::srgb(0.90, 0.85, 0.50) // mildly dehydrated
            } else {
                crop_stage_color(crop.current_stage, total_stages, crop.dead)
            };
            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                    ..default()
                },
                Transform::from_translation(translation),
                CropTileEntity { grid_x: pos.0, grid_y: pos.1 },
                crop,
            )).id()
        };

        farm_entities.crop_entities.insert(pos, entity);
    }

    // ── Despawn stale entities ────────────────────────────────────────────────
    let stale: Vec<(i32, i32)> = farm_entities
        .crop_entities
        .keys()
        .filter(|pos| !farm_state.crops.contains_key(pos))
        .cloned()
        .collect();

    for pos in stale {
        if let Some(entity) = farm_entities.crop_entities.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}
