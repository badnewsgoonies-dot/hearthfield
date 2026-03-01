//! World objects: trees, rocks, stumps, bushes, logs, and forageables.
//!
//! These are interactive entities that exist on maps. They respond to tool
//! use events and can be destroyed to drop items.

use bevy::prelude::*;
use crate::shared::*;

use super::maps::{WorldObjectKind, ObjectPlacement};
use super::WorldMap;

// ═══════════════════════════════════════════════════════════════════════
// OBJECT ATLAS RESOURCE
// ═══════════════════════════════════════════════════════════════════════

/// Caches loaded texture atlas handles for world objects.
/// Loaded lazily on first map spawn.
#[derive(Resource, Default)]
pub struct ObjectAtlases {
    pub loaded: bool,
    pub grass_biome_image: Handle<Image>,
    pub grass_biome_layout: Handle<TextureAtlasLayout>,
    pub fences_image: Handle<Image>,
    pub fences_layout: Handle<TextureAtlasLayout>,
    // Building tilesets (Sprout Lands)
    pub house_walls_image: Handle<Image>,
    pub house_walls_layout: Handle<TextureAtlasLayout>,
    pub house_roof_image: Handle<Image>,
    pub house_roof_layout: Handle<TextureAtlasLayout>,
    pub doors_image: Handle<Image>,
    pub doors_layout: Handle<TextureAtlasLayout>,
    pub hills_image: Handle<Image>,
    pub hills_layout: Handle<TextureAtlasLayout>,
    pub wood_bridge_image: Handle<Image>,
    pub wood_bridge_layout: Handle<TextureAtlasLayout>,
    pub tools_image: Handle<Image>,
    pub tools_layout: Handle<TextureAtlasLayout>,
}

/// Loads object atlas assets on first use. Subsequent calls are no-ops.
pub fn ensure_object_atlases_loaded(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlases: &mut ObjectAtlases,
) {
    if atlases.loaded {
        return;
    }

    // grass_biome.png: 144x80px -> 16x16 tiles, 9 columns x 5 rows
    atlases.grass_biome_image = asset_server.load("sprites/grass_biome.png");
    atlases.grass_biome_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        9,
        5,
        None,
        None,
    ));

    // fences.png: 64x64px -> 16x16 tiles, 4 columns x 4 rows
    atlases.fences_image = asset_server.load("tilesets/fences.png");
    atlases.fences_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        4,
        4,
        None,
        None,
    ));

    // house_walls.png: 80x48px -> 16x16 tiles, 5 columns x 3 rows
    atlases.house_walls_image = asset_server.load("tilesets/house_walls.png");
    atlases.house_walls_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        5,
        3,
        None,
        None,
    ));

    // house_roof.png: 112x80px -> 16x16 tiles, 7 columns x 5 rows
    atlases.house_roof_image = asset_server.load("tilesets/house_roof.png");
    atlases.house_roof_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        7,
        5,
        None,
        None,
    ));

    // doors.png: 16x64px -> 16x16 tiles, 1 column x 4 rows
    atlases.doors_image = asset_server.load("tilesets/doors.png");
    atlases.doors_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        1,
        4,
        None,
        None,
    ));

    // hills.png: 176x144px -> 16x16 tiles, 11 columns x 9 rows
    atlases.hills_image = asset_server.load("tilesets/hills.png");
    atlases.hills_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        11,
        9,
        None,
        None,
    ));

    // wood_bridge.png: 80x48px -> 16x16 tiles, 5 columns x 3 rows
    atlases.wood_bridge_image = asset_server.load("sprites/wood_bridge.png");
    atlases.wood_bridge_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        5,
        3,
        None,
        None,
    ));

    // tools.png: 96x96px -> 16x16 tiles, 6 columns x 6 rows
    atlases.tools_image = asset_server.load("sprites/tools.png");
    atlases.tools_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        6,
        6,
        None,
        None,
    ));

    atlases.loaded = true;
}

// ═══════════════════════════════════════════════════════════════════════
// FURNITURE ATLAS RESOURCE
// ═══════════════════════════════════════════════════════════════════════

/// Caches loaded texture atlas handles for furniture sprites (shipping bin,
/// crafting bench, carpenter board, machines, feed trough).
/// Loaded lazily on first map spawn.
#[derive(Resource, Default)]
pub struct FurnitureAtlases {
    pub loaded: bool,
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Loads the furniture atlas on first use. Subsequent calls are no-ops.
pub fn ensure_furniture_atlases_loaded(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    atlases: &mut FurnitureAtlases,
) {
    if atlases.loaded {
        return;
    }
    // furniture.png: 144x96px -> 16x16 tiles, 9 columns x 6 rows
    atlases.image = asset_server.load("sprites/furniture.png");
    atlases.layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(16, 16),
        9,
        6,
        None,
        None,
    ));
    atlases.loaded = true;
}

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Marker for all world object entities (for bulk despawn on map change).
#[derive(Component, Debug)]
pub struct WorldObject;

/// Marker for the shipping bin interactable entity.
#[derive(Component, Debug)]
pub struct ShippingBinMarker;

/// Marker for the carpenter board interactable entity.
#[derive(Component, Debug)]
pub struct CarpenterBoardMarker;

