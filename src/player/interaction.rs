use super::CollisionMap;
use crate::shared::*;
use crate::world::map_data::{EdgeTarget, MapRegistry};
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// Map Transition Detection (data-driven via MapRegistry)
// ═══════════════════════════════════════════════════════════════════════════

/// Map edge boundaries derived from `MapRegistry`.
/// Falls back to hardcoded defaults if the map is not in the registry.
fn map_bounds_from_registry(map: &MapId, registry: &MapRegistry) -> (i32, i32, i32, i32) {
    if let Some(data) = registry.maps.get(map) {
        (0, data.width as i32 - 1, 0, data.height as i32 - 1)
    } else {
        map_bounds_hardcoded(map)
    }
}

/// Hardcoded fallback for map bounds (kept for safety).
fn map_bounds_hardcoded(map: &MapId) -> (i32, i32, i32, i32) {
    match map {
        MapId::Farm => (0, 31, 0, 23),
        MapId::Town => (0, 27, 0, 21),
        MapId::Beach => (0, 19, 0, 13),
        MapId::Forest => (0, 21, 0, 17),
        MapId::DeepForest => (0, 29, 0, 27),
        MapId::MineEntrance => (0, 13, 0, 11),
        MapId::Mine => (0, 23, 0, 23),
        MapId::PlayerHouse => (0, 15, 0, 15),
        MapId::TownHouseWest => (0, 11, 0, 11),
        MapId::TownHouseEast => (0, 11, 0, 11),
        MapId::GeneralStore => (0, 11, 0, 11),
        MapId::AnimalShop => (0, 11, 0, 11),
        MapId::Blacksmith => (0, 11, 0, 11),
        MapId::Library => (0, 13, 0, 11),
        MapId::Tavern => (0, 15, 0, 13),
        MapId::CoralIsland => (0, 29, 0, 21),
    }
}

/// Clamp a carried edge coordinate to the interior of the destination map so
/// corner transitions don't strand the player on the receiving border.
fn clamp_to_interior(value: i32, min: i32, max: i32) -> i32 {
    if max - min >= 2 {
        value.clamp(min + 1, max - 1)
    } else {
        value.clamp(min, max)
    }
}

/// Resolve an edge target to concrete (x, y) coordinates.
fn resolve_edge_target(
    target: &EdgeTarget,
    target_map: MapId,
    gx: i32,
    gy: i32,
    registry: &MapRegistry,
) -> (i32, i32) {
    match target {
        EdgeTarget::ClampX(fixed_y) => {
            let (min_x, max_x, _, _) = map_bounds_from_registry(&target_map, registry);
            (clamp_to_interior(gx, min_x, max_x), *fixed_y)
        }
        EdgeTarget::ClampY(fixed_x) => {
            let (_, _, min_y, max_y) = map_bounds_from_registry(&target_map, registry);
            (*fixed_x, clamp_to_interior(gy, min_y, max_y))
        }
        EdgeTarget::Fixed(fx, fy) => (*fx, *fy),
    }
}

/// Data-driven transition check: doors first, then edge boundaries.
/// Falls back to the hardcoded `edge_transition_hardcoded` if the map
/// is not present in the registry.
fn edge_transition_from_registry(
    map: &MapId,
    gx: i32,
    gy: i32,
    registry: &MapRegistry,
) -> Option<(MapId, i32, i32)> {
    let Some(data) = registry.maps.get(map) else {
        return edge_transition_hardcoded(map, gx, gy);
    };

    // ── Door triggers (checked BEFORE edge boundaries) ──
    for door in &data.doors {
        if (door.x_min..=door.x_max).contains(&gx) && gy == door.y {
            return Some((door.to_map, door.to_x, door.to_y));
        }
    }

    // NOTE: MapData.transitions are zone-based triggers kept for
    // documentation / future use.  Actual game transitions are driven
    // entirely by doors (above) and edge boundaries (below), matching
    // the original hardcoded logic.

    // ── Edge boundaries ──
    let (min_x, max_x, min_y, max_y) = (0, data.width as i32 - 1, 0, data.height as i32 - 1);

    if gy >= max_y {
        if let Some((target_map, ref target)) = data.edges.north {
            let (tx, ty) = resolve_edge_target(target, target_map, gx, gy, registry);
            return Some((target_map, tx, ty));
        }
    }
    if gy <= min_y {
        if let Some((target_map, ref target)) = data.edges.south {
            let (tx, ty) = resolve_edge_target(target, target_map, gx, gy, registry);
            return Some((target_map, tx, ty));
        }
    }
    if gx >= max_x {
        if let Some((target_map, ref target)) = data.edges.east {
            let (tx, ty) = resolve_edge_target(target, target_map, gx, gy, registry);
            return Some((target_map, tx, ty));
        }
    }
    if gx <= min_x {
        if let Some((target_map, ref target)) = data.edges.west {
            let (tx, ty) = resolve_edge_target(target, target_map, gx, gy, registry);
            return Some((target_map, tx, ty));
        }
    }

    None
}

