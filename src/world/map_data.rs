//! Data-driven map definitions loaded from RON files.
//!
//! Each map can be defined in `assets/maps/{map_id}.ron` as a `MapData` struct.
//! At startup these are loaded into a `MapRegistry` resource.  The runtime
//! `MapDef` type used by the renderer is produced via `map_data_to_map_def()`.

use crate::shared::*;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::maps::{MapDef, ObjectPlacement, WorldObjectKind};

// ═══════════════════════════════════════════════════════════════════════
// SERIALIZABLE MAP TYPES
// ═══════════════════════════════════════════════════════════════════════

/// A complete, RON-serializable description of a game map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub id: MapId,
    pub width: usize,
    pub height: usize,
    /// Row-major tile grid: `tiles[y * width + x]`.
    pub tiles: Vec<TileKind>,
    /// World objects (trees, rocks, stumps, etc.).
    pub objects: Vec<ObjectDef>,
    /// Forageable spawn positions.
    pub forage_points: Vec<(i32, i32)>,
    /// Default player spawn position for this map.
    pub spawn_pos: (i32, i32),
    /// Zone-based transitions (walking onto `from_rect` warps to target).
    pub transitions: Vec<TransitionDef>,
    /// Door triggers (walking onto x_min..=x_max at y warps to interior).
    pub doors: Vec<DoorDef>,
    /// Edge transitions: which maps border this one on each edge.
    pub edges: EdgeDefs,
    /// Building visual definitions for this map.
    pub buildings: Vec<BuildingDataDef>,
}

/// An object placed on the map at load time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDef {
    pub x: i32,
    pub y: i32,
    pub kind: WorldObjectKind,
}

/// A zone-based transition: walking onto `from_rect` teleports the player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDef {
    /// (x, y, w, h) trigger rectangle.
    pub from_rect: (i32, i32, i32, i32),
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

/// A door trigger: walking onto `x_min..=x_max` at `y` warps to an interior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorDef {
    pub x_min: i32,
    pub x_max: i32,
    pub y: i32,
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

/// Describes how to position the player when transitioning via an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeTarget {
    /// Clamp player X to target map width; Y is fixed.
    ClampX(i32),
    /// Clamp player Y to target map height; X is fixed.
    ClampY(i32),
    /// Fixed spawn position.
    Fixed(i32, i32),
}

/// Which maps border this one on each cardinal edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefs {
    pub north: Option<(MapId, EdgeTarget)>,
    pub south: Option<(MapId, EdgeTarget)>,
    pub east: Option<(MapId, EdgeTarget)>,
    pub west: Option<(MapId, EdgeTarget)>,
}

/// A building footprint for visual rendering (stored in RON).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDataDef {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub roof_tint: (f32, f32, f32),
}

// ═══════════════════════════════════════════════════════════════════════
// MAP REGISTRY
// ═══════════════════════════════════════════════════════════════════════

/// Resource holding all loaded map data, keyed by `MapId`.
#[derive(Resource, Debug, Clone, Default)]
pub struct MapRegistry {
    pub maps: HashMap<MapId, MapData>,
}

// ═══════════════════════════════════════════════════════════════════════
// CONVERSION: MapData → MapDef
// ═══════════════════════════════════════════════════════════════════════

