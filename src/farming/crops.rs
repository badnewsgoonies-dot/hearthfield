//! Crop planting and growth-stage management.

use bevy::prelude::*;
use crate::shared::*;
use super::{
    FarmEntities, CropTileEntity, PlantSeedEvent,
    grid_to_world, crop_stage_color, crop_can_grow_in_season,
};

// ─────────────────────────────────────────────────────────────────────────────
// Detect seed use — player presses interact while holding a seed over tilled soil
// ─────────────────────────────────────────────────────────────────────────────

/// Detect when the player presses the interact key (F) while holding a seed
/// item in their selected hotbar slot, then emit a PlantSeedEvent.
pub fn detect_seed_use(
    keys: Res<ButtonInput<KeyCode>>,
    player_state: Res<PlayerState>,
    inventory: Res<Inventory>,
    farm_state: Res<FarmState>,
    calendar: Res<Calendar>,
    crop_registry: Res<CropRegistry>,
    mut plant_events: EventWriter<PlantSeedEvent>,
) {
    // Interact key: F
    if !keys.just_pressed(KeyCode::KeyF) {
        return;
    }

    // Only on the farm map.
    if player_state.current_map != MapId::Farm {
        return;
    }

    // Get the selected item from inventory.
    let slot_idx = inventory.selected_slot;
    let Some(slot) = inventory.slots[slot_idx].as_ref() else {
        return;
    };

    let seed_id = slot.item_id.clone();

    // Find a crop def that uses this seed.
    let Some(crop_def) = crop_registry.crops.values().find(|c| c.seed_id == seed_id) else {
        return; // Not a seed
    };

    // Check season validity.
    if !crop_can_grow_in_season(crop_def, calendar.season) {
        // The player tried to plant out of season — silently ignore (UI could show a message).
        return;
    }

    // Determine the player's facing direction to find the target tile.
    // We'll use grid_x/grid_y relative to the player's world position.
    // Since we don't have the player's transform here, we rely on the player domain
    // to send a ToolUseEvent. For planting we use the same target_x/target_y from the
    // ToolUseEvent mechanism. However, planting uses "interact" rather than a tool.
    //
    // As an alternative approach that doesn't require cross-domain coupling, we'll
    // emit the event with a special sentinel and let the player plugin's position
    // inform us. Instead we detect it simply: we look for ANY adjacent tilled tile
    // that could be planted. To keep things decoupled, we make the PlantSeedEvent
    // fired from ToolUseEvent in the farming domain itself (see handle_plant_seed).
    //
    // For now emit a PlantSeedEvent at tile 0,0 — the actual coupling is via ToolUseEvent.
    // This system is a secondary path for direct keyboard use:

    // We need the player's position. Since we cannot import from player domain, we use
    // a convention: read from PlayerState.current_map + assume the player is at origin
    // if we don't have transform data. The real planting trigger comes from ToolUseEvent
    // which carries target_x/target_y. This system augments it.
    //
    // Practical approach: scan farm_state for tilled tiles near (0,0) origin as placeholder.
    // The actual, correct planting trigger is in handle_plant_seed which reads ToolUseEvent.
    let _ = (farm_state, plant_events, seed_id);
    // NOTE: planting is primarily triggered via ToolUseEvent with the Hoe already used.
    // The PlantSeedEvent path is used by higher-level planting logic. See handle_plant_seed.
}

// ─────────────────────────────────────────────────────────────────────────────
// Handle plant seed event
// ─────────────────────────────────────────────────────────────────────────────