/// Hardcoded fallback — the original edge_transition logic, preserved for
/// use when no RON data is available.
fn edge_transition_hardcoded(map: &MapId, gx: i32, gy: i32) -> Option<(MapId, i32, i32)> {
    let (min_x, max_x, min_y, max_y) = map_bounds_hardcoded(map);

    // ── Door-entry zone checks (BEFORE edge checks) ──────────────────
    if *map == MapId::Farm && (15..=16).contains(&gx) && gy == 2 {
        return Some((MapId::PlayerHouse, 8, 14));
    }
    if *map == MapId::Town && (5..=6).contains(&gx) && gy == 2 {
        return Some((MapId::GeneralStore, 6, 10));
    }
    if *map == MapId::Town && (22..=23).contains(&gx) && gy == 2 {
        return Some((MapId::AnimalShop, 6, 10));
    }
    if *map == MapId::Town && (22..=23).contains(&gx) && gy == 13 {
        return Some((MapId::Blacksmith, 6, 10));
    }
    if *map == MapId::Town && (3..=4).contains(&gx) && gy == 13 {
        return Some((MapId::TownHouseWest, 6, 10));
    }
    if *map == MapId::Town && (9..=10).contains(&gx) && gy == 13 {
        return Some((MapId::TownHouseEast, 6, 10));
    }

    // ── Edge-boundary transitions ────────────────────────────────────
    if *map == MapId::Farm {
        if gy <= min_y {
            return Some((MapId::Town, clamp_to_interior(gx, 0, 27), 20));
        }
        if gx >= max_x {
            return Some((MapId::Forest, 1, clamp_to_interior(gy, 0, 17)));
        }
        if gx <= min_x {
            return Some((MapId::MineEntrance, 12, 6));
        }
    }
    if *map == MapId::Town {
        if gy >= max_y {
            return Some((MapId::Farm, clamp_to_interior(gx, 0, 31), 1));
        }
        if gy <= min_y {
            return Some((MapId::Beach, clamp_to_interior(gx, 0, 19), 7));
        }
        if gx >= max_x {
            return Some((MapId::Forest, 1, clamp_to_interior(gy, 0, 17)));
        }
    }
    if *map == MapId::Beach {
        if gy >= max_y {
            return Some((MapId::Town, clamp_to_interior(gx, 0, 27), 1));
        }
        if gx >= max_x {
            return Some((MapId::Farm, 1, clamp_to_interior(gy, 0, 23)));
        }
    }
    if *map == MapId::Forest {
        if gx <= min_x {
            return Some((MapId::Farm, 30, clamp_to_interior(gy, 0, 23)));
        }
        if gy >= max_y {
            return Some((MapId::MineEntrance, 7, 1));
        }
    }
    if *map == MapId::MineEntrance {
        if gx >= max_x && (5..=6).contains(&gy) {
            return Some((MapId::Farm, 1, 9));
        }
        if gy <= min_y {
            return Some((MapId::Forest, 11, 16));
        }
        if (6..=7).contains(&gx) && gy == 3 {
            return Some((MapId::Mine, 8, 14));
        }
    }
    if *map == MapId::PlayerHouse && gy >= max_y {
        return Some((MapId::Farm, 16, 3));
    }
    if *map == MapId::TownHouseWest && gy >= max_y {
        return Some((MapId::Town, 3, 14));
    }
    if *map == MapId::TownHouseEast && gy >= max_y {
        return Some((MapId::Town, 9, 14));
    }
    if *map == MapId::GeneralStore && gy >= max_y {
        return Some((MapId::Town, 6, 8));
    }
    if *map == MapId::AnimalShop && gy >= max_y {
        return Some((MapId::Town, 22, 8));
    }
    if *map == MapId::Blacksmith && gy >= max_y {
        return Some((MapId::Town, 22, 18));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal registry for tests using the hardcoded generators.
    fn test_registry() -> MapRegistry {
        crate::world::map_data::build_map_registry()
    }

    #[test]
    fn player_house_exit_lands_on_farm_path_outside_door_trigger() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::PlayerHouse, 8, 15, &reg),
            Some((MapId::Farm, 16, 3))
        );
    }

    #[test]
    fn farmhouse_path_tile_does_not_immediately_reenter_house() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::Farm, 16, 3, &reg),
            None
        );
        assert_eq!(
            edge_transition_from_registry(&MapId::Farm, 16, 2, &reg),
            Some((MapId::PlayerHouse, 8, 14))
        );
    }

    #[test]
    fn farm_west_edge_routes_toward_mine_path_not_beach() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::Farm, 0, 10, &reg),
            Some((MapId::MineEntrance, 12, 6))
        );
    }

    #[test]
    fn farm_south_corner_transition_lands_inside_town_not_on_corner() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::Farm, 0, 0, &reg),
            Some((MapId::Town, 1, 20))
        );
        assert_eq!(
            edge_transition_from_registry(&MapId::Farm, 31, 0, &reg),
            Some((MapId::Town, 26, 20))
        );
    }

    #[test]
    fn town_east_corner_transition_lands_inside_forest_not_on_corner() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::Town, 27, 1, &reg),
            Some((MapId::Forest, 1, 1))
        );
        assert_eq!(
            edge_transition_from_registry(&MapId::Town, 27, 20, &reg),
            Some((MapId::Forest, 1, 16))
        );
    }

    #[test]
    fn town_houses_have_enter_and_exit_transitions() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::Town, 3, 13, &reg),
            Some((MapId::TownHouseWest, 6, 10))
        );
        assert_eq!(
            edge_transition_from_registry(&MapId::TownHouseWest, 6, 11, &reg),
            Some((MapId::Town, 3, 14))
        );
        assert_eq!(
            edge_transition_from_registry(&MapId::Town, 9, 13, &reg),
            Some((MapId::TownHouseEast, 6, 10))
        );
        assert_eq!(
            edge_transition_from_registry(&MapId::TownHouseEast, 6, 11, &reg),
            Some((MapId::Town, 9, 14))
        );
    }

    #[test]
    fn mine_entrance_east_path_returns_to_farm() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::MineEntrance, 13, 5, &reg),
            Some((MapId::Farm, 1, 9))
        );
    }

    #[test]
    fn mine_entrance_cave_mouth_enters_mine() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::MineEntrance, 7, 3, &reg),
            Some((MapId::Mine, 8, 14))
        );
    }

    #[test]
    fn mine_exit_spawn_does_not_immediately_reenter_cave() {
        let reg = test_registry();
        assert_eq!(
            edge_transition_from_registry(&MapId::MineEntrance, 7, 4, &reg),
            None
        );
    }
}

