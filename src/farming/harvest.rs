//! Harvest system — player interacts with mature crops.

use bevy::prelude::*;
use crate::shared::*;
use super::{FarmEntities, HarvestAttemptEvent, CropTileEntity};

// ─────────────────────────────────────────────────────────────────────────────
// Detect harvest input (Space bar)
// ─────────────────────────────────────────────────────────────────────────────

/// Detect Space bar press and emit HarvestAttemptEvent at the tile in front of
/// the player.  Because we cannot import the player domain, we approximate by
/// reading from a query on the Player component (defined in shared).
pub fn detect_harvest_input(
    keys: Res<ButtonInput<KeyCode>>,
    player_query: Query<&Transform, With<Player>>,
    player_state: Res<PlayerState>,
    mut harvest_events: EventWriter<HarvestAttemptEvent>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    if player_state.current_map != MapId::Farm {
        return;
    }

    let Ok(transform) = player_query.get_single() else {
        return;
    };

    // Convert world position to grid.
    let grid_x = (transform.translation.x / TILE_SIZE).round() as i32;
    let grid_y = (transform.translation.y / TILE_SIZE).round() as i32;

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
    let quantity: u8 = 1; // TODO: quality/luck bonuses could multiply this.

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
        quality: None, // TODO(A2): roll quality based on fertilizer/luck
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
        let translation = super::grid_to_world(pos.0, pos.1).with_z(2.0);
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
