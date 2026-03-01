use bevy::prelude::*;
use crate::shared::*;
use crate::world::TransitionZone;
use super::CollisionMap;

// Default energy restored by an edible item when no registry entry is found.
const DEFAULT_FOOD_ENERGY: f32 = 20.0;

// ═══════════════════════════════════════════════════════════════════════════
// Map Transition Detection
// ═══════════════════════════════════════════════════════════════════════════

/// Map edge boundaries and target transitions.
/// In a full implementation the world domain would provide these via a
/// resource; here we define default map sizes per MapId so the player
/// can trigger transitions by walking to the edge.
fn map_bounds(map: &MapId) -> (i32, i32, i32, i32) {
    // (min_x, max_x, min_y, max_y) — must match generate_*() in world/maps.rs
    match map {
        MapId::Farm => (0, 31, 0, 23),          // 32×24
        MapId::Town => (0, 27, 0, 21),          // 28×22
        MapId::Beach => (0, 19, 0, 13),         // 20×14
        MapId::Forest => (0, 21, 0, 17),        // 22×18
        MapId::MineEntrance => (0, 13, 0, 11),  // 14×12
        MapId::Mine => (0, 23, 0, 23),          // 24×24
        MapId::PlayerHouse => (0, 15, 0, 15),   // 16×16
        MapId::GeneralStore => (0, 11, 0, 11),  // 12×12
        MapId::AnimalShop => (0, 11, 0, 11),    // 12×12
        MapId::Blacksmith => (0, 11, 0, 11),    // 12×12
    }
}

/// Determine which map the player should transition to when they walk
/// off a given edge. Returns `None` if no transition applies.
fn edge_transition(map: &MapId, gx: i32, gy: i32) -> Option<(MapId, i32, i32)> {
    let (min_x, max_x, min_y, max_y) = map_bounds(map);

    // Farm exits
    if *map == MapId::Farm {
        // South edge → Town
        if gy <= min_y {
            return Some((MapId::Town, gx.clamp(0, 47), 46));
        }
        // East edge → Forest
        if gx >= max_x {
            return Some((MapId::Forest, 1, gy.clamp(0, 39)));
        }
        // North edge → nothing (mountain boundary)
        // West edge → Beach
        if gx <= min_x {
            return Some((MapId::Beach, 30, gy.clamp(0, 31)));
        }
    }

    // Town exits
    if *map == MapId::Town {
        // North edge → Farm
        if gy >= max_y {
            return Some((MapId::Farm, gx.clamp(0, 63), 1));
        }
        // South edge → Beach
        if gy <= min_y {
            return Some((MapId::Beach, gx.clamp(0, 31), 30));
        }
        // East edge → Forest
        if gx >= max_x {
            return Some((MapId::Forest, 1, gy.clamp(0, 39)));
        }
    }

    // Beach exits
    if *map == MapId::Beach {
        // North edge → Town
        if gy >= max_y {
            return Some((MapId::Town, gx.clamp(0, 47), 1));
        }
        // East edge → Farm
        if gx >= max_x {
            return Some((MapId::Farm, 1, gy.clamp(0, 63)));
        }
    }

    // Forest exits
    if *map == MapId::Forest {
        // West edge → Farm
        if gx <= min_x {
            return Some((MapId::Farm, 62, gy.clamp(0, 63)));
        }
        // North edge → MineEntrance
        if gy >= max_y {
            return Some((MapId::MineEntrance, 12, 1));
        }
    }

    // MineEntrance exits
    if *map == MapId::MineEntrance {
        // South edge → Forest
        if gy <= min_y {
            return Some((MapId::Forest, 20, 38));
        }
    }

    // Interior rooms — exit through south edge → appropriate outdoor map
    if *map == MapId::PlayerHouse && gy <= min_y {
        return Some((MapId::Farm, 10, 9));
    }
    if *map == MapId::GeneralStore && gy <= min_y {
        return Some((MapId::Town, 24, 20));
    }
    if *map == MapId::AnimalShop && gy <= min_y {
        return Some((MapId::Town, 10, 20));
    }
    if *map == MapId::Blacksmith && gy <= min_y {
        return Some((MapId::Town, 38, 20));
    }

    None
}

