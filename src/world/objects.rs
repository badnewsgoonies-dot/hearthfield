//! World objects: trees, rocks, stumps, bushes, logs, and forageables.
//!
//! These are interactive entities that exist on maps. They respond to tool
//! use events and can be destroyed to drop items.

use bevy::prelude::*;
use crate::shared::*;

use super::maps::{WorldObjectKind, ObjectPlacement};
use super::WorldMap;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker for all world object entities (for bulk despawn on map change).
#[derive(Component, Debug)]
pub struct WorldObject;

/// Tracks the kind, health, and grid position of a world object.
#[derive(Component, Debug, Clone)]
pub struct WorldObjectData {
    pub kind: WorldObjectKind,
    pub health: u8,
    pub max_health: u8,
    pub grid_x: i32,
    pub grid_y: i32,
}

/// Marker for forageable item entities.
#[derive(Component, Debug)]
pub struct Forageable {
    pub item_id: ItemId,
    pub grid_x: i32,
    pub grid_y: i32,
}

// ═══════════════════════════════════════════════════════════════════════
// OBJECT PROPERTIES
// ═══════════════════════════════════════════════════════════════════════

impl WorldObjectKind {
    /// Max health for this object type.
    pub fn max_health(self) -> u8 {
        match self {
            WorldObjectKind::Tree => 10,
            WorldObjectKind::Rock => 6,
            WorldObjectKind::Stump => 4,
            WorldObjectKind::Bush => 2,
            WorldObjectKind::LargeRock => 12,
            WorldObjectKind::Log => 4,
        }
    }

    /// Which tool is effective against this object.
    pub fn effective_tool(self) -> ToolKind {
        match self {
            WorldObjectKind::Tree => ToolKind::Axe,
            WorldObjectKind::Stump => ToolKind::Axe,
            WorldObjectKind::Bush => ToolKind::Scythe,
            WorldObjectKind::Log => ToolKind::Axe,
            WorldObjectKind::Rock => ToolKind::Pickaxe,
            WorldObjectKind::LargeRock => ToolKind::Pickaxe,
        }
    }

    /// Damage dealt by the effective tool per tier.
    pub fn tool_damage(self, tier: ToolTier) -> u8 {
        let base = match tier {
            ToolTier::Basic => 1,
            ToolTier::Copper => 2,
            ToolTier::Iron => 3,
            ToolTier::Gold => 4,
            ToolTier::Iridium => 6,
        };
        // Scythe-type objects take extra damage
        if matches!(self, WorldObjectKind::Bush) {
            base * 2
        } else {
            base
        }
    }

    /// Items dropped when this object is destroyed. Returns (item_id, quantity).
    pub fn drops(self) -> Vec<(&'static str, u8)> {
        match self {
            WorldObjectKind::Tree => vec![("wood", 8), ("tree_seed", 1)],
            WorldObjectKind::Stump => vec![("wood", 4)],
            WorldObjectKind::Log => vec![("wood", 4)],
            WorldObjectKind::Bush => vec![("fiber", 2)],
            WorldObjectKind::Rock => vec![("stone", 4)],
            WorldObjectKind::LargeRock => vec![("stone", 8), ("copper_ore", 2)],
        }
    }

    /// Color used for placeholder sprite.
    pub fn color(self) -> Color {
        match self {
            WorldObjectKind::Tree => Color::srgb(0.15, 0.5, 0.15),
            WorldObjectKind::Stump => Color::srgb(0.45, 0.35, 0.2),
            WorldObjectKind::Log => Color::srgb(0.5, 0.38, 0.22),
            WorldObjectKind::Bush => Color::srgb(0.2, 0.55, 0.25),
            WorldObjectKind::Rock => Color::srgb(0.55, 0.55, 0.58),
            WorldObjectKind::LargeRock => Color::srgb(0.45, 0.45, 0.5),
        }
    }

    /// Sprite size for this object.
    pub fn sprite_size(self) -> Vec2 {
        match self {
            WorldObjectKind::Tree => Vec2::new(TILE_SIZE, TILE_SIZE * 2.0),
            WorldObjectKind::LargeRock => Vec2::new(TILE_SIZE * 1.5, TILE_SIZE * 1.5),
            _ => Vec2::new(TILE_SIZE, TILE_SIZE),
        }
    }

