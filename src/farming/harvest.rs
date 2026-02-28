//! Harvest system — player interacts with mature crops.

use bevy::prelude::*;
use rand::Rng;
use crate::shared::*;
use super::{FarmEntities, HarvestAttemptEvent, CropTileEntity};

// ─────────────────────────────────────────────────────────────────────────────
// Detect harvest input (Space bar)
// ─────────────────────────────────────────────────────────────────────────────

/// Detect Space bar press and emit HarvestAttemptEvent at the tile in front of
/// the player.  Because we cannot import the player domain, we approximate by
/// reading from a query on the Player component (defined in shared).
pub fn detect_harvest_input(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    player_query: Query<&LogicalPosition, With<Player>>,
    player_state: Res<PlayerState>,
    mut harvest_events: EventWriter<HarvestAttemptEvent>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    if !player_input.tool_use {
        return;
    }

    if player_state.current_map != MapId::Farm {
        return;
    }

    let Ok(logical_pos) = player_query.get_single() else {
        return;
    };

    // Convert world position to grid.
    let grid_x = (logical_pos.0.x / TILE_SIZE).round() as i32;
    let grid_y = (logical_pos.0.y / TILE_SIZE).round() as i32;

    harvest_events.send(HarvestAttemptEvent { grid_x, grid_y });
}

// ─────────────────────────────────────────────────────────────────────────────
// Process harvest attempt
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_harvest_attempt(
    mut harvest_events: EventReader<HarvestAttemptEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
    mut item_pickup_events: EventWriter<ItemPickupEvent>,
    mut crop_harvested_events: EventWriter<CropHarvestedEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    crop_registry: Res<CropRegistry>,
) {
    for event in harvest_events.read() {
        let pos = (event.grid_x, event.grid_y);

        // Also check adjacent tiles (the player might be slightly off).
        let candidates = [
            pos,
            (pos.0 + 1, pos.1),
            (pos.0 - 1, pos.1),
            (pos.0, pos.1 + 1),
            (pos.0, pos.1 - 1),
        ];

        for target_pos in candidates {
            if try_harvest_at(
                target_pos,
                &mut farm_state,
                &mut farm_entities,
                &mut commands,
                &mut item_pickup_events,
                &mut crop_harvested_events,
                &crop_registry,
            ) {
                sfx_events.send(PlaySfxEvent { sfx_id: "harvest".to_string() });
                break; // Only harvest one crop per input.
            }
        }
    }
}

/// Try to harvest the crop at `pos`. Returns true if a harvest occurred.
fn try_harvest_at(
    pos: (i32, i32),
    farm_state: &mut FarmState,
    farm_entities: &mut FarmEntities,
    commands: &mut Commands,
    item_pickup_events: &mut EventWriter<ItemPickupEvent>,
    crop_harvested_events: &mut EventWriter<CropHarvestedEvent>,
    crop_registry: &CropRegistry,
) -> bool {
    let Some(crop) = farm_state.crops.get(&pos) else {
        return false;
    };

    if crop.dead {
        // Remove dead crop.
        despawn_crop(pos, farm_state, farm_entities, commands);
        return true;
    }

    let Some(def) = crop_registry.crops.get(&crop.crop_id).cloned() else {
        return false;
    };

    // A crop is mature when current_stage == growth_days.len() (all stages done).
    let mature_stage = def.growth_days.len() as u8;
    let crop_stage = crop.current_stage;

    if crop_stage < mature_stage {
        return false; // Not ready.
    }

    // Harvest!
    let quality = roll_harvest_quality();
    let quantity: u8 = if def.regrows { 1 } else { 1 }; // base quantity always 1

    item_pickup_events.send(ItemPickupEvent {
        item_id: def.harvest_id.clone(),
        quantity,
    });

    crop_harvested_events.send(CropHarvestedEvent {
        crop_id: def.id.clone(),
        harvest_id: def.harvest_id.clone(),
        quantity,
        x: pos.0,
        y: pos.1,
        quality: Some(quality),
    });

    if def.regrows {
        // Reset to regrow stage.  The crop goes back to the last stage and
        // waits regrow_days to produce again.
        let regrow_stage = (def.growth_days.len() as u8).saturating_sub(1);
        if let Some(crop_mut) = farm_state.crops.get_mut(&pos) {
            crop_mut.current_stage = regrow_stage;
            crop_mut.days_in_stage = 0;
            crop_mut.dead = false;
        }

        // Update the sprite to show regrow stage.
        if let Some(crop_ref) = farm_state.crops.get(&pos) {
            update_crop_entity_color(pos, crop_ref, &def, farm_entities, commands);
        }
    } else {
        // Remove the crop entirely.
        despawn_crop(pos, farm_state, farm_entities, commands);

        // Also reset soil to tilled (harvest doesn't remove the tilled state).
        if let Some(state) = farm_state.soil.get_mut(&pos) {
            if *state == SoilState::Watered {
                *state = SoilState::Tilled;
            }
        }
    }

    true
}

/// Remove a crop from FarmState and despawn its entity.
pub fn despawn_crop(
    pos: (i32, i32),
    farm_state: &mut FarmState,
    farm_entities: &mut FarmEntities,
    commands: &mut Commands,
) {
    farm_state.crops.remove(&pos);

    if let Some(entity) = farm_entities.crop_entities.remove(&pos) {
        commands.entity(entity).despawn();
    }
}

/// Roll the quality of a harvested crop.
/// Distribution: 74% Normal, 20% Silver, 5% Gold, 1% Iridium.
fn roll_harvest_quality() -> ItemQuality {
    let roll: f32 = rand::thread_rng().gen_range(0.0..1.0);
    if roll < 0.01 {
        ItemQuality::Iridium
    } else if roll < 0.06 {
        ItemQuality::Gold
    } else if roll < 0.26 {
        ItemQuality::Silver
    } else {
        ItemQuality::Normal
    }
}

/// Update colour of an existing crop entity in-place.
fn update_crop_entity_color(
    pos: (i32, i32),
    crop: &CropTile,
    def: &CropDef,
    farm_entities: &mut FarmEntities,
    commands: &mut Commands,
) {
    use super::crop_stage_color;

    let total_stages = def.growth_days.len() as u8;
    let color = crop_stage_color(crop.current_stage, total_stages, crop.dead);

    if let Some(&entity) = farm_entities.crop_entities.get(&pos) {
        // Insert updated Sprite; Bevy replaces it in-place.
        commands.entity(entity).insert(Sprite {
            color,
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
            ..default()
        });
    } else {
        // Entity doesn't exist yet — spawn it.
        // We need a CropTileEntity and CropTile; both are cloneable.
        let translation = super::grid_to_world(pos.0, pos.1).with_z(Z_FARM_OVERLAY + 1.0);
        let entity = commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                ..default()
            },
            Transform::from_translation(translation),
            CropTileEntity { grid_x: pos.0, grid_y: pos.1 },
            crop.clone(),
        )).id();
        farm_entities.crop_entities.insert(pos, entity);
    }
}