/// Check whether the player has reached a map edge and send a
/// `MapTransitionEvent` if so.
pub fn map_transition_check(
    player_state: Res<PlayerState>,
    query: Query<&GridPosition, With<Player>>,
    mut map_events: EventWriter<MapTransitionEvent>,
) {
    let Ok(grid_pos) = query.get_single() else {
        return;
    };

    if let Some((to_map, to_x, to_y)) =
        edge_transition(&player_state.current_map, grid_pos.x, grid_pos.y)
    {
        map_events.send(MapTransitionEvent {
            to_map,
            to_x,
            to_y,
        });
    }
}

/// Handle incoming `MapTransitionEvent` — reposition player and update
/// `PlayerState.current_map`. The world domain handles loading/despawning
/// tiles; we only move the player.
pub fn handle_map_transition(
    mut events: EventReader<MapTransitionEvent>,
    mut player_state: ResMut<PlayerState>,
    mut collision_map: ResMut<CollisionMap>,
    mut camera_snap: ResMut<super::CameraSnap>,
    mut query: Query<(&mut LogicalPosition, &mut GridPosition), With<Player>>,
) {
    // Process only the most recent transition (in case multiple fire).
    let Some(ev) = events.read().last() else {
        return;
    };

    let Ok((mut logical_pos, mut grid_pos)) = query.get_single_mut() else {
        return;
    };

    // Update current map.
    player_state.current_map = ev.to_map;

    // Reposition player to the target tile.
    let wc = grid_to_world_center(ev.to_x, ev.to_y);
    logical_pos.0.x = wc.x;
    logical_pos.0.y = wc.y;
    grid_pos.x = ev.to_x;
    grid_pos.y = ev.to_y;

    // Tell camera to snap instantly instead of lerping.
    camera_snap.frames_remaining = 3;

    // Invalidate the collision map — the world domain will re-populate it
    // for the new map.
    collision_map.initialised = false;
    collision_map.solid_tiles.clear();

    // Update bounds for the new map.
    let (min_x, max_x, min_y, max_y) = map_bounds(&ev.to_map);
    collision_map.bounds = (min_x, max_x, min_y, max_y);
    collision_map.initialised = true;
}

// ═══════════════════════════════════════════════════════════════════════════
// Item Pickup
// ═══════════════════════════════════════════════════════════════════════════