    /// Whether this object blocks movement.
    pub fn is_solid(self) -> bool {
        true
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWNING
// ═══════════════════════════════════════════════════════════════════════

/// Spawn world objects from a list of placements.
pub fn spawn_world_objects(
    commands: &mut Commands,
    placements: &[ObjectPlacement],
    world_map: &mut WorldMap,
) {
    for placement in placements {
        let kind = placement.kind;
        let data = WorldObjectData {
            kind,
            health: kind.max_health(),
            max_health: kind.max_health(),
            grid_x: placement.x,
            grid_y: placement.y,
        };

        let size = kind.sprite_size();
        // For tall objects like trees, offset Y so the base aligns with the tile
        let y_offset = if size.y > TILE_SIZE {
            (size.y - TILE_SIZE) / 2.0
        } else {
            0.0
        };

        commands.spawn((
            Sprite {
                color: kind.color(),
                custom_size: Some(size),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                placement.x as f32 * TILE_SIZE,
                placement.y as f32 * TILE_SIZE + y_offset,
                5.0, // Above tiles, below UI
            )),
            WorldObject,
            data,
        ));

        // Mark the tile as solid in the collision map
        world_map.set_solid(placement.x, placement.y, true);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TOOL USE HANDLING
// ═══════════════════════════════════════════════════════════════════════

/// System that handles tool use events on world objects.
pub fn handle_tool_use_on_objects(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    mut objects: Query<(Entity, &mut WorldObjectData, &mut Sprite), With<WorldObject>>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut world_map: ResMut<WorldMap>,
) {
    for event in tool_events.read() {
        for (entity, mut obj_data, mut sprite) in objects.iter_mut() {
            if obj_data.grid_x == event.target_x && obj_data.grid_y == event.target_y {
                let effective = obj_data.kind.effective_tool();
                if event.tool == effective {
                    let damage = obj_data.kind.tool_damage(event.tier);
                    let new_health = obj_data.health.saturating_sub(damage);
                    obj_data.health = new_health;

                    // Visual feedback: lighten color as health decreases
                    let health_ratio = obj_data.health as f32 / obj_data.max_health as f32;
                    let base_color = obj_data.kind.color();
                    let linear = base_color.to_linear();
                    let r = linear.red;
                    let g = linear.green;
                    let b = linear.blue;
                    sprite.color = Color::srgb(
                        r + (1.0 - r) * (1.0 - health_ratio) * 0.3,
                        g + (1.0 - g) * (1.0 - health_ratio) * 0.3,
                        b + (1.0 - b) * (1.0 - health_ratio) * 0.3,
                    );

                    // Play hit sound
                    sfx_writer.send(PlaySfxEvent {
                        sfx_id: match effective {
                            ToolKind::Axe => "chop".to_string(),
                            ToolKind::Pickaxe => "rock_hit".to_string(),
                            ToolKind::Scythe => "swish".to_string(),
                            _ => "hit".to_string(),
                        },
                    });

                    if new_health == 0 {
                        // Object destroyed: drop items
                        let drops = obj_data.kind.drops();
                        for (item_id, quantity) in drops {
                            pickup_writer.send(ItemPickupEvent {
                                item_id: item_id.to_string(),
                                quantity,
                            });
                        }

                        // If it was a tree, leave a stump
                        if obj_data.kind == WorldObjectKind::Tree {
                            let stump_data = WorldObjectData {
                                kind: WorldObjectKind::Stump,
                                health: WorldObjectKind::Stump.max_health(),
                                max_health: WorldObjectKind::Stump.max_health(),
                                grid_x: obj_data.grid_x,
                                grid_y: obj_data.grid_y,
                            };
                            commands.spawn((
                                Sprite {
                                    color: WorldObjectKind::Stump.color(),
                                    custom_size: Some(WorldObjectKind::Stump.sprite_size()),
                                    ..default()
                                },
                                Transform::from_translation(Vec3::new(
                                    obj_data.grid_x as f32 * TILE_SIZE,
                                    obj_data.grid_y as f32 * TILE_SIZE,
                                    5.0,
                                )),
                                WorldObject,
                                stump_data,
                            ));
                            // Tile stays solid (stump)
                        } else {
                            // Clear solid flag
                            world_map.set_solid(obj_data.grid_x, obj_data.grid_y, false);
                        }

                        // Despawn the destroyed object
                        commands.entity(entity).despawn();

                        // Play destruction sound
                        sfx_writer.send(PlaySfxEvent {
                            sfx_id: "object_break".to_string(),
                        });
                    }
                }
                // Only one object per tile, so break after finding the match
                break;
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// FORAGEABLES
// ═══════════════════════════════════════════════════════════════════════

/// Seasonal forageable definitions: (item_id, color for placeholder).
pub fn seasonal_forageables(season: Season) -> Vec<(&'static str, Color)> {
    match season {
        Season::Spring => vec![
            ("wild_horseradish", Color::srgb(0.8, 0.85, 0.7)),
            ("daffodil", Color::srgb(0.95, 0.9, 0.3)),
            ("leek", Color::srgb(0.4, 0.7, 0.35)),
            ("dandelion", Color::srgb(0.9, 0.85, 0.2)),
            ("spring_onion", Color::srgb(0.5, 0.75, 0.4)),
        ],
        Season::Summer => vec![
            ("grape", Color::srgb(0.5, 0.2, 0.6)),
            ("spice_berry", Color::srgb(0.8, 0.2, 0.2)),
            ("sweet_pea", Color::srgb(0.85, 0.6, 0.8)),
            ("red_mushroom", Color::srgb(0.8, 0.15, 0.1)),
        ],
        Season::Fall => vec![
            ("common_mushroom", Color::srgb(0.7, 0.55, 0.35)),
            ("wild_plum", Color::srgb(0.5, 0.2, 0.4)),
            ("hazelnut", Color::srgb(0.6, 0.45, 0.25)),
            ("blackberry", Color::srgb(0.2, 0.1, 0.25)),
            ("chanterelle", Color::srgb(0.9, 0.7, 0.3)),
        ],
        Season::Winter => vec![
            ("winter_root", Color::srgb(0.7, 0.55, 0.4)),
            ("crystal_fruit", Color::srgb(0.6, 0.8, 0.95)),
            ("snow_yam", Color::srgb(0.9, 0.88, 0.85)),
            ("crocus", Color::srgb(0.7, 0.5, 0.85)),
        ],
    }
}

/// Spawn forageables for the current day on the active map.
pub fn spawn_forageables(
    commands: &mut Commands,
    forage_points: &[(i32, i32)],
    season: Season,
    day: u8,
    world_map: &WorldMap,
) {
    let forageables = seasonal_forageables(season);
    if forageables.is_empty() {
        return;
    }

    // Use day as a seed for pseudo-random selection (deterministic per day)
    // Spawn on roughly 40-60% of available points, varying by day
    for (i, &(gx, gy)) in forage_points.iter().enumerate() {
        // Simple hash to determine if this point spawns today
        let hash = ((day as usize).wrapping_mul(31).wrapping_add(i.wrapping_mul(17))) % 10;
        if hash > 5 {
            continue; // ~40% spawn rate
        }

        // Don't spawn on solid tiles
        if world_map.is_solid(gx, gy) {
            continue;
        }

        // Pick which forageable
        let idx = ((day as usize).wrapping_mul(7).wrapping_add(i.wrapping_mul(13))) % forageables.len();
        let (item_id, color) = &forageables[idx];

        commands.spawn((
            Sprite {
                color: *color,
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                ..default()
            },
            Transform::from_translation(Vec3::new(
                gx as f32 * TILE_SIZE,
                gy as f32 * TILE_SIZE,
                4.0,
            )),
            WorldObject,
            Forageable {
                item_id: item_id.to_string(),
                grid_x: gx,
                grid_y: gy,
            },
        ));
    }
}

/// System: player picks up forageables by walking over them (or interacting).
/// For now, we check if there's a tool use event on a forageable tile.
pub fn handle_forageable_pickup(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    forageables: Query<(Entity, &Forageable), With<WorldObject>>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for event in tool_events.read() {
        for (entity, forageable) in forageables.iter() {
            if forageable.grid_x == event.target_x && forageable.grid_y == event.target_y {
                // Pick it up regardless of tool (interacting)
                pickup_writer.send(ItemPickupEvent {
                    item_id: forageable.item_id.clone(),
                    quantity: 1,
                });
                sfx_writer.send(PlaySfxEvent {
                    sfx_id: "pickup".to_string(),
                });
                commands.entity(entity).despawn();
                break;
            }
        }
    }
}
