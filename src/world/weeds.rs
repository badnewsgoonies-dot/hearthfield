//! Weed spawning and removal system.
//!
//! Weeds sprout overnight on the farm, occupying empty tiles.
//! The player clears them with the scythe, receiving fiber.

use bevy::prelude::*;
use crate::shared::*;

use super::{CurrentMapId, WorldMap};
use super::maps::WorldObjectKind;
use super::objects::{WorldObject, WorldObjectData};

// ═══════════════════════════════════════════════════════════════════════
// COMPONENT
// ═══════════════════════════════════════════════════════════════════════

/// Marker for weed entities on the farm.
#[derive(Component, Debug)]
pub struct Weed {
    pub grid_x: i32,
    pub grid_y: i32,
}

// ═══════════════════════════════════════════════════════════════════════
// WEED SPAWNING (on DayEndEvent)
// ═══════════════════════════════════════════════════════════════════════

/// On each day end, spawn 2-4 weeds at random empty farm tiles.
pub fn spawn_daily_weeds(
    mut commands: Commands,
    mut day_events: EventReader<DayEndEvent>,
    current_map: Res<CurrentMapId>,
    world_map: Res<WorldMap>,
    farm_state: Res<FarmState>,
    existing_weeds: Query<&Weed>,
    objects: Query<&WorldObjectData, With<WorldObject>>,
) {
    for event in day_events.read() {
        // Only spawn weeds on the farm map
        if current_map.map_id != MapId::Farm {
            continue;
        }

        // Cap max weeds on the farm at 20 to prevent weed spam.
        let existing_weed_count = existing_weeds.iter().count();
        if existing_weed_count >= 20 {
            continue;
        }

        // Build a set of occupied positions (crops, objects, existing weeds)
        let mut occupied = std::collections::HashSet::new();
        for (pos, _) in farm_state.crops.iter() {
            occupied.insert(*pos);
        }
        for (pos, _) in farm_state.soil.iter() {
            occupied.insert(*pos);
        }
        for (pos, _) in farm_state.objects.iter() {
            occupied.insert(*pos);
        }
        for weed in existing_weeds.iter() {
            occupied.insert((weed.grid_x, weed.grid_y));
        }
        for obj in objects.iter() {
            occupied.insert((obj.grid_x, obj.grid_y));
        }

        // Determine number of weeds to spawn (2-4), using day as seed
        let seed = (event.day as u32)
            .wrapping_mul(37)
            .wrapping_add(event.year.wrapping_mul(113))
            .wrapping_add(event.season.index() as u32 * 7);
        let raw_count = 2 + (seed % 3) as i32; // 2, 3, or 4
        // Clamp so we never exceed 20 total weeds on the farm.
        let count = raw_count.min(20i32 - existing_weed_count as i32);
        if count <= 0 {
            continue;
        }

        let mut spawned = 0;
        // Try positions within the farm bounds (0..20, 0..20)
        let farm_w = 20i32;
        let farm_h = 20i32;

        for attempt in 0..40u32 {
            if spawned >= count {
                break;
            }

            // Pseudo-random position derived from day + attempt
            let hash = seed
                .wrapping_mul(attempt.wrapping_add(1).wrapping_mul(61))
                .wrapping_add(attempt.wrapping_mul(17));
            let x = (hash % farm_w as u32) as i32;
            let y = ((hash / farm_w as u32) % farm_h as u32) as i32;

            // Skip if occupied or solid terrain
            if occupied.contains(&(x, y)) {
                continue;
            }
            if world_map.is_solid(x, y) {
                continue;
            }

            // Spawn the weed entity
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.25, 0.55, 0.18),
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.5)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    x as f32 * TILE_SIZE,
                    y as f32 * TILE_SIZE,
                    4.5, // Above tiles, below full objects
                )),
                Weed { grid_x: x, grid_y: y },
            ));

            occupied.insert((x, y));
            spawned += 1;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// WEED SCYTHE HANDLING
// ═══════════════════════════════════════════════════════════════════════

/// When the player uses the scythe on a tile containing a weed, destroy
/// the weed and drop fiber.
pub fn handle_weed_scythe(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    weeds: Query<(Entity, &Weed)>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for event in tool_events.read() {
        if event.tool != ToolKind::Scythe {
            continue;
        }

        for (entity, weed) in weeds.iter() {
            if weed.grid_x == event.target_x && weed.grid_y == event.target_y {
                // Drop fiber
                pickup_writer.send(ItemPickupEvent {
                    item_id: "fiber".to_string(),
                    quantity: 1,
                });

                sfx_writer.send(PlaySfxEvent {
                    sfx_id: "swish".to_string(),
                });

                commands.entity(entity).despawn();
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weed_spawn_count_bounds() {
        // The formula is: count = 2 + (seed % 3), giving values 2, 3, or 4.
        // Verify for a range of inputs.
        for day in 1u8..=28 {
            for year in 1u32..=3 {
                for season_idx in 0u32..=3 {
                    let seed = (day as u32)
                        .wrapping_mul(37)
                        .wrapping_add(year.wrapping_mul(113))
                        .wrapping_add(season_idx * 7);
                    let count = 2 + (seed % 3) as i32;
                    assert!(count >= 2 && count <= 4,
                        "Weed count should be 2-4, got {} for day={} year={} season={}",
                        count, day, year, season_idx);
                }
            }
        }
    }

    #[test]
    fn test_weed_position_within_farm_bounds() {
        // Verify the position hash always stays within farm bounds (0..20, 0..20)
        let farm_w = 20i32;
        let farm_h = 20i32;
        for day in 1u8..=28 {
            let seed = (day as u32).wrapping_mul(37).wrapping_add(113);
            for attempt in 0..40u32 {
                let hash = seed
                    .wrapping_mul(attempt.wrapping_add(1).wrapping_mul(61))
                    .wrapping_add(attempt.wrapping_mul(17));
                let x = (hash % farm_w as u32) as i32;
                let y = ((hash / farm_w as u32) % farm_h as u32) as i32;
                assert!(x >= 0 && x < farm_w, "x={} out of bounds", x);
                assert!(y >= 0 && y < farm_h, "y={} out of bounds", y);
            }
        }
    }
}