/// Check for interactable / pickup items on the tile the player is
/// standing on or the tile they face when pressing F.
pub fn item_pickup_check(
    player_input: Res<PlayerInput>,
    query: Query<(&GridPosition, &PlayerMovement), With<Player>>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    farm_state: Res<FarmState>,
    player_state: Res<PlayerState>,
    input_blocks: Res<InputBlocks>,
    interaction_claimed: Res<InteractionClaimed>,
) {
    if input_blocks.is_blocked() {
        return;
    }

    // Manual interaction pickup on F key
    if !player_input.interact {
        return;
    }

    if interaction_claimed.0 {
        return;
    }

    let Ok((grid_pos, movement)) = query.get_single() else {
        return;
    };

    let (dx, dy) = super::facing_offset(&movement.facing);
    let target_x = grid_pos.x + dx;
    let target_y = grid_pos.y + dy;

    // Check if there's a harvestable crop at the target tile (on the farm).
    if player_state.current_map == MapId::Farm {
        if let Some(crop) = farm_state.crops.get(&(target_x, target_y)) {
            // If the crop is mature (not dead), allow pickup.
            // The farming domain handles the actual crop removal;
            // we just signal intent via an event.
            if !crop.dead {
                // We send an ItemPickupEvent with the crop_id.
                // The farming domain will handle whether it's actually
                // harvestable (mature) and will produce the harvest_id.
                pickup_events.send(ItemPickupEvent {
                    item_id: crop.crop_id.clone(),
                    quantity: 1,
                });
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Item Pickup → Inventory
// ═══════════════════════════════════════════════════════════════════════════

/// Reads ItemPickupEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn add_items_to_inventory(
    mut pickup_events: EventReader<ItemPickupEvent>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.item_id)
            .map(|def| def.stack_size)
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.item_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(PlaySfxEvent {
                sfx_id: "item_pickup".to_string(),
            });
            info!(
                "[Player] Picked up {} × '{}'",
                ev.quantity, ev.item_id
            );
        } else {
            info!(
                "[Player] Inventory full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.item_id, remaining
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Day End Handling
// ═══════════════════════════════════════════════════════════════════════════

/// When a day ends (player sleeps), restore stamina to maximum and
/// reposition the player to their bed in the farmhouse.
/// Sends a MapTransitionEvent so the world domain loads the PlayerHouse map.
pub fn handle_day_end(
    mut events: EventReader<DayEndEvent>,
    mut player_state: ResMut<PlayerState>,
    mut query: Query<(&mut LogicalPosition, &mut GridPosition), With<Player>>,
    mut map_events: EventWriter<MapTransitionEvent>,
) {
    for _ev in events.read() {
        // Restore stamina fully.
        player_state.stamina = player_state.max_stamina;

        // Restore health fully.
        player_state.health = player_state.max_health;

        let bed_gx = 5;
        let bed_gy = 8;

        // Send MapTransitionEvent so the world domain loads PlayerHouse tiles.
        if player_state.current_map != MapId::PlayerHouse {
            map_events.send(MapTransitionEvent {
                to_map: MapId::PlayerHouse,
                to_x: bed_gx,
                to_y: bed_gy,
            });
        }

        // Move player back to farmhouse bed position.
        player_state.current_map = MapId::PlayerHouse;

        if let Ok((mut logical_pos, mut grid_pos)) = query.get_single_mut() {
            let wc = grid_to_world_center(bed_gx, bed_gy);
            logical_pos.0.x = wc.x;
            logical_pos.0.y = wc.y;
            grid_pos.x = bed_gx;
            grid_pos.y = bed_gy;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Stamina Recovery
// ═══════════════════════════════════════════════════════════════════════════

/// Reads `StaminaRestoreEvent` and applies stamina recovery to the player,
/// capped at `max_stamina`.
pub fn handle_stamina_restore(
    mut events: EventReader<StaminaRestoreEvent>,
    mut player_state: ResMut<PlayerState>,
) {
    for ev in events.read() {
        let before = player_state.stamina;
        player_state.stamina =
            (player_state.stamina + ev.amount).min(player_state.max_stamina);
        let gained = player_state.stamina - before;
        info!(
            "[Player] Stamina restored {:.1} (source: {:?}) — now {:.1}/{:.1}",
            gained, ev.source, player_state.stamina, player_state.max_stamina
        );
    }
}

/// Reads `ConsumeItemEvent`, looks the item up in the `ItemRegistry`, removes
/// it from inventory, and fires a `StaminaRestoreEvent` for the appropriate
/// energy value.
pub fn handle_consume_item(
    mut events: EventReader<ConsumeItemEvent>,
    mut inventory: ResMut<Inventory>,
    item_registry: Res<ItemRegistry>,
    mut stamina_restore_events: EventWriter<StaminaRestoreEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for ev in events.read() {
        // Look up item definition to get the energy restore value.
        let energy_value = if let Some(def) = item_registry.get(&ev.item_id) {
            if def.edible {
                def.energy_restore
            } else {
                // Item exists but is not edible — skip.
                info!("[Player] Tried to consume non-edible item '{}'", ev.item_id);
                continue;
            }
        } else {
            // Item not in registry; apply a default for any unknown food-like item.
            DEFAULT_FOOD_ENERGY
        };

        // Remove one from inventory.
        let removed = inventory.try_remove(&ev.item_id, 1);
        if removed == 0 {
            info!("[Player] Cannot consume '{}' — not in inventory", ev.item_id);
            continue;
        }

        // Send stamina restore event.
        stamina_restore_events.send(StaminaRestoreEvent {
            amount: energy_value,
            source: StaminaSource::Food(ev.item_id.clone()),
        });

        // Play eat sound effect.
        sfx_events.send(PlaySfxEvent {
            sfx_id: "eat".to_string(),
        });

        info!(
            "[Player] Consumed '{}' — restoring {:.1} stamina",
            ev.item_id, energy_value
        );
    }
}

/// Checks each frame whether stamina has reached zero at or past midnight
/// (hour >= 24). If so, the player passes out and a `DayEndEvent` is sent.
pub fn check_stamina_consequences(
    player_state: Res<PlayerState>,
    calendar: Res<Calendar>,
    mut day_end_events: EventWriter<DayEndEvent>,
    mut has_passed_out: Local<bool>,
) {
    if player_state.stamina <= 0.0 && calendar.hour >= 24 {
        // Only fire once per exhaustion episode; reset when stamina recovers.
        if !*has_passed_out {
            *has_passed_out = true;
            warn!(
                "[Player] Passed out from exhaustion at hour {}! Ending the day.",
                calendar.hour
            );
            day_end_events.send(DayEndEvent {
                day: calendar.day,
                season: calendar.season,
                year: calendar.year,
            });
        }
    } else if player_state.stamina > 0.0 {
        // Reset the flag once stamina is restored (e.g. after sleep).
        *has_passed_out = false;
    }
}