/// Check whether the player has reached a map edge and send a
/// `MapTransitionEvent` if so.
pub fn map_transition_check(
    player_state: Res<PlayerState>,
    query: Query<&GridPosition, With<Player>>,
    mut map_events: EventWriter<MapTransitionEvent>,
    registry: Res<MapRegistry>,
) {
    let Ok(grid_pos) = query.get_single() else {
        return;
    };

    if let Some((to_map, to_x, to_y)) =
        edge_transition_from_registry(&player_state.current_map, grid_pos.x, grid_pos.y, &registry)
    {
        map_events.send(MapTransitionEvent { to_map, to_x, to_y });
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
    registry: Res<MapRegistry>,
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
    // for the new map via sync_collision_map when WorldMap updates.
    collision_map.initialised = false;
    collision_map.solid_tiles.clear();

    // Update bounds for the new map.
    let (min_x, max_x, min_y, max_y) = map_bounds_from_registry(&ev.to_map, &registry);
    collision_map.bounds = (min_x, max_x, min_y, max_y);
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
    mut toast_events: EventWriter<ToastEvent>,
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
            info!("[Player] Picked up {} × '{}'", ev.quantity, ev.item_id);
        } else {
            let name = item_registry
                .get(&ev.item_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.item_id);
            toast_events.send(ToastEvent {
                message: format!("Inventory full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
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

        // If the player is in the mine, the mining domain handles the transition
        // (with gold penalty and partial health restore). Skip here.
        let in_mine = player_state.current_map == MapId::Mine;
        if in_mine {
            continue;
        }

        // Restore health fully.
        player_state.health = player_state.max_health;

        let bed_gx = 12;
        let bed_gy = 4;

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

/// Grant starter items on first entering Playing state (inventory is empty).
/// The intro dialogue mentions "seeds in your pack" so we deliver on that promise.
pub fn grant_starter_items(mut inventory: ResMut<Inventory>, item_registry: Res<ItemRegistry>) {
    // Only grant if inventory is completely empty (fresh game, not a load).
    let has_items = inventory.slots.iter().any(|s| s.is_some());
    if has_items {
        return;
    }

    let starters = [
        ("hoe", 1u8),         // Required to till soil — first step in farming
        ("turnip_seeds", 15), // Spring crop — enough for a starter plot
        ("potato_seeds", 5),  // Second spring crop
        ("wood", 20),         // For crafting a chest or fence
        ("stone", 15),        // Basic materials
        ("bread", 3),         // Food to restore stamina on Day 1
    ];

    for (item_id, qty) in &starters {
        let max_stack = item_registry
            .get(item_id)
            .map(|def| def.stack_size)
            .unwrap_or(99);
        inventory.try_add(item_id, *qty, max_stack);
    }

    info!("Granted starter items to new player");
}