/// Convert a data-driven `MapData` into the runtime `MapDef` used by the
/// renderer and collision system.
pub fn map_data_to_map_def(data: &MapData) -> MapDef {
    let transitions = data
        .transitions
        .iter()
        .map(|t| MapTransition {
            from_map: data.id,
            from_rect: t.from_rect,
            to_map: t.to_map,
            to_pos: (t.to_x, t.to_y),
        })
        .collect();

    let objects = data
        .objects
        .iter()
        .map(|o| ObjectPlacement {
            x: o.x,
            y: o.y,
            kind: o.kind,
        })
        .collect();

    MapDef {
        id: data.id,
        width: data.width,
        height: data.height,
        tiles: data.tiles.clone(),
        transitions,
        objects,
        forage_points: data.forage_points.clone(),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// LOADER
// ═══════════════════════════════════════════════════════════════════════

/// Attempt to load a `MapData` from `assets/maps/{map_id}.ron`.
/// Returns `None` if the file doesn't exist or fails to parse.
pub fn load_map_data(map_id: MapId) -> Option<MapData> {
    let name = map_id_filename(map_id);
    let path = format!("assets/maps/{}.ron", name);
    let contents = std::fs::read_to_string(&path).ok()?;
    match ron::from_str::<MapData>(&contents) {
        Ok(data) => Some(data),
        Err(e) => {
            bevy::log::warn!("Failed to parse {}: {}", path, e);
            None
        }
    }
}

/// Build a `MapRegistry` by loading all maps from RON files (with
/// hardcoded fallback).
pub fn build_map_registry() -> MapRegistry {
    use super::maps::{default_spawn_position, generate_map};

    let all_maps = [
        MapId::Farm,
        MapId::Town,
        MapId::TownWest,
        MapId::Beach,
        MapId::Forest,
        MapId::DeepForest,
        MapId::MineEntrance,
        MapId::Mine,
        MapId::PlayerHouse,
        MapId::TownHouseWest,
        MapId::TownHouseEast,
        MapId::GeneralStore,
        MapId::AnimalShop,
        MapId::Blacksmith,
        MapId::Library,
        MapId::Tavern,
        MapId::CoralIsland,
        MapId::SnowMountain,
    ];

    let mut registry = MapRegistry {
        maps: HashMap::new(),
    };

    for &map_id in &all_maps {
        let data = load_map_data(map_id).unwrap_or_else(|| {
            // Fallback: convert hardcoded generator to MapData
            let map_def = generate_map(map_id);
            let spawn = default_spawn_position(map_id);
            let mut data = map_def_to_map_data(&map_def, spawn);
            data.doors = doors_for(map_id);
            data.edges = edges_for(map_id);
            data.buildings = buildings_for(map_id);
            data
        });
        registry.maps.insert(map_id, data);
    }

    registry
}

// ═══════════════════════════════════════════════════════════════════════
// EXPORT: MapDef → MapData (for generating RON files)
// ═══════════════════════════════════════════════════════════════════════

/// Convert a runtime `MapDef` into a serializable `MapData`.
/// Used for exporting existing hardcoded maps to RON.
pub fn map_def_to_map_data(map_def: &MapDef, spawn_pos: (i32, i32)) -> MapData {
    let objects = map_def
        .objects
        .iter()
        .map(|o| ObjectDef {
            x: o.x,
            y: o.y,
            kind: o.kind,
        })
        .collect();

    let transitions = map_def
        .transitions
        .iter()
        .map(|t| TransitionDef {
            from_rect: t.from_rect,
            to_map: t.to_map,
            to_x: t.to_pos.0,
            to_y: t.to_pos.1,
        })
        .collect();

    // Doors and edges are extracted from the hardcoded edge_transition()
    // logic separately — we fill them in during export.
    MapData {
        id: map_def.id,
        width: map_def.width,
        height: map_def.height,
        tiles: map_def.tiles.clone(),
        objects,
        forage_points: map_def.forage_points.clone(),
        spawn_pos,
        transitions,
        doors: Vec::new(),
        edges: EdgeDefs {
            north: None,
            south: None,
            east: None,
            west: None,
        },
        buildings: Vec::new(),
    }
}

/// Export all hardcoded maps to `assets/maps/*.ron`.
/// Populates doors, edges, and buildings from the hardcoded logic.
#[cfg(test)]
pub fn export_all_maps() -> Vec<(MapId, MapData)> {
    use super::maps::{default_spawn_position, generate_map};

    let all_maps = [
        MapId::Farm,
        MapId::Town,
        MapId::TownWest,
        MapId::Beach,
        MapId::Forest,
        MapId::DeepForest,
        MapId::MineEntrance,
        MapId::Mine,
        MapId::PlayerHouse,
        MapId::TownHouseWest,
        MapId::TownHouseEast,
        MapId::GeneralStore,
        MapId::AnimalShop,
        MapId::Blacksmith,
        MapId::Library,
        MapId::Tavern,
        MapId::CoralIsland,
        MapId::SnowMountain,
    ];

    let mut results = Vec::new();

    for &map_id in &all_maps {
        let map_def = generate_map(map_id);
        let spawn = default_spawn_position(map_id);
        let mut data = map_def_to_map_data(&map_def, spawn);

        // Populate doors, edges, and buildings from hardcoded data.
        data.doors = doors_for(map_id);
        data.edges = edges_for(map_id);
        data.buildings = buildings_for(map_id);

        results.push((map_id, data));
    }

    results
}

/// Write all exported maps to `assets/maps/` as RON files.
#[cfg(test)]
pub fn write_all_ron_files() -> std::io::Result<()> {
    std::fs::create_dir_all("assets/maps")?;

    for (map_id, data) in export_all_maps() {
        let name = map_id_filename(map_id);
        let path = format!("assets/maps/{}.ron", name);
        let config = ron::ser::PrettyConfig::new()
            .depth_limit(4)
            .separate_tuple_members(false)
            .enumerate_arrays(false);
        let ron_str =
            ron::ser::to_string_pretty(&data, config).expect("Failed to serialize map data");
        std::fs::write(&path, ron_str)?;
    }

    Ok(())
}

// ── Per-map door definitions (from hardcoded edge_transition) ──────────

fn doors_for(map_id: MapId) -> Vec<DoorDef> {
    match map_id {
        MapId::Farm => vec![DoorDef {
            x_min: 15,
            x_max: 16,
            y: 2,
            to_map: MapId::PlayerHouse,
            to_x: 8,
            to_y: 14,
        }],
        MapId::Town => vec![
            DoorDef {
                x_min: 5,
                x_max: 6,
                y: 2,
                to_map: MapId::GeneralStore,
                to_x: 6,
                to_y: 10,
            },
            DoorDef {
                x_min: 22,
                x_max: 23,
                y: 2,
                to_map: MapId::AnimalShop,
                to_x: 6,
                to_y: 10,
            },
            DoorDef {
                x_min: 22,
                x_max: 23,
                y: 13,
                to_map: MapId::Blacksmith,
                to_x: 6,
                to_y: 10,
            },
            DoorDef {
                x_min: 8,
                x_max: 9,
                y: 17,
                to_map: MapId::Library,
                to_x: 7,
                to_y: 10,
            },
            DoorDef {
                x_min: 15,
                x_max: 16,
                y: 17,
                to_map: MapId::Tavern,
                to_x: 8,
                to_y: 12,
            },
        ],
        MapId::TownWest => vec![
            DoorDef {
                x_min: 3,
                x_max: 4,
                y: 13,
                to_map: MapId::TownHouseWest,
                to_x: 6,
                to_y: 10,
            },
            DoorDef {
                x_min: 9,
                x_max: 10,
                y: 13,
                to_map: MapId::TownHouseEast,
                to_x: 6,
                to_y: 10,
            },
        ],
        MapId::MineEntrance => vec![
            // Cave mouth at the end of the entrance path → Mine floor 1.
            DoorDef {
                x_min: 6,
                x_max: 7,
                y: 3,
                to_map: MapId::Mine,
                to_x: 8,
                to_y: 14,
            },
        ],
        _ => Vec::new(),
    }
}

// ── Per-map edge definitions (from hardcoded edge_transition) ──────────

fn edges_for(map_id: MapId) -> EdgeDefs {
    match map_id {
        MapId::Farm => EdgeDefs {
            north: Some((MapId::SnowMountain, EdgeTarget::ClampX(1))),
            south: Some((MapId::Town, EdgeTarget::ClampX(20))),
            east: Some((MapId::Forest, EdgeTarget::ClampY(1))),
            west: Some((MapId::MineEntrance, EdgeTarget::Fixed(12, 6))),
        },
        MapId::Town => EdgeDefs {
            north: Some((MapId::Farm, EdgeTarget::Fixed(15, 20))),
            south: Some((MapId::Beach, EdgeTarget::ClampX(7))),
            east: Some((MapId::Forest, EdgeTarget::ClampY(1))),
            west: Some((MapId::TownWest, EdgeTarget::ClampY(14))),
        },
        MapId::TownWest => EdgeDefs {
            north: None,
            south: None,
            east: Some((MapId::Town, EdgeTarget::ClampY(1))),
            west: None,
        },
        MapId::Beach => EdgeDefs {
            north: Some((MapId::Town, EdgeTarget::ClampX(1))),
            south: Some((MapId::CoralIsland, EdgeTarget::Fixed(15, 1))),
            east: Some((MapId::Farm, EdgeTarget::ClampY(1))),
            west: None,
        },
        MapId::Forest => EdgeDefs {
            north: Some((MapId::MineEntrance, EdgeTarget::Fixed(7, 1))),
            south: None,
            east: Some((MapId::DeepForest, EdgeTarget::ClampY(1))),
            west: Some((MapId::Farm, EdgeTarget::Fixed(30, 10))),
        },
        MapId::DeepForest => EdgeDefs {
            north: None,
            south: None,
            east: None,
            west: Some((MapId::Forest, EdgeTarget::ClampY(20))),
        },
        MapId::MineEntrance => EdgeDefs {
            north: Some((MapId::SnowMountain, EdgeTarget::Fixed(5, 12))),
            south: Some((MapId::Forest, EdgeTarget::Fixed(11, 16))),
            east: Some((MapId::Farm, EdgeTarget::Fixed(1, 10))),
            west: None,
        },
        MapId::PlayerHouse => EdgeDefs {
            north: Some((MapId::Farm, EdgeTarget::Fixed(16, 3))),
            south: None,
            east: None,
            west: None,
        },
        MapId::TownHouseWest => EdgeDefs {
            north: Some((MapId::TownWest, EdgeTarget::Fixed(3, 14))),
            south: None,
            east: None,
            west: None,
        },
        MapId::TownHouseEast => EdgeDefs {
            north: Some((MapId::TownWest, EdgeTarget::Fixed(9, 14))),
            south: None,
            east: None,
            west: None,
        },
        MapId::GeneralStore => EdgeDefs {
            north: Some((MapId::Town, EdgeTarget::Fixed(6, 8))),
            south: None,
            east: None,
            west: None,
        },
        MapId::AnimalShop => EdgeDefs {
            north: Some((MapId::Town, EdgeTarget::Fixed(22, 8))),
            south: None,
            east: None,
            west: None,
        },
        MapId::Blacksmith => EdgeDefs {
            north: Some((MapId::Town, EdgeTarget::Fixed(22, 18))),
            south: None,
            east: None,
            west: None,
        },
        MapId::Library => EdgeDefs {
            north: Some((MapId::Town, EdgeTarget::Fixed(8, 18))),
            south: None,
            east: None,
            west: None,
        },
        MapId::Tavern => EdgeDefs {
            north: Some((MapId::Town, EdgeTarget::Fixed(16, 18))),
            south: None,
            east: None,
            west: None,
        },
        MapId::Mine => EdgeDefs {
            north: None,
            south: None,
            east: None,
            west: None,
        },
        MapId::CoralIsland => EdgeDefs {
            north: Some((MapId::Beach, EdgeTarget::Fixed(10, 12))),
            south: None,
            east: None,
            west: None,
        },
        MapId::SnowMountain => EdgeDefs {
            north: None,
            south: Some((MapId::Farm, EdgeTarget::ClampX(22))),
            east: None,
            west: None,
        },
    }
}

// ── Per-map building definitions ───────────────────────────────────────

fn buildings_for(map_id: MapId) -> Vec<BuildingDataDef> {
    match map_id {
        MapId::Farm => vec![
            BuildingDataDef {
                x: 13,
                y: 0,
                w: 6,
                h: 3,
                roof_tint: (0.75, 0.5, 0.4),
            },
            BuildingDataDef {
                x: 9,
                y: 17,
                w: 3,
                h: 2,
                roof_tint: (0.9, 0.8, 0.5),
            },
            BuildingDataDef {
                x: 3,
                y: 16,
                w: 5,
                h: 3,
                roof_tint: (0.7, 0.3, 0.3),
            },
        ],
        MapId::Town => vec![
            BuildingDataDef {
                x: 2,
                y: 2,
                w: 8,
                h: 5,
                roof_tint: (0.85, 0.55, 0.4),
            },
            BuildingDataDef {
                x: 18,
                y: 2,
                w: 8,
                h: 5,
                roof_tint: (0.5, 0.7, 0.85),
            },
            BuildingDataDef {
                x: 20,
                y: 13,
                w: 6,
                h: 4,
                roof_tint: (0.6, 0.55, 0.55),
            },
        ],
        MapId::TownWest => vec![
            BuildingDataDef {
                x: 2,
                y: 13,
                w: 5,
                h: 3,
                roof_tint: (0.75, 0.85, 0.6),
            },
            BuildingDataDef {
                x: 8,
                y: 13,
                w: 5,
                h: 3,
                roof_tint: (0.85, 0.75, 0.55),
            },
        ],
        _ => Vec::new(),
    }
}

/// Map a `MapId` to its lowercase filename (without extension).
pub fn map_id_filename(map_id: MapId) -> &'static str {
    match map_id {
        MapId::Farm => "farm",
        MapId::Town => "town",
        MapId::TownWest => "town_west",
        MapId::Beach => "beach",
        MapId::Forest => "forest",
        MapId::DeepForest => "deep_forest",
        MapId::MineEntrance => "mine_entrance",
        MapId::Mine => "mine",
        MapId::PlayerHouse => "player_house",
        MapId::TownHouseWest => "town_house_west",
        MapId::TownHouseEast => "town_house_east",
        MapId::GeneralStore => "general_store",
        MapId::AnimalShop => "animal_shop",
        MapId::Blacksmith => "blacksmith",
        MapId::Library => "library",
        MapId::Tavern => "tavern",
        MapId::CoralIsland => "coral_island",
        MapId::SnowMountain => "snow_mountain",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_ron_files() {
        write_all_ron_files().expect("Failed to write RON files");
        // Verify all map files exist and round-trip correctly
        let all_maps = [
            MapId::Farm,
            MapId::Town,
            MapId::TownWest,
            MapId::Beach,
            MapId::Forest,
            MapId::DeepForest,
            MapId::MineEntrance,
            MapId::Mine,
            MapId::PlayerHouse,
            MapId::TownHouseWest,
            MapId::TownHouseEast,
            MapId::GeneralStore,
            MapId::AnimalShop,
            MapId::Blacksmith,
            MapId::Library,
            MapId::Tavern,
            MapId::CoralIsland,
            MapId::SnowMountain,
        ];
        for &map_id in &all_maps {
            let name = map_id_filename(map_id);
            let path = format!("assets/maps/{}.ron", name);
            let contents = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("Cannot read {}: {}", path, e));
            let data: MapData =
                ron::from_str(&contents).unwrap_or_else(|e| panic!("Cannot parse {}: {}", path, e));
            assert_eq!(data.id, map_id);
            assert!(data.width > 0);
            assert!(data.height > 0);
            assert_eq!(data.tiles.len(), data.width * data.height);
        }
    }

    #[test]
    fn map_data_round_trip_matches_hardcoded() {
        use super::super::maps::{default_spawn_position, generate_map};

        // Generate RON files first
        write_all_ron_files().expect("Failed to write RON files");

        let all_maps = [
            MapId::Farm,
            MapId::Town,
            MapId::TownWest,
            MapId::Beach,
            MapId::Forest,
            MapId::DeepForest,
            MapId::MineEntrance,
            MapId::Mine,
            MapId::PlayerHouse,
            MapId::TownHouseWest,
            MapId::TownHouseEast,
            MapId::GeneralStore,
            MapId::AnimalShop,
            MapId::Blacksmith,
            MapId::Library,
            MapId::Tavern,
            MapId::CoralIsland,
            MapId::SnowMountain,
        ];

        for &map_id in &all_maps {
            let hardcoded_def = generate_map(map_id);
            let loaded_data = load_map_data(map_id)
                .unwrap_or_else(|| panic!("Failed to load RON for {:?}", map_id));
            let loaded_def = map_data_to_map_def(&loaded_data);

            assert_eq!(
                hardcoded_def.id, loaded_def.id,
                "id mismatch for {:?}",
                map_id
            );
            assert_eq!(
                hardcoded_def.width, loaded_def.width,
                "width mismatch for {:?}",
                map_id
            );
            assert_eq!(
                hardcoded_def.height, loaded_def.height,
                "height mismatch for {:?}",
                map_id
            );
            assert_eq!(
                hardcoded_def.tiles, loaded_def.tiles,
                "tiles mismatch for {:?}",
                map_id
            );
            assert_eq!(
                hardcoded_def.objects.len(),
                loaded_def.objects.len(),
                "objects count mismatch for {:?}",
                map_id
            );
            assert_eq!(
                hardcoded_def.forage_points, loaded_def.forage_points,
                "forage_points mismatch for {:?}",
                map_id
            );
            assert_eq!(
                loaded_data.spawn_pos,
                default_spawn_position(map_id),
                "spawn_pos mismatch for {:?}",
                map_id
            );
        }
    }
}
