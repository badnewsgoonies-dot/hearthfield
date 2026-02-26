//! Visual synchronisation systems — keep sprite colours in sync with game state.

use bevy::prelude::*;
use crate::shared::*;
use super::{
    FarmEntities, SoilTileEntity, CropTileEntity,
    grid_to_world, crop_stage_color,
    soil::soil_color,
};

// ─────────────────────────────────────────────────────────────────────────────
// Soil sprite sync
// ─────────────────────────────────────────────────────────────────────────────

/// Update the colour of every soil sprite to match the current SoilState in
/// FarmState.  Also spawns missing entities and despawns stale ones.
pub fn sync_soil_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    mut soil_query: Query<(&SoilTileEntity, &mut Sprite)>,
) {
    // Update existing entities.
    for (tile, mut sprite) in soil_query.iter_mut() {
        let pos = (tile.grid_x, tile.grid_y);
        if let Some(&state) = farm_state.soil.get(&pos) {
            sprite.color = soil_color(state);
        }
    }

    // Spawn entities for soil tiles that don't have an entity yet.
    let missing: Vec<((i32, i32), SoilState)> = farm_state
        .soil
        .iter()
        .filter(|(&pos, _)| !farm_entities.soil_entities.contains_key(&pos))
        .map(|(&pos, &state)| (pos, state))
        .collect();

    for (pos, state) in missing {
        let color = soil_color(state);
        let translation = grid_to_world(pos.0, pos.1);
        let entity = commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform::from_translation(translation),
            SoilTileEntity { grid_x: pos.0, grid_y: pos.1 },
            SoilTile { state, grid_x: pos.0, grid_y: pos.1 },
        )).id();
        farm_entities.soil_entities.insert(pos, entity);
    }

    // Despawn soil entities whose tiles were removed from FarmState.
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

/// Update crop sprite colours to reflect current stage / wilting / dead state.
pub fn sync_crop_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    crop_registry: Res<CropRegistry>,
    mut crop_query: Query<(&CropTileEntity, &mut Sprite, &mut CropTile)>,
) {
    // Update colours and sync CropTile component data for existing entities.
    for (tile, mut sprite, mut crop_component) in crop_query.iter_mut() {
        let pos = (tile.grid_x, tile.grid_y);
        if let Some(crop) = farm_state.crops.get(&pos) {
            // Sync the component.
            *crop_component = crop.clone();

            let total_stages = crop_registry
                .crops
                .get(&crop.crop_id)
                .map(|d| d.growth_days.len() as u8)
                .unwrap_or(4);

            let color = if crop.dead {
                Color::srgb(0.35, 0.28, 0.20)
            } else if crop.days_without_water >= 2 {
                // Wilting — desaturated yellowish green.
                Color::srgb(0.65, 0.62, 0.25)
            } else {
                crop_stage_color(crop.current_stage, total_stages, crop.dead)
            };

            sprite.color = color;

            // Scale up slightly when mature to make it more visible.
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

    // Spawn entities for crops that don't have an entity yet.
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

        let color = crop_stage_color(crop.current_stage, total_stages, crop.dead);
        let translation = grid_to_world(pos.0, pos.1).with_z(2.0);

        let entity = commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                ..default()
            },
            Transform::from_translation(translation),
            CropTileEntity { grid_x: pos.0, grid_y: pos.1 },
            crop,
        )).id();

        farm_entities.crop_entities.insert(pos, entity);
    }

    // Despawn crop entities whose crops were removed from FarmState.
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
