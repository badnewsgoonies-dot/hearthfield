use bevy::prelude::*;
use crate::shared::*;
use super::{grid_to_world, CollisionMap};

// ═══════════════════════════════════════════════════════════════════════════
// Map Transition Detection
// ═══════════════════════════════════════════════════════════════════════════

/// Map edge boundaries and target transitions.
/// In a full implementation the world domain would provide these via a
/// resource; here we define default map sizes per MapId so the player
/// can trigger transitions by walking to the edge.
fn map_bounds(map: &MapId) -> (i32, i32, i32, i32) {
    match map {
        MapId::Farm => (0, 63, 0, 63),
        MapId::Town => (0, 47, 0, 47),
        MapId::Beach => (0, 31, 0, 31),
        MapId::Forest => (0, 39, 0, 39),
        MapId::MineEntrance => (0, 23, 0, 23),
        MapId::Mine => (0, 23, 0, 23),
        MapId::PlayerHouse => (0, 11, 0, 11),
        MapId::GeneralStore => (0, 11, 0, 11),
        MapId::AnimalShop => (0, 11, 0, 11),
        MapId::Blacksmith => (0, 11, 0, 11),
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
    mut query: Query<(&mut Transform, &mut GridPosition), With<Player>>,
) {
    // Process only the most recent transition (in case multiple fire).
    let Some(ev) = events.read().last() else {
        return;
    };

    let Ok((mut transform, mut grid_pos)) = query.get_single_mut() else {
        return;
    };

    // Update current map.
    player_state.current_map = ev.to_map;

    // Reposition player to the target tile.
    let (wx, wy) = grid_to_world(ev.to_x, ev.to_y);
    transform.translation.x = wx;
    transform.translation.y = wy;
    grid_pos.x = ev.to_x;
    grid_pos.y = ev.to_y;

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
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(&GridPosition, &PlayerMovement), With<Player>>,
    mut pickup_events: EventWriter<ItemPickupEvent>,
    farm_state: Res<FarmState>,
    player_state: Res<PlayerState>,
) {
    // Manual interaction pickup on F key
    if !keyboard.just_pressed(KeyCode::KeyF) {
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
// Day End Handling
// ═══════════════════════════════════════════════════════════════════════════

/// When a day ends (player sleeps), restore stamina to maximum and
/// reposition the player to their bed in the farmhouse.
pub fn handle_day_end(
    mut events: EventReader<DayEndEvent>,
    mut player_state: ResMut<PlayerState>,
    mut query: Query<(&mut Transform, &mut GridPosition), With<Player>>,
) {
    for _ev in events.read() {
        // Restore stamina fully.
        player_state.stamina = player_state.max_stamina;

        // Restore health fully.
        player_state.health = player_state.max_health;

        // Move player back to farmhouse bed position.
        player_state.current_map = MapId::PlayerHouse;

        if let Ok((mut transform, mut grid_pos)) = query.get_single_mut() {
            let bed_gx = 5;
            let bed_gy = 8;
            let (wx, wy) = grid_to_world(bed_gx, bed_gy);
            transform.translation.x = wx;
            transform.translation.y = wy;
            grid_pos.x = bed_gx;
            grid_pos.y = bed_gy;
        }
    }
}