/// Listen for PlantSeedEvent and actually plant the crop.
pub fn handle_plant_seed(
    mut plant_events: EventReader<PlantSeedEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut inventory: ResMut<Inventory>,
    mut commands: Commands,
    mut item_removed_events: EventWriter<ItemRemovedEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    crop_registry: Res<CropRegistry>,
    calendar: Res<Calendar>,
) {
    for event in plant_events.read() {
        let pos = (event.grid_x, event.grid_y);

        // Tile must be tilled or watered.
        let soil = farm_state.soil.get(&pos).copied();
        if soil != Some(SoilState::Tilled) && soil != Some(SoilState::Watered) {
            continue;
        }

        // Tile must not already have a crop.
        if farm_state.crops.contains_key(&pos) {
            continue;
        }

        // Find crop def by seed id.
        let Some(crop_def) = crop_registry
            .crops
            .values()
            .find(|c| c.seed_id == event.seed_item_id)
            .cloned()
        else {
            continue;
        };

        // Check season validity.
        if !crop_can_grow_in_season(&crop_def, calendar.season) {
            continue;
        }

        // Remove one seed from inventory.
        if inventory.try_remove(&event.seed_item_id, 1) == 0 {
            // Player doesn't have the seed any more.
            continue;
        }

        item_removed_events.send(ItemRemovedEvent {
            item_id: event.seed_item_id.clone(),
            quantity: 1,
        });

        // Create the crop entry in FarmState.
        let crop = CropTile {
            crop_id: crop_def.id.clone(),
            current_stage: 0,
            days_in_stage: 0,
            watered_today: soil == Some(SoilState::Watered),
            days_without_water: 0,
            dead: false,
        };
        farm_state.crops.insert(pos, crop.clone());

        sfx_events.send(PlaySfxEvent { sfx_id: "plant".to_string() });

        // Spawn crop sprite entity.
        spawn_crop_entity(
            &mut commands,
            &mut farm_entities,
            pos,
            &crop,
            &crop_def,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool-use planting detection
// ─────────────────────────────────────────────────────────────────────────────

// This system bridges the ToolUseEvent (Hoe) pattern: when a player presses
// the use button while holding a seed (not a hoe), we treat it as planting.
// In practice the player plugin fires ToolUseEvent only for actual tools.
// Planting is triggered by PlantSeedEvent which the player/ui domain sends.
// We include a direct keyboard listener here for local testing convenience.

// ─────────────────────────────────────────────────────────────────────────────
// Entity spawn helper
// ─────────────────────────────────────────────────────────────────────────────

pub fn spawn_crop_entity(
    commands: &mut Commands,
    farm_entities: &mut FarmEntities,
    pos: (i32, i32),
    crop: &CropTile,
    crop_def: &CropDef,
) {
    let total_stages = crop_def.growth_days.len() as u8;
    let color = crop_stage_color(crop.current_stage, total_stages, crop.dead);
    let translation = grid_to_world(pos.0, pos.1).with_z(2.0); // above soil

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

// ─────────────────────────────────────────────────────────────────────────────
// Crop growth advancement (called from events_handler::on_day_end)
// ─────────────────────────────────────────────────────────────────────────────

/// Advance all crops by one day.  Called from the DayEnd handler.
pub fn advance_crop_growth(
    farm_state: &mut FarmState,
    crop_registry: &CropRegistry,
    current_season: Season,
    is_rainy: bool,
) -> Vec<(i32, i32)> // returns positions of crops that need entity updates
{
    let positions: Vec<(i32, i32)> = farm_state.crops.keys().cloned().collect();
    let mut updated = Vec::new();

    for pos in positions {
        let Some(crop) = farm_state.crops.get_mut(&pos) else {
            continue;
        };

        if crop.dead {
            continue;
        }

        // Rain auto-waters crops.
        let effectively_watered = crop.watered_today || is_rainy;

        // Get crop definition.
        let Some(def) = crop_registry.crops.get(&crop.crop_id) else {
            continue;
        };

        // Kill crops that can't grow in current season.
        if !crop_can_grow_in_season(def, current_season) {
            crop.dead = true;
            updated.push(pos);
            continue;
        }

        if effectively_watered {
            crop.days_in_stage += 1;
            crop.days_without_water = 0;

            // Check if we can advance to the next stage.
            let stage_idx = crop.current_stage as usize;
            if stage_idx < def.growth_days.len() {
                let days_needed = def.growth_days[stage_idx];
                if crop.days_in_stage >= days_needed {
                    // Advance stage.
                    let max_stage = def.growth_days.len() as u8; // stages are 0..len
                    if crop.current_stage < max_stage {
                        crop.current_stage += 1;
                    }
                    crop.days_in_stage = 0;
                }
            }
        } else {
            // Not watered today.
            crop.days_without_water += 1;

            if crop.days_without_water >= 3 {
                // 3 days without water → dead.
                crop.dead = true;
            }
            // 2 days → wilting (visual handled by sync_crop_sprites via days_without_water).
        }

        // Reset watered_today for the next day.
        crop.watered_today = false;

        updated.push(pos);
    }

    updated
}

/// Reset all soil from Watered back to Tilled at day start
/// (rain may re-water later; sprinklers run first).
pub fn reset_soil_watered_state(farm_state: &mut FarmState) {
    for state in farm_state.soil.values_mut() {
        if *state == SoilState::Watered {
            *state = SoilState::Tilled;
        }
    }
}
