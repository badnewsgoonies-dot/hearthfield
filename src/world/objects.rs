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

            commands.spawn((
                sprite,
                Transform::from_translation(Vec3::new(
                    placement.x as f32 * TILE_SIZE,
                    placement.y as f32 * TILE_SIZE + y_offset,
                    Z_ENTITY_BASE,
                )),
                WorldObject,
                YSorted,
                data,
            ));
        } else {
            // Fallback: colored rectangle if atlases failed to load
            commands.spawn((
                Sprite {
                    color: kind.color(),
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    placement.x as f32 * TILE_SIZE,
                    placement.y as f32 * TILE_SIZE + y_offset,
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

                                commands.spawn((
                                    stump_sprite,
                                    Transform::from_translation(Vec3::new(
                                        obj_data.grid_x as f32 * TILE_SIZE,
                                        obj_data.grid_y as f32 * TILE_SIZE,
                                        Z_ENTITY_BASE,
                                    )),
                                    WorldObject,
                                    YSorted,
                                    stump_data,
                                ));
                            } else {
                                // Fallback: colored rectangle
                                commands.spawn((
                                    Sprite {
                                        color: WorldObjectKind::Stump.color(),
                                        custom_size: Some(WorldObjectKind::Stump.sprite_size()),
                                        ..default()
                                    },
                                    Transform::from_translation(Vec3::new(
                                        obj_data.grid_x as f32 * TILE_SIZE,
                                        obj_data.grid_y as f32 * TILE_SIZE,
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
            let wx = x as f32 * TILE_SIZE;
            let wy = y as f32 * TILE_SIZE;
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.25, 0.55, 0.2),
                    custom_size: Some(Vec2::new(TILE_SIZE * 0.5, TILE_SIZE * 0.5)),
                    ..default()
                },
                Transform::from_translation(Vec3::new(wx, wy, Z_ENTITY_BASE)),
                LogicalPosition(Vec2::new(wx, wy)),
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

                commands.spawn((
                    sprite,
                    Transform::from_translation(Vec3::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE + y_offset,
                        Z_ENTITY_BASE,
                    )),
                    WorldObject,
                    YSorted,
                    data,
                ));
            } else {
                commands.spawn((
                    Sprite {
                        color: kind.color(),
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_translation(Vec3::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE + y_offset,
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
) {
    if player_state.current_map != MapId::Farm || !query.is_empty() {
        return;
    }
    let (wx, wy) = crate::player::grid_to_world(14, 6);
    commands.spawn((
        ShippingBinMarker,
        WorldObject,
        Interactable {
            kind: InteractionKind::ShippingBin,
            label: "Ship Items".into(),
        },
        Sprite {
            color: Color::srgb(0.55, 0.35, 0.15),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(Vec3::new(wx, wy, Z_ENTITY_BASE)),
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
) {
    if player_state.current_map != MapId::Farm || !query.is_empty() {
        return;
    }
    let (wx, wy) = crate::player::grid_to_world(12, 6);
    commands.spawn((
        CraftingBenchMarker,
        WorldObject,
        Interactable {
            kind: InteractionKind::CraftingBench,
            label: "Crafting Bench".into(),
        },
        Sprite {
            color: Color::srgb(0.6, 0.5, 0.3),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(Vec3::new(wx, wy, Z_ENTITY_BASE)),
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
) {
    if player_state.current_map != MapId::Town || !query.is_empty() {
        return;
    }
    let (wx, wy) = crate::player::grid_to_world(10, 8);
    commands.spawn((
        CarpenterBoardMarker,
        WorldObject,
        Interactable {
            kind: InteractionKind::BuildingUpgrade,
            label: "Building Upgrades".into(),
        },
        Sprite {
            color: Color::srgb(0.65, 0.55, 0.35),
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        Transform::from_translation(Vec3::new(wx, wy, Z_ENTITY_BASE)),
        YSorted,
        Visibility::default(),
    ));
}