/// Marker for the crafting bench interactable entity.
#[derive(Component, Debug)]
pub struct CraftingBenchMarker;

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
    ///
    /// Enhanced drop tables:
    /// - Tree: 3-5 wood + always a tree_seed
    /// - Rock: 2-3 stone + copper_ore
    /// - LargeRock: 5 stone + 2 copper_ore + geode
    /// - Stump: 2 hardwood
    /// - Log: 4 hardwood
    /// - Bush: 2 fiber + seasonal berry
    pub fn drops(self) -> Vec<(&'static str, u8)> {
        match self {
            WorldObjectKind::Tree => vec![("wood", 4), ("tree_seed", 1)],
            WorldObjectKind::Stump => vec![("hardwood", 2)],
            WorldObjectKind::Log => vec![("hardwood", 4)],
            WorldObjectKind::Bush => vec![("fiber", 2), ("wild_berry", 1)],
            WorldObjectKind::Rock => vec![("stone", 3), ("copper_ore", 1)],
            WorldObjectKind::LargeRock => vec![("stone", 5), ("copper_ore", 2), ("geode", 1)],
        }
    }

    /// Color used for placeholder sprite (fallback).
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
    #[allow(dead_code)]
    pub fn is_solid(self) -> bool {
        true
    }

    /// Minimum tool tier required to damage this object.
    pub fn required_tier(self) -> ToolTier {
        match self {
            WorldObjectKind::LargeRock => ToolTier::Gold,
            WorldObjectKind::Stump => ToolTier::Copper,
            WorldObjectKind::Log => ToolTier::Iron,
            _ => ToolTier::Basic,
        }
    }

    /// Atlas index in grass_biome.png for this object kind.
    /// grass_biome.png is 9 columns x 5 rows of 16x16 tiles.
    /// Typical layout:
    ///   Row 0: grass decorations / small plants
    ///   Row 1: bushes, small tree tops
    ///   Row 2: tree trunk / mid sections
    ///   Row 3: rocks, stumps
    ///   Row 4: logs, large rocks
    pub fn atlas_index(self) -> usize {
        match self {
            WorldObjectKind::Tree => 10,    // row 1, col 1 — tree/bush top
            WorldObjectKind::Bush => 1,     // row 0, col 1 — small bush/grass
            WorldObjectKind::Stump => 27,   // row 3, col 0 — stump/rock-like
            WorldObjectKind::Rock => 29,    // row 3, col 2 — rock
            WorldObjectKind::LargeRock => 38, // row 4, col 2 — large rock
            WorldObjectKind::Log => 36,     // row 4, col 0 — log
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SPAWNING
// ═══════════════════════════════════════════════════════════════════════

/// Spawn world objects from a list of placements, using texture atlas sprites.
pub fn spawn_world_objects(
    commands: &mut Commands,
    placements: &[ObjectPlacement],
    world_map: &mut WorldMap,
    object_atlases: &ObjectAtlases,
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

        if object_atlases.loaded {
            // Use atlas sprite from grass_biome.png
            let atlas_index = kind.atlas_index();
            let mut sprite = Sprite::from_atlas_image(
                object_atlases.grass_biome_image.clone(),
                TextureAtlas {
                    layout: object_atlases.grass_biome_layout.clone(),
                    index: atlas_index,
                },
            );
            sprite.custom_size = Some(size);

            let wc = grid_to_world_center(placement.x, placement.y);
            commands.spawn((
                sprite,
                Transform::from_translation(Vec3::new(
                    wc.x,
                    wc.y + y_offset,
                    Z_ENTITY_BASE,
                )),
                WorldObject,
                YSorted,
                data,
            ));
        } else {
            // Fallback: colored rectangle if atlases failed to load
            let wc = grid_to_world_center(placement.x, placement.y);
            commands.spawn((
                Sprite {
                    color: kind.color(),
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    wc.x,
                    wc.y + y_offset,
                    Z_ENTITY_BASE,
                )),
                WorldObject,
                YSorted,
                data,
            ));
        }

        // Mark the tile as solid in the collision map
        world_map.set_solid(placement.x, placement.y, true);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// TOOL USE HANDLING
// ═══════════════════════════════════════════════════════════════════════

/// Returns a numeric level for a ToolTier, used for tier comparison.
fn tier_level(tier: ToolTier) -> u8 {
    match tier {
        ToolTier::Basic => 0,
        ToolTier::Copper => 1,
        ToolTier::Iron => 2,
        ToolTier::Gold => 3,
        ToolTier::Iridium => 4,
    }
}

/// Human-readable tool name for toast messages.
fn tool_display_name(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Axe => "axe",
        ToolKind::Pickaxe => "pickaxe",
        ToolKind::Hoe => "hoe",
        ToolKind::WateringCan => "watering can",
        ToolKind::FishingRod => "fishing rod",
        ToolKind::Scythe => "scythe",
    }
}

/// System that handles tool use events on world objects.
pub fn handle_tool_use_on_objects(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    mut objects: Query<(Entity, &mut WorldObjectData, &mut Sprite), With<WorldObject>>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
    mut world_map: ResMut<WorldMap>,
    object_atlases: Res<ObjectAtlases>,
) {
    for event in tool_events.read() {
        for (entity, mut obj_data, mut sprite) in objects.iter_mut() {
            if obj_data.grid_x == event.target_x && obj_data.grid_y == event.target_y {
                let effective = obj_data.kind.effective_tool();
                if event.tool == effective {
                    // Check tool tier requirement
                    let required = obj_data.kind.required_tier();
                    if tier_level(event.tier) < tier_level(required) {
                        toast_writer.send(ToastEvent {
                            message: format!(
                                "Need a better {}!",
                                tool_display_name(event.tool)
                            ),
                            duration_secs: 2.0,
                        });
                        break;
                    }

                    let damage = obj_data.kind.tool_damage(event.tier);
                    let new_health = obj_data.health.saturating_sub(damage);
                    obj_data.health = new_health;

                    // Visual feedback: tint the sprite as health decreases
                    let health_ratio = obj_data.health as f32 / obj_data.max_health as f32;
                    let tint = 0.7 + 0.3 * health_ratio; // from 1.0 (full) to 0.7 (nearly dead)
                    sprite.color = Color::srgb(tint, tint, tint);

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

                            if object_atlases.loaded {
                                let stump_index = WorldObjectKind::Stump.atlas_index();
                                let mut stump_sprite = Sprite::from_atlas_image(
                                    object_atlases.grass_biome_image.clone(),
                                    TextureAtlas {
                                        layout: object_atlases.grass_biome_layout.clone(),
                                        index: stump_index,
                                    },
                                );
                                stump_sprite.custom_size =
                                    Some(WorldObjectKind::Stump.sprite_size());

                                let stump_wc = grid_to_world_center(obj_data.grid_x, obj_data.grid_y);
                                commands.spawn((
                                    stump_sprite,
                                    Transform::from_translation(Vec3::new(
                                        stump_wc.x,
                                        stump_wc.y,
                                        Z_ENTITY_BASE,
                                    )),
                                    WorldObject,
                                    YSorted,
                                    stump_data,
                                ));
                            } else {
                                // Fallback: colored rectangle
                                let stump_wc = grid_to_world_center(obj_data.grid_x, obj_data.grid_y);
                                commands.spawn((
                                    Sprite {
                                        color: WorldObjectKind::Stump.color(),
                                        custom_size: Some(WorldObjectKind::Stump.sprite_size()),
                                        ..default()
                                    },
                                    Transform::from_translation(Vec3::new(
                                        stump_wc.x,
                                        stump_wc.y,
                                        Z_ENTITY_BASE,
                                    )),
                                    WorldObject,
                                    YSorted,
                                    stump_data,
                                ));
                            }
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

/// Maps a forageable item_id to an atlas index in grass_biome.png.
fn forageable_atlas_index(item_id: &str) -> Option<usize> {
    Some(match item_id {
        "wild_horseradish" => 3,
        "daffodil" => 4,
        "leek" => 5,
        "dandelion" => 7,
        "spring_onion" => 8,
        "grape" => 11,
        "spice_berry" => 12,
        "sweet_pea" => 13,
        "red_mushroom" => 14,
        "common_mushroom" => 9,
        "wild_plum" => 15,
        "hazelnut" => 16,
        "blackberry" => 17,
        "chanterelle" => 10,
        "winter_root" => 16,
        "crystal_fruit" => 13,
        "snow_yam" => 3,
        "crocus" => 7,
        _ => return None,
    })
}

/// Spawn forageables for the current day on the active map.
pub fn spawn_forageables(
    commands: &mut Commands,
    forage_points: &[(i32, i32)],
    season: Season,
    day: u8,
    world_map: &WorldMap,
    object_atlases: &ObjectAtlases,
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

        let fwc = grid_to_world_center(gx, gy);
        let sprite = if let Some(atlas_idx) = forageable_atlas_index(item_id) {
            if object_atlases.loaded {
                let mut s = Sprite::from_atlas_image(
                    object_atlases.grass_biome_image.clone(),
                    TextureAtlas {
                        layout: object_atlases.grass_biome_layout.clone(),
                        index: atlas_idx,
                    },
                );
                s.custom_size = Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7));
                s
            } else {
                Sprite {
                    color: *color,
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                    ..default()
                }
            }
        } else {
            Sprite {
                color: *color,
                custom_size: Some(Vec2::new(TILE_SIZE * 0.7, TILE_SIZE * 0.7)),
                ..default()
            }
        };

        commands.spawn((
            sprite,
            Transform::from_translation(Vec3::new(
                fwc.x,
                fwc.y,
                Z_ENTITY_BASE,
            )),
            WorldObject,
            YSorted,
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

// ═══════════════════════════════════════════════════════════════════════
// WEEDS
// ═══════════════════════════════════════════════════════════════════════

/// A weed that spawns on empty farm tiles overnight.
/// Can be cleared with a scythe for fiber.
#[derive(Component, Debug)]
pub struct Weed {
    pub grid_x: i32,
    pub grid_y: i32,
}

/// System: on DayEndEvent, spawn 2-4 weeds on random empty farm tiles.
/// Only spawns weeds when the current map is Farm.
pub fn spawn_daily_weeds(
    mut commands: Commands,
    mut day_events: EventReader<DayEndEvent>,
    current_map: Res<super::CurrentMapId>,
    world_map: Res<super::WorldMap>,
    farm_state: Res<FarmState>,
    existing_weeds: Query<&Weed>,
    object_atlases: Res<ObjectAtlases>,
) {
    for event in day_events.read() {
        // Only spawn weeds on the farm map
        if current_map.map_id != MapId::Farm {
            continue;
        }

        // Collect existing weed positions so we don't stack
        let mut occupied: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for weed in existing_weeds.iter() {
            occupied.insert((weed.grid_x, weed.grid_y));
        }

        // Determine how many weeds to spawn (2-4), using day as pseudo-random seed
        let weed_count = 2 + ((event.day as usize).wrapping_mul(13).wrapping_add(event.season.index())) % 3;

        let farm_w = 20i32;
        let farm_h = 20i32;
        let mut spawned = 0;

        // Attempt up to weed_count * 10 random positions to find valid spots
        for attempt in 0..(weed_count * 10) {
            if spawned >= weed_count {
                break;
            }

            // Simple deterministic pseudo-random based on day, season, and attempt index
            let hash = (event.day as usize)
                .wrapping_mul(31)
                .wrapping_add(event.season.index().wrapping_mul(97))
                .wrapping_add(attempt.wrapping_mul(53))
                .wrapping_add(event.year as usize * 7);
            let x = (hash % farm_w as usize) as i32;
            let y = (((hash / farm_w as usize).wrapping_mul(17).wrapping_add(attempt * 3)) % farm_h as usize) as i32;

            // Skip if tile is solid (water, objects, etc.)
            if world_map.is_solid(x, y) {
                continue;
            }

            // Skip if there's already a crop here
            if farm_state.crops.contains_key(&(x, y)) {
                continue;
            }

            // Skip if there's already soil tilled here
            if farm_state.soil.contains_key(&(x, y)) {
                continue;
            }

            // Skip if a weed already exists here
            if occupied.contains(&(x, y)) {
                continue;
            }

            // Spawn the weed entity
            let wwc = grid_to_world_center(x, y);
            let sprite = if object_atlases.loaded {
                let mut s = Sprite::from_atlas_image(
                    object_atlases.grass_biome_image.clone(),
                    TextureAtlas {
                        layout: object_atlases.grass_biome_layout.clone(),
                        index: 2, // weed/grass frame from row 0
                    },
                );
                s.custom_size = Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.5));
                s
            } else {
                Sprite {
                    color: Color::srgb(0.25, 0.55, 0.2),
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.5)),
                    ..default()
                }
            };
            commands.spawn((
                sprite,
                Transform::from_translation(Vec3::new(wwc.x, wwc.y, Z_ENTITY_BASE)),
                LogicalPosition(Vec2::new(wwc.x, wwc.y)),
                YSorted,
                Weed {
                    grid_x: x,
                    grid_y: y,
                },
            ));

            occupied.insert((x, y));
            spawned += 1;
        }
    }
}

/// System: handle scythe use on weeds. Despawn the weed and drop fiber.
pub fn handle_weed_scythe(
    mut commands: Commands,
    mut tool_events: EventReader<ToolUseEvent>,
    weeds: Query<(Entity, &Weed)>,
    mut pickup_writer: EventWriter<ItemPickupEvent>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
) {
    for event in tool_events.read() {
        // Only scythe clears weeds
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

// ═══════════════════════════════════════════════════════════════════════
// TREE REGROWTH
// ═══════════════════════════════════════════════════════════════════════

/// System: on each season change, spawn 1-2 new trees at random empty
/// positions on the farm. This simulates natural regrowth over time.
pub fn regrow_trees_on_season_change(
    mut commands: Commands,
    mut season_events: EventReader<SeasonChangeEvent>,
    current_map: Res<super::CurrentMapId>,
    mut world_map: ResMut<super::WorldMap>,
    farm_state: Res<FarmState>,
    object_atlases: Res<ObjectAtlases>,
    existing_objects: Query<&WorldObjectData, With<WorldObject>>,
    existing_weeds: Query<&Weed>,
) {
    for event in season_events.read() {
        // Only regrow on the farm
        if current_map.map_id != MapId::Farm {
            continue;
        }

        // Collect occupied positions from existing world objects and weeds
        let mut occupied: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();
        for obj in existing_objects.iter() {
            occupied.insert((obj.grid_x, obj.grid_y));
        }
        for weed in existing_weeds.iter() {
            occupied.insert((weed.grid_x, weed.grid_y));
        }

        // Spawn 1-2 new trees
        let tree_count = 1 + (event.year as usize + event.new_season.index()) % 2;
        let farm_w = 20i32;
        let farm_h = 20i32;
        let mut spawned = 0;

        for attempt in 0..(tree_count * 15) {
            if spawned >= tree_count {
                break;
            }

            // Deterministic pseudo-random position
            let hash = (event.year as usize)
                .wrapping_mul(41)
                .wrapping_add(event.new_season.index().wrapping_mul(67))
                .wrapping_add(attempt.wrapping_mul(29));
            let x = (hash % farm_w as usize) as i32;
            let y = (((hash / farm_w as usize).wrapping_mul(23).wrapping_add(attempt * 7)) % farm_h as usize) as i32;

            // Skip solid, cropped, tilled, or occupied tiles
            if world_map.is_solid(x, y) {
                continue;
            }
            if farm_state.crops.contains_key(&(x, y)) {
                continue;
            }
            if farm_state.soil.contains_key(&(x, y)) {
                continue;
            }
            if occupied.contains(&(x, y)) {
                continue;
            }

            // Spawn a new tree
            let kind = WorldObjectKind::Tree;
            let data = WorldObjectData {
                kind,
                health: kind.max_health(),
                max_health: kind.max_health(),
                grid_x: x,
                grid_y: y,
            };

            let size = kind.sprite_size();
            let y_offset = if size.y > TILE_SIZE {
                (size.y - TILE_SIZE) / 2.0
            } else {
                0.0
            };

            if object_atlases.loaded {
                let atlas_index = kind.atlas_index();
                let mut sprite = Sprite::from_atlas_image(
                    object_atlases.grass_biome_image.clone(),
                    TextureAtlas {
                        layout: object_atlases.grass_biome_layout.clone(),
                        index: atlas_index,
                    },
                );
                sprite.custom_size = Some(size);

                let rwc = grid_to_world_center(x, y);
                commands.spawn((
                    sprite,
                    Transform::from_translation(Vec3::new(
                        rwc.x,
                        rwc.y + y_offset,
                        Z_ENTITY_BASE,
                    )),
                    WorldObject,
                    YSorted,
                    data,
                ));
            } else {
                let rwc = grid_to_world_center(x, y);
                commands.spawn((
                    Sprite {
                        color: kind.color(),
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(
                        rwc.x,
                        rwc.y + y_offset,
                        Z_ENTITY_BASE,
                    )),
                    WorldObject,
                    YSorted,
                    data,
                ));
            }

            // Mark the tile as solid
            world_map.set_solid(x, y, true);
            occupied.insert((x, y));
            spawned += 1;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTERACTABLE OBJECT SPAWNING
// ═══════════════════════════════════════════════════════════════════════

/// Spawns the shipping bin on the Farm map at grid (14, 6).
/// Only spawns if the player is on the Farm map and the bin hasn't been spawned yet.
pub fn spawn_shipping_bin(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    query: Query<Entity, With<ShippingBinMarker>>,
    furniture: Res<FurnitureAtlases>,
) {
    if player_state.current_map != MapId::Farm || !query.is_empty() {
        return;
    }
    let wc = grid_to_world_center(14, 6);
    let sprite = if furniture.loaded {
        let mut s = Sprite::from_atlas_image(
            furniture.image.clone(),
            TextureAtlas {
                layout: furniture.layout.clone(),
                index: 18,
            },
        );
        s.custom_size = Some(Vec2::splat(TILE_SIZE));
        s
    } else {
        Sprite {
            color: Color::srgb(0.55, 0.35, 0.15),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        }
    };
    commands.spawn((
        ShippingBinMarker,
        WorldObject,
        Interactable {
            kind: InteractionKind::ShippingBin,
            label: "Ship Items".into(),
        },
        sprite,
        Transform::from_translation(Vec3::new(wc.x, wc.y, Z_ENTITY_BASE)),
        YSorted,
        Visibility::default(),
    ));
}

/// Spawns the crafting bench on the Farm map at grid (12, 6).
/// Only spawns if the player is on the Farm map and the bench hasn't been spawned yet.
pub fn spawn_crafting_bench(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    query: Query<Entity, With<CraftingBenchMarker>>,
    furniture: Res<FurnitureAtlases>,
) {
    if player_state.current_map != MapId::Farm || !query.is_empty() {
        return;
    }
    let wc = grid_to_world_center(12, 6);
    let sprite = if furniture.loaded {
        let mut s = Sprite::from_atlas_image(
            furniture.image.clone(),
            TextureAtlas {
                layout: furniture.layout.clone(),
                index: 27,
            },
        );
        s.custom_size = Some(Vec2::splat(TILE_SIZE));
        s
    } else {
        Sprite {
            color: Color::srgb(0.6, 0.5, 0.3),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        }
    };
    commands.spawn((
        CraftingBenchMarker,
        WorldObject,
        Interactable {
            kind: InteractionKind::CraftingBench,
            label: "Crafting Bench".into(),
        },
        sprite,
        Transform::from_translation(Vec3::new(wc.x, wc.y, Z_ENTITY_BASE)),
        YSorted,
        Visibility::default(),
    ));
}

/// Spawns the carpenter board on the Town map at grid (10, 8).
/// Only spawns if the player is on the Town map and the board hasn't been spawned yet.
pub fn spawn_carpenter_board(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    query: Query<Entity, With<CarpenterBoardMarker>>,
    furniture: Res<FurnitureAtlases>,
) {
    if player_state.current_map != MapId::Town || !query.is_empty() {
        return;
    }
    let wc = grid_to_world_center(10, 8);
    let sprite = if furniture.loaded {
        let mut s = Sprite::from_atlas_image(
            furniture.image.clone(),
            TextureAtlas {
                layout: furniture.layout.clone(),
                index: 20,
            },
        );
        s.custom_size = Some(Vec2::splat(TILE_SIZE));
        s
    } else {
        Sprite {
            color: Color::srgb(0.65, 0.55, 0.35),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        }
    };
    commands.spawn((
        CarpenterBoardMarker,
        WorldObject,
        Interactable {
            kind: InteractionKind::BuildingUpgrade,
            label: "Building Upgrades".into(),
        },
        sprite,
        Transform::from_translation(Vec3::new(wc.x, wc.y, Z_ENTITY_BASE)),
        YSorted,
        Visibility::default(),
    ));
}

// ═══════════════════════════════════════════════════════════════════════
// BUILDING ENTRANCE SIGNS — floating labels above shop/building doors
// ═══════════════════════════════════════════════════════════════════════

/// Marker for building sign entities (despawned on map change).
#[derive(Component)]
pub struct BuildingSign;

/// Building entrance definitions: (grid_x, grid_y, label).
const BUILDING_SIGNS: &[(i32, i32, &str)] = &[
    (9,  5, "General Store"),
    (39, 5, "Animal Shop"),
    (42, 31, "Blacksmith"),
];

/// Spawn floating text labels above building entrances on the town map.
pub fn spawn_building_signs(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    existing: Query<Entity, With<BuildingSign>>,
) {
    if player_state.current_map != MapId::Town || !existing.is_empty() {
        return;
    }
    for &(gx, gy, label) in BUILDING_SIGNS {
        let wc = grid_to_world_center(gx, gy);
        commands.spawn((
            BuildingSign,
            WorldObject,
            Text2d::new(label.to_string()),
            TextFont {
                font_size: 5.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.95, 0.7)),
            Transform::from_xyz(wc.x, wc.y + TILE_SIZE * 0.8, Z_GROUND + 4.0),
            Visibility::default(),
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// BUILDING SPRITES — render Sprout Lands house walls, roofs, and doors
// ═══════════════════════════════════════════════════════════════════════

/// Marker for building overlay sprite entities (despawned on map change).
#[derive(Component)]
pub struct BuildingOverlay;

/// Definition for a building to render on a map.
struct BuildingDef {
    /// Top-left grid position of the building footprint.
    x: i32,
    y: i32,
    /// Width and height in tiles.
    w: i32,
    h: i32,
    /// Grid position of the door (relative to map).
    door_x: i32,
    door_y: i32,
    /// Roof color tint to differentiate buildings.
    roof_tint: Color,
}

/// Town building definitions.
fn town_buildings() -> Vec<BuildingDef> {
    vec![
        // General Store (north-west)
        BuildingDef {
            x: 2, y: 2, w: 8, h: 5,
            door_x: 5, door_y: 2,
            roof_tint: Color::srgb(0.85, 0.55, 0.4),
        },
        // Animal Shop (north-east)
        BuildingDef {
            x: 18, y: 2, w: 8, h: 5,
            door_x: 22, door_y: 2,
            roof_tint: Color::srgb(0.5, 0.7, 0.85),
        },
        // Blacksmith (east, below plaza)
        BuildingDef {
            x: 20, y: 13, w: 6, h: 4,
            door_x: 22, door_y: 13,
            roof_tint: Color::srgb(0.6, 0.55, 0.55),
        },
        // NPC House 1 (west, below plaza — doc/librarian area)
        BuildingDef {
            x: 2, y: 13, w: 5, h: 3,
            door_x: 3, door_y: 13,
            roof_tint: Color::srgb(0.75, 0.85, 0.6),
        },
        // NPC House 2 (center-west, below plaza — fisher/kid)
        BuildingDef {
            x: 8, y: 13, w: 5, h: 3,
            door_x: 9, door_y: 13,
            roof_tint: Color::srgb(0.85, 0.75, 0.55),
        },
    ]
}

fn farm_buildings() -> Vec<BuildingDef> {
    vec![
        // Player house (top center of farm)
        BuildingDef {
            x: 13, y: 0, w: 6, h: 3,
            door_x: 15, door_y: 0,
            roof_tint: Color::srgb(0.75, 0.5, 0.4),
        },
        // Chicken coop (bottom-left area)
        BuildingDef {
            x: 9, y: 17, w: 3, h: 2,
            door_x: 10, door_y: 17,
            roof_tint: Color::srgb(0.9, 0.8, 0.5),
        },
        // Barn (bottom-left area)
        BuildingDef {
            x: 3, y: 16, w: 5, h: 3,
            door_x: 5, door_y: 16,
            roof_tint: Color::srgb(0.7, 0.3, 0.3),
        },
    ]
}

/// Spawn multi-tile building overlays using Sprout Lands house tilesets.
/// Renders walls on the building footprint and roof tiles above.
pub fn spawn_building_sprites(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    existing: Query<Entity, With<BuildingOverlay>>,
    object_atlases: Res<ObjectAtlases>,
) {
    if !existing.is_empty() || !object_atlases.loaded {
        return;
    }

    let buildings = match player_state.current_map {
        MapId::Town => town_buildings(),
        MapId::Farm => farm_buildings(),
        _ => return,
    };

    for bld in &buildings {
        // --- WALLS: tile the building footprint ---
        // house_walls.png layout (5 cols x 3 rows) — opaque tiles only:
        //   [1]=wall, [3]=wall, [4]=wall, [6]=light, [8]=wall, [9]=wall, [11]=wall
        //   Transparent (skip): 0,2,5,7,10,12,13,14
        // We use opaque tiles: 8/9 for body, 1/3/4 for top row, 11 for bottom.
        for dy in 0..bld.h {
            for dx in 0..bld.w {
                let gx = bld.x + dx;
                let gy = bld.y + dy;

                // Pick wall tile — use only opaque indices
                let is_left = dx == 0;
                let is_right = dx == bld.w - 1;
                let is_top = dy == bld.h - 1; // highest Y = back wall
                let is_bottom = dy == 0;       // lowest Y = front wall

                let wall_index = match (is_top, is_bottom, is_left, is_right) {
                    (true, _, true, _) => 1,     // top-left: wall face
                    (true, _, _, true) => 4,     // top-right: wall face
                    (true, _, _, _) => 3,        // top center: wall face
                    (_, true, true, _) => 11,    // bottom-left: base
                    (_, true, _, true) => 9,     // bottom-right: wall
                    (_, true, _, _) => 6,        // bottom center: light wall
                    (_, _, true, _) => 1,        // left edge: wall face
                    (_, _, _, true) => 9,        // right edge: wall
                    _ => 8,                      // interior fill: wall
                };

                let wc = grid_to_world_center(gx, gy);
                let mut sprite = Sprite::from_atlas_image(
                    object_atlases.house_walls_image.clone(),
                    TextureAtlas {
                        layout: object_atlases.house_walls_layout.clone(),
                        index: wall_index,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

                commands.spawn((
                    BuildingOverlay,
                    WorldObject,
                    sprite,
                    Transform::from_xyz(wc.x, wc.y, Z_GROUND + 1.0),
                    Visibility::default(),
                ));
            }
        }

        // --- ROOF: span the width of the building, above the back wall ---
        // house_roof.png layout (7 cols x 5 rows) — opaque tiles:
        //   Row 2: [15=left, 17-20=body]  Row 3: [22=left, 24-27=body]
        //   Row 4: [31-34=eave]
        // We use eave (row 4) for the row closest to walls, body (row 3) for peak.
        let roof_rows = 2.min((bld.h + 1) / 2); // 1-2 roof rows
        for ry in 0..roof_rows {
            for dx in 0..bld.w {
                let gx = bld.x + dx;
                // Place roof BEHIND the building (higher Y = away from camera)
                let gy = bld.y + bld.h + ry;

                let is_left = dx == 0;
                let is_right = dx == bld.w - 1;

                let roof_index = if ry == 0 {
                    // Bottom roof row (eave, closest to building)
                    if is_left { 31 } else if is_right { 34 } else { 32 }
                } else {
                    // Top roof row (body/peak, furthest from building)
                    if is_left { 22 } else if is_right { 27 } else { 25 }
                };

                let wc = grid_to_world_center(gx, gy);
                let mut sprite = Sprite::from_atlas_image(
                    object_atlases.house_roof_image.clone(),
                    TextureAtlas {
                        layout: object_atlases.house_roof_layout.clone(),
                        index: roof_index,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(TILE_SIZE));
                sprite.color = bld.roof_tint;

                commands.spawn((
                    BuildingOverlay,
                    WorldObject,
                    sprite,
                    Transform::from_xyz(wc.x, wc.y, Z_GROUND + 3.0),
                    Visibility::default(),
                ));
            }
        }

        // --- DOOR: place at entrance ---
        // doors.png: 1 col x 4 rows. Opaque tiles: 1, 3. Use index 1.
        let dwc = grid_to_world_center(bld.door_x, bld.door_y);
        let mut door_sprite = Sprite::from_atlas_image(
            object_atlases.doors_image.clone(),
            TextureAtlas {
                layout: object_atlases.doors_layout.clone(),
                index: 1,
            },
        );
        door_sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

        commands.spawn((
            BuildingOverlay,
            WorldObject,
            door_sprite,
            Transform::from_xyz(dwc.x, dwc.y, Z_GROUND + 2.0),
            Visibility::default(),
        ));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INTERIOR DECORATIONS — furniture sprites inside buildings
// ═══════════════════════════════════════════════════════════════════════

/// Marker for interior decoration entities (despawned on map change).
#[derive(Component)]
pub struct InteriorDecoration;

/// A furniture placement: grid position + atlas index in furniture.png.
struct FurniturePlacement {
    x: i32,
    y: i32,
    index: usize,
    /// Optional: 2-tile wide item (spawns at x and x+1).
    wide: bool,
}

fn player_house_furniture() -> Vec<FurniturePlacement> {
    vec![
        // ── Bedroom (upper-right area) ──
        // Bed (2-tile wide)
        FurniturePlacement { x: 12, y: 2, index: 2, wide: false },
        FurniturePlacement { x: 13, y: 2, index: 3, wide: false },
        // Nightstand beside bed
        FurniturePlacement { x: 11, y: 2, index: 31, wide: false },
        // Dresser
        FurniturePlacement { x: 14, y: 4, index: 21, wide: false },
        // Bedroom rug under bed
        FurniturePlacement { x: 12, y: 3, index: 48, wide: false },
        FurniturePlacement { x: 13, y: 3, index: 49, wide: false },

        // ── Kitchen (upper-left area) ──
        // Counter along back wall
        FurniturePlacement { x: 2, y: 1, index: 32, wide: false },
        FurniturePlacement { x: 3, y: 1, index: 33, wide: false },
        FurniturePlacement { x: 4, y: 1, index: 32, wide: false },
        // Pantry barrel
        FurniturePlacement { x: 1, y: 3, index: 27, wide: false },
        // Kitchen table
        FurniturePlacement { x: 3, y: 4, index: 0, wide: false },
        FurniturePlacement { x: 4, y: 4, index: 1, wide: false },
        // Stools at table
        FurniturePlacement { x: 3, y: 5, index: 5, wide: false },
        FurniturePlacement { x: 4, y: 5, index: 5, wide: false },

        // ── Living Room (center) ──
        // Bookshelf on left wall (2-tile tall)
        FurniturePlacement { x: 1, y: 6, index: 9, wide: false },
        FurniturePlacement { x: 1, y: 7, index: 18, wide: false },
        // Chairs flanking rug
        FurniturePlacement { x: 5, y: 7, index: 4, wide: false },
        FurniturePlacement { x: 10, y: 7, index: 4, wide: false },
        // Lamp next to bookshelf
        FurniturePlacement { x: 1, y: 8, index: 24, wide: false },
        // Plant in corner
        FurniturePlacement { x: 14, y: 8, index: 22, wide: false },

        // ── Fireplace area ──
        // (Stone tiles handle the fireplace, but add decorative items)
        FurniturePlacement { x: 5, y: 1, index: 24, wide: false }, // lamp on mantel

        // ── Entry Area ──
        // Coat/supply barrel near door
        FurniturePlacement { x: 2, y: 13, index: 27, wide: false },
        // Crate near door
        FurniturePlacement { x: 13, y: 13, index: 36, wide: false },
        // Welcome mat (furniture over path tiles)
        FurniturePlacement { x: 7, y: 14, index: 48, wide: false },
        FurniturePlacement { x: 8, y: 14, index: 49, wide: false },
    ]
}

fn general_store_furniture() -> Vec<FurniturePlacement> {
    vec![
        // ── Behind counter (shopkeeper area, y=1-3) ──
        // Shelves along back wall
        FurniturePlacement { x: 2, y: 1, index: 9, wide: false },
        FurniturePlacement { x: 3, y: 1, index: 10, wide: false },
        FurniturePlacement { x: 4, y: 1, index: 11, wide: false },
        FurniturePlacement { x: 6, y: 1, index: 9, wide: false },
        FurniturePlacement { x: 7, y: 1, index: 10, wide: false },
        FurniturePlacement { x: 8, y: 1, index: 11, wide: false },
        // Back crates
        FurniturePlacement { x: 9, y: 2, index: 36, wide: false },

        // ── Counter surface (y=4, stone tiles underneath) ──
        FurniturePlacement { x: 4, y: 4, index: 32, wide: false },
        FurniturePlacement { x: 5, y: 4, index: 33, wide: false },
        FurniturePlacement { x: 7, y: 4, index: 32, wide: false },

        // ── Customer area displays ──
        // Left wall shelves
        FurniturePlacement { x: 1, y: 5, index: 9, wide: false },
        FurniturePlacement { x: 1, y: 6, index: 18, wide: false },
        FurniturePlacement { x: 1, y: 8, index: 9, wide: false },
        // Right wall shelves
        FurniturePlacement { x: 10, y: 5, index: 10, wide: false },
        FurniturePlacement { x: 10, y: 6, index: 18, wide: false },
        FurniturePlacement { x: 10, y: 8, index: 10, wide: false },

        // ── Entrance area ──
        // Barrel near door
        FurniturePlacement { x: 2, y: 9, index: 27, wide: false },
        // Crate near door
        FurniturePlacement { x: 9, y: 9, index: 36, wide: false },
        // Potted plant
        FurniturePlacement { x: 1, y: 4, index: 22, wide: false },
        FurniturePlacement { x: 10, y: 3, index: 22, wide: false },
    ]
}

fn blacksmith_furniture() -> Vec<FurniturePlacement> {
    vec![
        // ── Forge area (back-right, on dirt) ──
        FurniturePlacement { x: 7, y: 1, index: 18, wide: false },
        FurniturePlacement { x: 8, y: 1, index: 18, wide: false },
        FurniturePlacement { x: 9, y: 2, index: 27, wide: false }, // water barrel

        // ── Anvil workspace (center, on wood floor island) ──
        FurniturePlacement { x: 5, y: 6, index: 19, wide: false },

        // ── Counter / reception (y=4) ──
        FurniturePlacement { x: 3, y: 4, index: 32, wide: false },
        FurniturePlacement { x: 4, y: 4, index: 33, wide: false },
        FurniturePlacement { x: 5, y: 4, index: 32, wide: false },

        // ── Storage corner (back-left, on dirt) ──
        FurniturePlacement { x: 1, y: 1, index: 36, wide: false },
        FurniturePlacement { x: 2, y: 1, index: 37, wide: false },
        FurniturePlacement { x: 1, y: 2, index: 38, wide: false },
        FurniturePlacement { x: 3, y: 1, index: 36, wide: false },

        // ── Decorations ──
        FurniturePlacement { x: 1, y: 5, index: 24, wide: false }, // lamp
        FurniturePlacement { x: 10, y: 5, index: 27, wide: false }, // barrel
        FurniturePlacement { x: 10, y: 8, index: 36, wide: false }, // crate near door

        // ── Tool display on right wall ──
        FurniturePlacement { x: 10, y: 3, index: 9, wide: false },
    ]
}

fn animal_shop_furniture() -> Vec<FurniturePlacement> {
    vec![
        // ── Hay/feed storage (back-left, on dirt) ──
        FurniturePlacement { x: 1, y: 1, index: 45, wide: false },
        FurniturePlacement { x: 2, y: 1, index: 46, wide: false },
        FurniturePlacement { x: 3, y: 1, index: 47, wide: false },
        FurniturePlacement { x: 1, y: 2, index: 45, wide: false },
        FurniturePlacement { x: 2, y: 2, index: 27, wide: false }, // feed barrel

        // ── Counter (y=4, stone underneath) ──
        FurniturePlacement { x: 5, y: 4, index: 32, wide: false },
        FurniturePlacement { x: 6, y: 4, index: 33, wide: false },
        FurniturePlacement { x: 7, y: 4, index: 32, wide: false },

        // ── Back wall shelves (right side) ──
        FurniturePlacement { x: 6, y: 1, index: 10, wide: false },
        FurniturePlacement { x: 7, y: 1, index: 11, wide: false },
        FurniturePlacement { x: 8, y: 1, index: 10, wide: false },
        FurniturePlacement { x: 9, y: 1, index: 9, wide: false },

        // ── Customer area ──
        FurniturePlacement { x: 1, y: 6, index: 28, wide: false }, // barrel
        FurniturePlacement { x: 10, y: 6, index: 22, wide: false }, // plant
        FurniturePlacement { x: 10, y: 9, index: 36, wide: false }, // crate
        FurniturePlacement { x: 1, y: 9, index: 27, wide: false }, // barrel near door
    ]
}

pub fn spawn_interior_decorations(
    mut commands: Commands,
    player_state: Res<PlayerState>,
    existing: Query<Entity, With<InteriorDecoration>>,
    furniture: Res<FurnitureAtlases>,
) {
    if !existing.is_empty() || !furniture.loaded {
        return;
    }

    let placements = match player_state.current_map {
        MapId::PlayerHouse => player_house_furniture(),
        MapId::GeneralStore => general_store_furniture(),
        MapId::Blacksmith => blacksmith_furniture(),
        MapId::AnimalShop => animal_shop_furniture(),
        _ => return,
    };

    for fp in &placements {
        let wc = grid_to_world_center(fp.x, fp.y);
        let mut sprite = Sprite::from_atlas_image(
            furniture.image.clone(),
            TextureAtlas {
                layout: furniture.layout.clone(),
                index: fp.index,
            },
        );
        sprite.custom_size = Some(Vec2::splat(TILE_SIZE));

        commands.spawn((
            InteriorDecoration,
            WorldObject,
            sprite,
            Transform::from_xyz(wc.x, wc.y, Z_ENTITY_BASE),
            YSorted,
            Visibility::default(),
        ));

        // Wide furniture: spawn second tile
        if fp.wide {
            let wc2 = grid_to_world_center(fp.x + 1, fp.y);
            let mut sprite2 = Sprite::from_atlas_image(
                furniture.image.clone(),
                TextureAtlas {
                    layout: furniture.layout.clone(),
                    index: fp.index + 1,
                },
            );
            sprite2.custom_size = Some(Vec2::splat(TILE_SIZE));

            commands.spawn((
                InteriorDecoration,
                WorldObject,
                sprite2,
                Transform::from_xyz(wc2.x, wc2.y, Z_ENTITY_BASE),
                YSorted,
                Visibility::default(),
            ));
        }
    }
}
