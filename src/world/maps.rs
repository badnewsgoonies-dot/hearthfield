//! Map data definitions for all game areas.
//!
//! Each map is defined as a 2D grid of TileKind values.
//! Maps also include transition zones and object spawn points.

use crate::shared::*;
use serde::{Deserialize, Serialize};

/// Complete definition of a game map.
#[derive(Debug, Clone)]
pub struct MapDef {
    pub id: MapId,
    pub width: usize,
    pub height: usize,
    /// Row-major tile data: tiles[y * width + x]
    pub tiles: Vec<TileKind>,
    /// Transition zones linking to other maps.
    /// Transition zone data for documentation purposes.
    /// NOTE: Actual transitions are handled by `edge_transition()` in
    /// `src/player/interaction.rs`, not by these definitions.
    pub transitions: Vec<MapTransition>,
    /// Initial world object placements (trees, rocks, etc.)
    pub objects: Vec<ObjectPlacement>,
    /// Forageable spawn points (grid positions).
    pub forage_points: Vec<(i32, i32)>,
}

impl MapDef {
    pub fn get_tile(&self, x: i32, y: i32) -> TileKind {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            TileKind::Void
        } else {
            self.tiles[y as usize * self.width + x as usize]
        }
    }
}

/// Describes an object placed on the map at load time.
#[derive(Debug, Clone)]
pub struct ObjectPlacement {
    pub x: i32,
    pub y: i32,
    pub kind: WorldObjectKind,
}

/// The types of world objects that can exist on maps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldObjectKind {
    Tree,
    Pine,
    Rock,
    Stump,
    Bush,
    LargeRock,
    Log,
    Dock,
    PalmTree,
    Coral,
    Driftwood,
}

// ═══════════════════════════════════════════════════════════════════════
// DEFAULT SPAWN POSITIONS
// ═══════════════════════════════════════════════════════════════════════

/// Returns a safe default spawn position for each map (e.g., for cutscene
/// teleports or fallback positioning).
pub fn default_spawn_position(map_id: MapId) -> (i32, i32) {
    match map_id {
        MapId::Farm => (16, 12),
        MapId::Town => (12, 8),
        MapId::TownWest => (12, 14),
        MapId::Beach => (10, 6),
        MapId::Forest => (8, 8),
        MapId::DeepForest => (3, 15),
        MapId::MineEntrance => (7, 6),
        MapId::Mine => (12, 12),
        MapId::PlayerHouse => (8, 8),
        MapId::TownHouseWest => (6, 8),
        MapId::TownHouseEast => (6, 8),
        MapId::GeneralStore => (6, 8),
        MapId::AnimalShop => (6, 8),
        MapId::Blacksmith => (6, 8),
        MapId::Library => (7, 10),
        MapId::Tavern => (8, 12),
        MapId::CoralIsland => (15, 1),
        MapId::SnowMountain => (16, 22),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MAP GENERATORS
// ═══════════════════════════════════════════════════════════════════════

pub fn generate_map(map_id: MapId) -> MapDef {
    match map_id {
        MapId::Farm => generate_farm(),
        MapId::Town => generate_town(),
        MapId::TownWest => generate_town_west(),
        MapId::Beach => generate_beach(),
        MapId::Forest => generate_forest(),
        MapId::DeepForest => generate_deep_forest(),
        MapId::MineEntrance => generate_mine_entrance(),
        MapId::Mine => generate_mine_floor(),
        MapId::PlayerHouse => generate_player_house(),
        MapId::TownHouseWest => generate_town_house_west(),
        MapId::TownHouseEast => generate_town_house_east(),
        MapId::GeneralStore => generate_general_store(),
        MapId::AnimalShop => generate_animal_shop(),
        MapId::Blacksmith => generate_blacksmith(),
        MapId::Library => super::map_data::load_map_data(MapId::Library)
            .map(|data| super::map_data::map_data_to_map_def(&data))
            .unwrap_or_else(generate_town_house_west),
        MapId::Tavern => super::map_data::load_map_data(MapId::Tavern)
            .map(|data| super::map_data::map_data_to_map_def(&data))
            .unwrap_or_else(generate_general_store),
        MapId::CoralIsland => generate_coral_island(),
        MapId::SnowMountain => super::map_data::load_map_data(MapId::SnowMountain)
            .map(|data| super::map_data::map_data_to_map_def(&data))
            .unwrap_or_else(generate_snow_mountain),
    }
}

// ---------------------------------------------------------------------------
// Farm map: 32x24  (Harvest Moon scale — ~3.5 screens)
// Layout: house at top-center, dirt field center, barn/coop bottom-left,
// pond bottom-right, exits on edges
// ---------------------------------------------------------------------------
fn generate_farm() -> MapDef {
    let w = 32usize;
    let h = 24usize;
    let mut tiles = vec![TileKind::Grass; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Player house footprint (top center)
    fill_rect(&mut tiles, 13, 0, 6, 3, TileKind::Stone);
    // Path from house down to fields
    fill_rect(&mut tiles, 15, 3, 2, 3, TileKind::Path);
    // Shipping bin area (right of house)
    fill_rect(&mut tiles, 20, 1, 2, 2, TileKind::WoodFloor);

    // Central farming area (the tillable field)
    fill_rect(&mut tiles, 6, 6, 20, 10, TileKind::Dirt);

    // Animal buildings (bottom-left)
    fill_rect(&mut tiles, 3, 16, 5, 3, TileKind::Stone); // Barn
    fill_rect(&mut tiles, 9, 17, 3, 2, TileKind::Stone); // Chicken coop
    fill_rect(&mut tiles, 8, 19, 4, 1, TileKind::Path); // Path connecting

    // Pond (bottom-right)
    fill_rect(&mut tiles, 24, 17, 5, 4, TileKind::Water);
    // Pond shoreline
    for dy in 0..6 {
        for dx in 0..7 {
            let xx = 23 + dx;
            let yy = 16 + dy;
            if xx < w && yy < h {
                let is_edge = dx == 0 || dy == 0 || dx == 6 || dy == 5;
                if is_edge && tiles[yy * w + xx] != TileKind::Water {
                    tiles[yy * w + xx] = TileKind::Sand;
                }
            }
        }
    }

    // Path west to mine
    fill_rect(&mut tiles, 0, 9, 6, 2, TileKind::Path);
    // Path east to forest
    fill_rect(&mut tiles, 26, 9, 6, 2, TileKind::Path);
    // Path south to town
    fill_rect(&mut tiles, 14, 16, 3, 8, TileKind::Path);

    let transitions = vec![
        // South exit -> Town
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (13, 23, 5, 1),
            to_map: MapId::Town,
            to_pos: (12, 1),
        },
        // East exit -> Forest
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (31, 8, 1, 4),
            to_map: MapId::Forest,
            to_pos: (1, 7),
        },
        // West exit -> Mine Entrance
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (0, 8, 1, 4),
            to_map: MapId::MineEntrance,
            to_pos: (12, 6),
        },
        // House entrance
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (15, 0, 2, 1),
            to_map: MapId::PlayerHouse,
            to_pos: (8, 14),
        },
    ];

    let mut objects = Vec::new();

    // Trees along top edge (around house)
    for x in (1..12).step_by(3) {
        objects.push(ObjectPlacement {
            x,
            y: 1,
            kind: WorldObjectKind::Tree,
        });
    }
    for x in (22..30).step_by(3) {
        objects.push(ObjectPlacement {
            x,
            y: 1,
            kind: WorldObjectKind::Tree,
        });
    }

    // Rocks in the dirt (player clears to farm)
    let rock_positions = [
        (8, 7),
        (12, 8),
        (18, 7),
        (22, 9),
        (10, 12),
        (16, 11),
        (20, 13),
        (14, 14),
        (24, 8),
    ];
    for (rx, ry) in &rock_positions {
        objects.push(ObjectPlacement {
            x: *rx,
            y: *ry,
            kind: WorldObjectKind::Rock,
        });
    }

    // Stumps
    objects.push(ObjectPlacement {
        x: 7,
        y: 14,
        kind: WorldObjectKind::Stump,
    });
    objects.push(ObjectPlacement {
        x: 25,
        y: 14,
        kind: WorldObjectKind::Stump,
    });

    // Bushes near pond
    objects.push(ObjectPlacement {
        x: 23,
        y: 17,
        kind: WorldObjectKind::Bush,
    });

    let forage_points = vec![(2, 4), (29, 3), (2, 20), (29, 20), (5, 22)];

    MapDef {
        id: MapId::Farm,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Town map: 28x22  (Harvest Moon scale — ~2.8 screens)
// Layout: shops top, plaza center, houses + blacksmith mid, park bottom
// ---------------------------------------------------------------------------
fn generate_town() -> MapDef {
    let w = 28usize;
    let h = 22usize;
    let mut tiles = vec![TileKind::Grass; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Main road N-S through center
    fill_rect(&mut tiles, 13, 0, 2, 22, TileKind::Path);
    // Main road E-W through middle
    fill_rect(&mut tiles, 0, 9, 28, 2, TileKind::Path);

    // Central plaza (stone square with fountain)
    fill_rect(&mut tiles, 11, 7, 6, 6, TileKind::Stone);
    fill_rect(&mut tiles, 13, 9, 2, 2, TileKind::Water); // Fountain

    // General Store (north-west)
    fill_rect(&mut tiles, 2, 2, 8, 5, TileKind::Stone);
    fill_rect(&mut tiles, 5, 7, 2, 2, TileKind::Path); // Path to plaza

    // Animal Shop (north-east)
    fill_rect(&mut tiles, 18, 2, 8, 5, TileKind::Stone);
    fill_rect(&mut tiles, 21, 7, 2, 2, TileKind::Path); // Path to plaza

    // Blacksmith (east, below plaza)
    fill_rect(&mut tiles, 20, 13, 6, 4, TileKind::Stone);
    fill_rect(&mut tiles, 17, 14, 3, 2, TileKind::Path); // Path to main road

    // Westbound residential lane
    fill_rect(&mut tiles, 0, 14, 13, 2, TileKind::Path);

    // Restaurant area (south center-east)
    fill_rect(&mut tiles, 14, 16, 5, 3, TileKind::Stone);

    // Park (south-west, small pond)
    fill_rect(&mut tiles, 2, 17, 8, 4, TileKind::Grass);
    fill_rect(&mut tiles, 4, 18, 3, 2, TileKind::Water);
    fill_rect(&mut tiles, 3, 18, 1, 2, TileKind::Sand);
    fill_rect(&mut tiles, 7, 18, 1, 2, TileKind::Sand);
    fill_rect(&mut tiles, 4, 17, 3, 1, TileKind::Sand);
    fill_rect(&mut tiles, 4, 20, 3, 1, TileKind::Sand);

    // Beach exit path (east edge)
    fill_rect(&mut tiles, 26, 11, 2, 2, TileKind::Path);

    let transitions = vec![
        // North exit -> Farm
        MapTransition {
            from_map: MapId::Town,
            from_rect: (12, 0, 4, 1),
            to_map: MapId::Farm,
            to_pos: (14, 22),
        },
        // General Store entrance
        MapTransition {
            from_map: MapId::Town,
            from_rect: (5, 2, 2, 1),
            to_map: MapId::GeneralStore,
            to_pos: (6, 10),
        },
        // Animal Shop entrance
        MapTransition {
            from_map: MapId::Town,
            from_rect: (22, 2, 2, 1),
            to_map: MapId::AnimalShop,
            to_pos: (6, 10),
        },
        // Blacksmith entrance
        MapTransition {
            from_map: MapId::Town,
            from_rect: (22, 13, 2, 1),
            to_map: MapId::Blacksmith,
            to_pos: (6, 10),
        },
        // West edge -> West Willowbrook
        MapTransition {
            from_map: MapId::Town,
            from_rect: (0, 13, 1, 4),
            to_map: MapId::TownWest,
            to_pos: (14, 14),
        },
        // East exit -> Beach
        MapTransition {
            from_map: MapId::Town,
            from_rect: (27, 10, 1, 4),
            to_map: MapId::Beach,
            to_pos: (1, 4),
        },
    ];

    let mut objects = Vec::new();
    // Trees along top
    for x in (0..12).step_by(3) {
        objects.push(ObjectPlacement {
            x,
            y: 0,
            kind: WorldObjectKind::Tree,
        });
    }
    for x in (16..28).step_by(3) {
        objects.push(ObjectPlacement {
            x,
            y: 0,
            kind: WorldObjectKind::Tree,
        });
    }
    // Park trees
    objects.push(ObjectPlacement {
        x: 2,
        y: 17,
        kind: WorldObjectKind::Tree,
    });
    objects.push(ObjectPlacement {
        x: 9,
        y: 17,
        kind: WorldObjectKind::Tree,
    });
    let forage_points = vec![(1, 19), (9, 20), (26, 19)];

    MapDef {
        id: MapId::Town,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// West Willowbrook: 16x22  outdoor residential district west of Town
// Layout: east-west lane, two homes, trees and small gardens
// ---------------------------------------------------------------------------
fn generate_town_west() -> MapDef {
    let w = 16usize;
    let h = 22usize;
    let mut tiles = vec![TileKind::Grass; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Main lane from Town into the residential district.
    fill_rect(&mut tiles, 0, 14, w, 2, TileKind::Path);

    // Yards and footpaths leading to each house.
    fill_rect(&mut tiles, 2, 13, 5, 3, TileKind::Stone);
    fill_rect(&mut tiles, 8, 13, 5, 3, TileKind::Stone);
    fill_rect(&mut tiles, 3, 13, 2, 3, TileKind::Path);
    fill_rect(&mut tiles, 9, 13, 2, 3, TileKind::Path);
    fill_rect(&mut tiles, 5, 9, 2, 5, TileKind::Path);
    fill_rect(&mut tiles, 10, 10, 2, 4, TileKind::Path);

    // Small clinic / study courtyards.
    fill_rect(&mut tiles, 3, 8, 2, 3, TileKind::Dirt);
    fill_rect(&mut tiles, 9, 8, 2, 2, TileKind::Dirt);

    let transitions = vec![
        MapTransition {
            from_map: MapId::TownWest,
            from_rect: (15, 13, 1, 4),
            to_map: MapId::Town,
            to_pos: (1, 14),
        },
        MapTransition {
            from_map: MapId::TownWest,
            from_rect: (3, 13, 2, 1),
            to_map: MapId::TownHouseWest,
            to_pos: (6, 10),
        },
        MapTransition {
            from_map: MapId::TownWest,
            from_rect: (9, 13, 2, 1),
            to_map: MapId::TownHouseEast,
            to_pos: (6, 10),
        },
    ];

    let objects = vec![
        ObjectPlacement {
            x: 1,
            y: 3,
            kind: WorldObjectKind::Tree,
        },
        ObjectPlacement {
            x: 4,
            y: 2,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 9,
            y: 3,
            kind: WorldObjectKind::Tree,
        },
        ObjectPlacement {
            x: 12,
            y: 4,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 2,
            y: 19,
            kind: WorldObjectKind::Tree,
        },
        ObjectPlacement {
            x: 11,
            y: 19,
            kind: WorldObjectKind::Tree,
        },
    ];

    let forage_points = vec![(2, 11), (12, 11), (6, 19)];

    MapDef {
        id: MapId::TownWest,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Beach map: 20x14  (Harvest Moon scale — ~1.3 screens)
// Layout: grass top, sand middle, ocean bottom, dock on right
// ---------------------------------------------------------------------------
fn generate_beach() -> MapDef {
    let w = 20usize;
    let h = 14usize;
    let mut tiles = vec![TileKind::Sand; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Ocean (bottom)
    for y in 9..h {
        for x in 0..w {
            let shore_y = 9.0 + 1.0 * (x as f32 * 0.4).sin();
            if y as f32 >= shore_y {
                tiles[y * w + x] = TileKind::Water;
            }
        }
    }

    // Grass strip (top)
    fill_rect(&mut tiles, 0, 0, 20, 3, TileKind::Grass);

    // Dock (right side, into water)
    fill_rect(&mut tiles, 15, 6, 2, 7, TileKind::WoodFloor);
    fill_rect(&mut tiles, 14, 11, 4, 2, TileKind::WoodFloor); // End platform

    // Path from town (left)
    fill_rect(&mut tiles, 0, 3, 4, 3, TileKind::Path);

    // Rocky area (top-right)
    fill_rect(&mut tiles, 16, 1, 3, 2, TileKind::Stone);

    // Tidal pools
    fill_rect(&mut tiles, 3, 8, 2, 1, TileKind::Water);
    fill_rect(&mut tiles, 9, 8, 1, 1, TileKind::Water);

    let transitions = vec![
        // West exit -> Town
        MapTransition {
            from_map: MapId::Beach,
            from_rect: (0, 3, 1, 3),
            to_map: MapId::Town,
            to_pos: (26, 11),
        },
    ];

    let objects = vec![
        ObjectPlacement {
            x: 17,
            y: 1,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 7,
            y: 7,
            kind: WorldObjectKind::Log,
        },
        ObjectPlacement {
            x: 12,
            y: 6,
            kind: WorldObjectKind::Log,
        },
    ];

    let forage_points = vec![(4, 5), (8, 4), (13, 5), (6, 7)];

    MapDef {
        id: MapId::Beach,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Forest map: 22x18  (Harvest Moon scale — ~1.8 screens)
// Layout: winding path, river, clearings, dense trees
// ---------------------------------------------------------------------------
fn generate_forest() -> MapDef {
    let w = 22usize;
    let h = 18usize;
    let mut tiles = vec![TileKind::Grass; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Main path from west (farm) winding east
    fill_rect(&mut tiles, 0, 7, 10, 2, TileKind::Path);
    fill_rect(&mut tiles, 10, 6, 4, 2, TileKind::Path); // Curve up
    fill_rect(&mut tiles, 14, 5, 8, 2, TileKind::Path);

    // South branch path
    fill_rect(&mut tiles, 8, 9, 2, 6, TileKind::Path);

    // River (east side, north-south)
    for y in 0..h {
        let rx = (17.0 + 1.0 * (y as f32 * 0.3).sin()) as usize;
        for dx in 0..2 {
            if rx + dx < w {
                tiles[y * w + rx + dx] = TileKind::Water;
            }
        }
    }

    // Bridge over river
    fill_rect(&mut tiles, 16, 5, 4, 2, TileKind::Bridge);

    // North clearing
    fill_rect(&mut tiles, 3, 2, 6, 4, TileKind::Grass);
    // Mushroom grove (south-west)
    fill_rect(&mut tiles, 2, 13, 5, 4, TileKind::Dirt);
    // South clearing
    fill_rect(&mut tiles, 11, 12, 5, 4, TileKind::Grass);

    let transitions = vec![
        // West exit -> Farm
        MapTransition {
            from_map: MapId::Forest,
            from_rect: (0, 6, 1, 4),
            to_map: MapId::Farm,
            to_pos: (30, 9),
        },
    ];

    let mut objects = Vec::new();

    // Dense trees (top area)
    for x in (0..22).step_by(2) {
        for y in (0..2).step_by(2) {
            objects.push(ObjectPlacement {
                x,
                y,
                kind: WorldObjectKind::Tree,
            });
        }
    }
    // Scattered forest trees
    let tree_positions = [
        (1, 4),
        (1, 10),
        (3, 8),
        (5, 10),
        (7, 4),
        (12, 3),
        (14, 3),
        (11, 10),
        (13, 14),
        (15, 10),
        (19, 2),
        (20, 8),
        (20, 12),
        (20, 16),
    ];
    for (tx, ty) in &tree_positions {
        objects.push(ObjectPlacement {
            x: *tx,
            y: *ty,
            kind: WorldObjectKind::Tree,
        });
    }
    // Pine trees (evergreen accents in the forest)
    let pine_positions = [(3, 5), (9, 3), (17, 8), (18, 14), (6, 14)];
    for (px, py) in &pine_positions {
        objects.push(ObjectPlacement {
            x: *px,
            y: *py,
            kind: WorldObjectKind::Pine,
        });
    }

    // Rocks near river
    objects.push(ObjectPlacement {
        x: 16,
        y: 3,
        kind: WorldObjectKind::Rock,
    });
    objects.push(ObjectPlacement {
        x: 16,
        y: 10,
        kind: WorldObjectKind::Rock,
    });
    // Stump
    objects.push(ObjectPlacement {
        x: 5,
        y: 3,
        kind: WorldObjectKind::Stump,
    });
    objects.push(ObjectPlacement {
        x: 13,
        y: 14,
        kind: WorldObjectKind::Stump,
    });
    // Bushes
    objects.push(ObjectPlacement {
        x: 1,
        y: 12,
        kind: WorldObjectKind::Bush,
    });
    objects.push(ObjectPlacement {
        x: 9,
        y: 12,
        kind: WorldObjectKind::Bush,
    });

    let forage_points = vec![
        (4, 3),
        (6, 4),
        (3, 14),
        (4, 16),
        (5, 14), // Mushroom grove
        (12, 13),
        (14, 15),
        (8, 11),
        (10, 14),
    ];

    MapDef {
        id: MapId::Forest,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Deep Forest: 30x28  (large, dense)
// ---------------------------------------------------------------------------
fn generate_deep_forest() -> MapDef {
    let w = 30usize;
    let h = 28usize;
    let mut tiles = vec![TileKind::Grass; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // ── Winding path from west entrance ──
    fill_rect(&mut tiles, 0, 14, 8, 2, TileKind::Path); // straight entry
    fill_rect(&mut tiles, 7, 13, 2, 3, TileKind::Path); // jog north
    fill_rect(&mut tiles, 8, 12, 6, 2, TileKind::Path); // east through center
    fill_rect(&mut tiles, 13, 12, 2, 4, TileKind::Path); // south bend
    fill_rect(&mut tiles, 14, 15, 6, 2, TileKind::Path); // east again
    fill_rect(&mut tiles, 19, 13, 2, 3, TileKind::Path); // jog north
    fill_rect(&mut tiles, 20, 12, 4, 2, TileKind::Path); // final stretch east

    // ── Central pond ──
    fill_rect(&mut tiles, 12, 8, 6, 4, TileKind::Water);

    // ── Rocky outcropping (southeast) ──
    fill_rect(&mut tiles, 22, 21, 6, 5, TileKind::Dirt);

    // ── Mushroom clearing (northeast) ──
    fill_rect(&mut tiles, 20, 2, 7, 5, TileKind::Grass); // Already grass, just noting the clearing

    // ── Flower meadow (southwest) ──
    fill_rect(&mut tiles, 2, 20, 8, 5, TileKind::Grass); // Open area

    let transitions = vec![MapTransition {
        from_map: MapId::DeepForest,
        from_rect: (0, 13, 1, 4),
        to_map: MapId::Forest,
        to_pos: (20, 8),
    }];

    let mut objects = Vec::new();

    // ── Dense tree border (north edge) ──
    for x in (0..30).step_by(2) {
        objects.push(ObjectPlacement {
            x,
            y: 0,
            kind: WorldObjectKind::Tree,
        });
    }
    for x in (1..28).step_by(3) {
        objects.push(ObjectPlacement {
            x,
            y: 1,
            kind: WorldObjectKind::Pine,
        });
    }

    // ── Dense tree border (east edge) ──
    for y in (0..28).step_by(3) {
        objects.push(ObjectPlacement {
            x: 28,
            y,
            kind: WorldObjectKind::Pine,
        });
        objects.push(ObjectPlacement {
            x: 29,
            y: y.saturating_add(1).min(27),
            kind: WorldObjectKind::Tree,
        });
    }

    // ── Dense tree border (south edge) ──
    for x in (0..20).step_by(2) {
        objects.push(ObjectPlacement {
            x,
            y: 27,
            kind: WorldObjectKind::Tree,
        });
    }

    // ── Interior tree clusters ──
    let tree_positions: &[(i32, i32)] = &[
        // Northwest grove
        (2, 3),
        (4, 2),
        (6, 4),
        (3, 6),
        (5, 7),
        (1, 8),
        // West of pond
        (8, 6),
        (9, 9),
        (7, 10),
        (10, 5),
        // East of pond
        (19, 7),
        (20, 9),
        (18, 10),
        // South scattered
        (6, 17),
        (8, 19),
        (11, 18),
        (16, 20),
        (18, 22),
        // Central area
        (10, 16),
        (15, 8),
        (17, 6),
    ];
    for &(tx, ty) in tree_positions {
        objects.push(ObjectPlacement {
            x: tx,
            y: ty,
            kind: WorldObjectKind::Tree,
        });
    }

    // ── Pine trees (evergreen accents) ──
    let pine_positions: &[(i32, i32)] = &[
        (1, 5),
        (4, 9),
        (7, 3),
        (11, 4),
        (16, 4),
        (3, 12),
        (6, 15),
        (9, 22),
        (14, 24),
        (22, 16),
        (25, 10),
        (24, 4),
        (12, 20),
        (17, 18),
        (26, 20),
    ];
    for &(px, py) in pine_positions {
        objects.push(ObjectPlacement {
            x: px,
            y: py,
            kind: WorldObjectKind::Pine,
        });
    }

    // ── Bushes (scattered undergrowth) ──
    let bush_positions: &[(i32, i32)] = &[
        (3, 21),
        (5, 22),
        (4, 24),
        (7, 23), // Flower meadow area
        (2, 10),
        (8, 14),
        (15, 10),
        (21, 6),
    ];
    for &(bx, by) in bush_positions {
        objects.push(ObjectPlacement {
            x: bx,
            y: by,
            kind: WorldObjectKind::Bush,
        });
    }

    // ── Stumps and logs ──
    objects.push(ObjectPlacement {
        x: 6,
        y: 11,
        kind: WorldObjectKind::Stump,
    });
    objects.push(ObjectPlacement {
        x: 16,
        y: 17,
        kind: WorldObjectKind::Stump,
    });
    objects.push(ObjectPlacement {
        x: 11,
        y: 23,
        kind: WorldObjectKind::Log,
    });
    objects.push(ObjectPlacement {
        x: 19,
        y: 20,
        kind: WorldObjectKind::Log,
    });

    // ── Rocky outcropping (southeast) ──
    objects.push(ObjectPlacement {
        x: 23,
        y: 22,
        kind: WorldObjectKind::Rock,
    });
    objects.push(ObjectPlacement {
        x: 25,
        y: 23,
        kind: WorldObjectKind::Rock,
    });
    objects.push(ObjectPlacement {
        x: 24,
        y: 24,
        kind: WorldObjectKind::LargeRock,
    });
    objects.push(ObjectPlacement {
        x: 26,
        y: 22,
        kind: WorldObjectKind::LargeRock,
    });
    objects.push(ObjectPlacement {
        x: 22,
        y: 24,
        kind: WorldObjectKind::Rock,
    });
    objects.push(ObjectPlacement {
        x: 27,
        y: 24,
        kind: WorldObjectKind::Rock,
    });

    // ── Forage points ──
    let forage_points = vec![
        // Mushroom clearing (northeast)
        (21, 3),
        (23, 4),
        (25, 3),
        (22, 5),
        // Flower meadow (southwest)
        (3, 22),
        (5, 23),
        (6, 21),
        // Scattered forest floor
        (8, 7),
        (18, 9),
        (10, 18),
        // Near pond
        (11, 7),
        (18, 8),
    ];

    MapDef {
        id: MapId::DeepForest,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Mine Entrance: 14x12  (compact)
// ---------------------------------------------------------------------------
fn generate_mine_entrance() -> MapDef {
    let w = 14usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::Stone; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Open ground
    fill_rect(&mut tiles, 2, 4, 10, 7, TileKind::Dirt);
    fill_rect(&mut tiles, 3, 8, 8, 3, TileKind::Grass);

    // Cave entrance (top center)
    fill_rect(&mut tiles, 5, 1, 4, 2, TileKind::Void);
    fill_rect(&mut tiles, 4, 0, 6, 1, TileKind::Stone);
    fill_rect(&mut tiles, 4, 3, 6, 1, TileKind::Stone);

    // Path from cave down
    fill_rect(&mut tiles, 6, 3, 2, 3, TileKind::Path);
    // Path east to farm
    fill_rect(&mut tiles, 12, 5, 2, 2, TileKind::Path);

    // Mountain spring
    fill_rect(&mut tiles, 2, 5, 1, 2, TileKind::Water);

    let transitions = vec![
        // East exit -> Farm
        MapTransition {
            from_map: MapId::MineEntrance,
            from_rect: (13, 5, 1, 2),
            to_map: MapId::Farm,
            to_pos: (1, 9),
        },
        // Cave entrance -> Mine floor 1
        MapTransition {
            from_map: MapId::MineEntrance,
            from_rect: (6, 3, 2, 1),
            to_map: MapId::Mine,
            to_pos: (8, 14),
        },
    ];

    let mut objects = Vec::new();
    let rock_positions = [(1, 2), (11, 2), (3, 6), (10, 6), (5, 10), (9, 10)];
    for (rx, ry) in &rock_positions {
        objects.push(ObjectPlacement {
            x: *rx,
            y: *ry,
            kind: WorldObjectKind::LargeRock,
        });
    }
    // Mountain pines
    objects.push(ObjectPlacement {
        x: 1,
        y: 8,
        kind: WorldObjectKind::Pine,
    });
    objects.push(ObjectPlacement {
        x: 12,
        y: 8,
        kind: WorldObjectKind::Pine,
    });

    let forage_points = vec![(4, 9), (9, 9)];

    MapDef {
        id: MapId::MineEntrance,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Mine floor (generic): 24x24 — used as base layout, mining domain will
// generate specific floors with rocks and enemies
// ---------------------------------------------------------------------------
fn generate_mine_floor() -> MapDef {
    let w = 24usize;
    let h = 24usize;
    let mut tiles = vec![TileKind::Stone; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Void border (cave walls)
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Open floor area
    fill_rect(&mut tiles, 2, 2, 20, 20, TileKind::Stone);

    // Ladder entry point area
    fill_rect(&mut tiles, 6, 12, 4, 4, TileKind::Dirt);

    // Elevator area
    fill_rect(&mut tiles, 14, 12, 4, 4, TileKind::WoodFloor);

    let transitions = vec![
        // Ladder up -> Mine entrance
        MapTransition {
            from_map: MapId::Mine,
            from_rect: (7, 12, 2, 2),
            to_map: MapId::MineEntrance,
            to_pos: (11, 4),
        },
    ];

    MapDef {
        id: MapId::Mine,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Player House: 16x16 interior — cozy Harvest Moon starter home
// y=0 is back wall, y=15 is front wall with door
// ---------------------------------------------------------------------------
fn generate_player_house() -> MapDef {
    let w = 16usize;
    let h = 16usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    let fill =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // ── Void perimeter (walls) ──
    for x in 0..w {
        tiles[x] = TileKind::Void; // y=0 back wall
        tiles[(h - 1) * w + x] = TileKind::Void; // y=15 front wall
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void; // x=0 left wall
        tiles[y * w + (w - 1)] = TileKind::Void; // x=15 right wall
    }

    // ── Door opening at front wall (y=15) ──
    tiles[15 * w + 7] = TileKind::WoodFloor;
    tiles[15 * w + 8] = TileKind::WoodFloor;

    // ── Stone fireplace on back wall (y=1-2, centered) ──
    fill(&mut tiles, 6, 1, 4, 2, TileKind::Stone);

    // ── Kitchen area upper-left: stone counter along back wall ──
    fill(&mut tiles, 1, 1, 4, 2, TileKind::Stone);

    // ── Living room rug (Path tiles) ──
    fill(&mut tiles, 5, 5, 6, 4, TileKind::Path);

    // ── Bedroom area upper-right: subtle rug ──
    fill(&mut tiles, 11, 2, 3, 2, TileKind::Path);

    // ── Entrance mat ──
    tiles[14 * w + 7] = TileKind::Path;
    tiles[14 * w + 8] = TileKind::Path;

    let transitions = vec![MapTransition {
        from_map: MapId::PlayerHouse,
        from_rect: (7, 15, 2, 1),
        to_map: MapId::Farm,
        to_pos: (16, 1), // just south of house door at (15,0)
    }];

    MapDef {
        id: MapId::PlayerHouse,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Town House West: 12x12 — modest scholar/doctor home, scenic only
// ---------------------------------------------------------------------------
fn generate_town_house_west() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    let fill =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;
    tiles[10 * w + 5] = TileKind::Path;
    tiles[10 * w + 6] = TileKind::Path;

    fill(&mut tiles, 1, 1, 3, 2, TileKind::Stone);
    fill(&mut tiles, 7, 1, 3, 2, TileKind::Path);
    fill(&mut tiles, 3, 5, 5, 2, TileKind::Path);

    let transitions = vec![MapTransition {
        from_map: MapId::TownHouseWest,
        from_rect: (5, 11, 2, 1),
        to_map: MapId::TownWest,
        to_pos: (3, 14),
    }];

    MapDef {
        id: MapId::TownHouseWest,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Town House East: 12x12 — cozy fisher/kid home, scenic only
// ---------------------------------------------------------------------------
fn generate_town_house_east() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    let fill =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;
    tiles[10 * w + 5] = TileKind::Path;
    tiles[10 * w + 6] = TileKind::Path;

    fill(&mut tiles, 1, 1, 4, 2, TileKind::Path);
    fill(&mut tiles, 7, 1, 3, 3, TileKind::Stone);
    fill(&mut tiles, 2, 6, 6, 2, TileKind::Path);

    let transitions = vec![MapTransition {
        from_map: MapId::TownHouseEast,
        from_rect: (5, 11, 2, 1),
        to_map: MapId::TownWest,
        to_pos: (9, 14),
    }];

    MapDef {
        id: MapId::TownHouseEast,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// General Store: 12x12 — organized shop with counter, shelves, displays
// ---------------------------------------------------------------------------
fn generate_general_store() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    let fill =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Void perimeter
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Door at y=11 (front wall)
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    // Sales counter — stone strip at y=4
    for x in 3..9 {
        tiles[4 * w + x] = TileKind::Stone;
    }

    // Behind-counter shelving area — stone along back wall
    fill(&mut tiles, 1, 1, 10, 1, TileKind::Stone);

    // Welcome mat at door
    tiles[10 * w + 5] = TileKind::Path;
    tiles[10 * w + 6] = TileKind::Path;

    // Display shelves along side walls (stone)
    for y in [3, 5, 7] {
        tiles[y * w + 1] = TileKind::Stone;
        tiles[y * w + 10] = TileKind::Stone;
    }

    let transitions = vec![MapTransition {
        from_map: MapId::GeneralStore,
        from_rect: (5, 11, 2, 1),
        to_map: MapId::Town,
        to_pos: (6, 3), // just south of store entrance at (5,2)
    }];

    MapDef {
        id: MapId::GeneralStore,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Animal Shop: 12x12 — rustic with hay storage, feed area
// ---------------------------------------------------------------------------
fn generate_animal_shop() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    let fill =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Void perimeter
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Door
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    // Hay/feed storage — dirt floor in back-left corner (y=1-3, x=1-4)
    fill(&mut tiles, 1, 1, 4, 3, TileKind::Dirt);

    // Sales counter — stone at y=4
    for x in 4..9 {
        tiles[4 * w + x] = TileKind::Stone;
    }

    // Back shelves — stone along back wall right side
    fill(&mut tiles, 5, 1, 5, 1, TileKind::Stone);

    // Entrance mat
    tiles[10 * w + 5] = TileKind::Path;
    tiles[10 * w + 6] = TileKind::Path;

    let transitions = vec![MapTransition {
        from_map: MapId::AnimalShop,
        from_rect: (5, 11, 2, 1),
        to_map: MapId::Town,
        to_pos: (22, 3),
    }];

    MapDef {
        id: MapId::AnimalShop,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Blacksmith: 12x12 — stone floors, forge area, anvil workspace
// ---------------------------------------------------------------------------
fn generate_blacksmith() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::Stone; w * h];

    let fill =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Void perimeter
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Interior stone (already base)
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            tiles[y * w + x] = TileKind::Stone;
        }
    }

    // Door
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    // Forge area — hot dirt in back-right (y=1-3, x=7-10)
    fill(&mut tiles, 7, 1, 3, 3, TileKind::Dirt);

    // Anvil workspace — wood floor island (y=5-7, x=4-6)
    fill(&mut tiles, 4, 5, 3, 3, TileKind::WoodFloor);

    // Counter / reception — wood floor at y=4, x=2-6
    for x in 2..7 {
        tiles[4 * w + x] = TileKind::WoodFloor;
    }

    // Entrance area — wood floor near door
    fill(&mut tiles, 4, 9, 4, 2, TileKind::WoodFloor);

    // Storage corner back-left (dirt)
    fill(&mut tiles, 1, 1, 3, 2, TileKind::Dirt);

    let transitions = vec![MapTransition {
        from_map: MapId::Blacksmith,
        from_rect: (5, 11, 2, 1),
        to_map: MapId::Town,
        to_pos: (22, 14),
    }];

    MapDef {
        id: MapId::Blacksmith,
        width: w,
        height: h,
        tiles,
        transitions,
        objects: Vec::new(),
        forage_points: Vec::new(),
    }
}

// ---------------------------------------------------------------------------
// Coral Island map: 30x22  (Tropical island — accessible by boat)
// Layout: water border, north dock, palm grove (NE), freshwater pond (CW),
//         rocky tide pools (SE), west beach, central sand/grass mix
// ---------------------------------------------------------------------------
fn generate_coral_island() -> MapDef {
    let w = 30usize;
    let h = 22usize;
    // Start with all water
    let mut tiles = vec![TileKind::Water; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let xx = x0 + dx;
                    let yy = y0 + dy;
                    if xx < w && yy < h {
                        tiles[yy * w + xx] = kind;
                    }
                }
            }
        };

    // Island interior — fill from (3, 2) to (26, 19) with Sand base
    fill_rect(&mut tiles, 3, 2, 24, 18, TileKind::Sand);

    // Central grass area
    fill_rect(&mut tiles, 7, 5, 16, 12, TileKind::Grass);

    // North dock approach: sand strip at y=1-2, x=13-17
    fill_rect(&mut tiles, 13, 1, 5, 2, TileKind::Sand);
    // Path leading south from dock
    fill_rect(&mut tiles, 14, 3, 2, 4, TileKind::Path);

    // West beach strip (x=3-6, full height island)
    fill_rect(&mut tiles, 3, 2, 4, 18, TileKind::Sand);

    // South beach strip
    fill_rect(&mut tiles, 3, 18, 24, 2, TileKind::Sand);

    // Northeast palm grove area — keep as sand with grass patches
    fill_rect(&mut tiles, 19, 3, 8, 8, TileKind::Sand);
    fill_rect(&mut tiles, 20, 4, 6, 6, TileKind::Grass);

    // Southeast tide pool area — dirt tiles
    fill_rect(&mut tiles, 20, 13, 7, 6, TileKind::Dirt);

    // Freshwater pond (center-west): 4x3 water tiles around (8-11, 10-12)
    fill_rect(&mut tiles, 8, 10, 4, 3, TileKind::Water);
    // Pond shoreline
    fill_rect(&mut tiles, 7, 9, 6, 5, TileKind::Sand);
    fill_rect(&mut tiles, 8, 10, 4, 3, TileKind::Water); // restore water

    let transitions = vec![
        // North edge → Beach map (sail back)
        MapTransition {
            from_map: MapId::CoralIsland,
            from_rect: (13, 0, 5, 1),
            to_map: MapId::Beach,
            to_pos: (10, 12),
        },
    ];

    let mut objects = Vec::new();

    // Dock pier objects at north arrival point
    objects.push(ObjectPlacement {
        x: 14,
        y: 1,
        kind: WorldObjectKind::Dock,
    });
    objects.push(ObjectPlacement {
        x: 16,
        y: 1,
        kind: WorldObjectKind::Dock,
    });

    // Northeast PalmTree grove (8+ trees)
    let palm_grove = [
        (20, 3),
        (22, 3),
        (24, 3),
        (26, 3),
        (21, 5),
        (23, 5),
        (25, 5),
        (20, 7),
        (22, 7),
        (24, 7),
        (26, 6),
        (27, 4),
    ];
    for (px, py) in &palm_grove {
        objects.push(ObjectPlacement {
            x: *px,
            y: *py,
            kind: WorldObjectKind::PalmTree,
        });
    }

    // Additional scattered PalmTrees (meet 15+ total)
    let more_palms = [
        (5, 3),
        (5, 7),
        (5, 12),
        (5, 17),
        (27, 12),
        (27, 16),
        (15, 18),
        (19, 18),
    ];
    for (px, py) in &more_palms {
        objects.push(ObjectPlacement {
            x: *px,
            y: *py,
            kind: WorldObjectKind::PalmTree,
        });
    }

    // Southeast tide pools: Coral objects (8+)
    let coral_spots = [
        (21, 14),
        (23, 14),
        (25, 14),
        (27, 14),
        (20, 16),
        (22, 16),
        (24, 16),
        (26, 16),
        (21, 18),
        (24, 18),
    ];
    for (cx, cy) in &coral_spots {
        objects.push(ObjectPlacement {
            x: *cx,
            y: *cy,
            kind: WorldObjectKind::Coral,
        });
    }

    // Southeast rocks (4+)
    let rock_spots = [(22, 13), (25, 13), (20, 15), (26, 15)];
    for (rx, ry) in &rock_spots {
        objects.push(ObjectPlacement {
            x: *rx,
            y: *ry,
            kind: WorldObjectKind::Rock,
        });
    }

    // West beach driftwood (5+)
    let driftwood_spots = [(4, 5), (4, 9), (4, 13), (4, 17), (6, 19)];
    for (dx, dy) in &driftwood_spots {
        objects.push(ObjectPlacement {
            x: *dx,
            y: *dy,
            kind: WorldObjectKind::Driftwood,
        });
    }

    // Forage points spread across the island (8+): shells, sea glass, tropical herbs
    let forage_points = vec![
        (6, 4),
        (6, 8),
        (6, 14),
        (6, 18),
        (13, 7),
        (17, 7),
        (10, 15),
        (15, 14),
        (23, 10),
        (12, 19),
    ];

    MapDef {
        id: MapId::CoralIsland,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}

// ---------------------------------------------------------------------------
// Snow Mountain map: 32x24 — snowy alpine area north of Farm
// Layout: stone/grass terrain with pine trees, rocks, and mountain paths
// ---------------------------------------------------------------------------
fn generate_snow_mountain() -> MapDef {
    let w = 32usize;
    let h = 24usize;
    let mut tiles = vec![TileKind::Stone; w * h];

    let fill_rect =
        |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
            for dy in 0..rh {
                for dx in 0..rw {
                    let x = x0 + dx;
                    let y = y0 + dy;
                    if x < w && y < h {
                        tiles[y * w + x] = kind;
                    }
                }
            }
        };

    let set = |tiles: &mut Vec<TileKind>, x: usize, y: usize, kind: TileKind| {
        if x < w && y < h {
            tiles[y * w + x] = kind;
        }
    };

    // Grassy meadow at southern base (rows 19-23)
    fill_rect(&mut tiles, 0, 19, 32, 5, TileKind::Grass);

    // West clearing (rows 14-18, cols 2-7)
    fill_rect(&mut tiles, 2, 14, 6, 5, TileKind::Grass);

    // East clearing (rows 15-18, cols 24-29)
    fill_rect(&mut tiles, 24, 15, 6, 4, TileKind::Grass);

    // Small alpine meadow (rows 6-8, cols 3-7)
    fill_rect(&mut tiles, 3, 6, 5, 3, TileKind::Grass);

    // Grass shore around frozen lake (fishing access)
    for y in 3..10usize {
        for x in 22..32.min(w) {
            let dist = (x as i32 - 27) * (x as i32 - 27) + (y as i32 - 6) * (y as i32 - 6);
            if dist <= 16 && dist > 10 {
                set(&mut tiles, x, y, TileKind::Grass);
            }
        }
    }

    // Switchback path — south entrance
    fill_rect(&mut tiles, 14, 19, 4, 5, TileKind::Path);

    // Path east (rows 18-19)
    fill_rect(&mut tiles, 14, 18, 10, 2, TileKind::Path);

    // Path north east side (rows 14-18)
    fill_rect(&mut tiles, 22, 14, 2, 5, TileKind::Path);

    // Path west (rows 13-14)
    fill_rect(&mut tiles, 8, 13, 16, 2, TileKind::Path);

    // Path north west side (rows 9-13)
    fill_rect(&mut tiles, 8, 9, 2, 5, TileKind::Path);

    // Path east to summit (rows 8-9)
    fill_rect(&mut tiles, 8, 8, 12, 2, TileKind::Path);

    // Path north to summit (rows 3-8)
    fill_rect(&mut tiles, 14, 3, 2, 6, TileKind::Path);

    // Summit clearing (rows 1-3, cols 12-18)
    fill_rect(&mut tiles, 12, 1, 7, 3, TileKind::Path);

    // Frozen mountain lake (northeast, elliptical)
    for y in 4..9usize {
        for x in 24..31usize {
            if ((x as i32 - 27) * (x as i32 - 27) + (y as i32 - 6) * (y as i32 - 6)) <= 10 {
                set(&mut tiles, x, y, TileKind::Water);
            }
        }
    }

    // Stream from lake
    for y in 9..12usize {
        set(&mut tiles, 26, y, TileKind::Water);
    }

    // Dirt patches
    for &(x, y) in &[(5usize, 15usize), (6, 16), (4, 16), (26, 16), (27, 17)] {
        set(&mut tiles, x, y, TileKind::Dirt);
    }

    // Grass patches for variety
    for &(x, y) in &[(13usize, 10usize), (18, 12), (11, 17), (10, 6), (20, 10)] {
        set(&mut tiles, x, y, TileKind::Grass);
    }

    let objects = vec![
        // Dense pines on western slope
        ObjectPlacement {
            x: 0,
            y: 2,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 2,
            y: 3,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 1,
            y: 5,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 3,
            y: 6,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 0,
            y: 8,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 2,
            y: 10,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 1,
            y: 12,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 3,
            y: 15,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 0,
            y: 16,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 2,
            y: 18,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 4,
            y: 4,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 5,
            y: 7,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 4,
            y: 10,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 5,
            y: 12,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 6,
            y: 14,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 4,
            y: 17,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 5,
            y: 20,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 3,
            y: 21,
            kind: WorldObjectKind::Pine,
        },
        // Dense pines on eastern slope
        ObjectPlacement {
            x: 28,
            y: 1,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 30,
            y: 3,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 31,
            y: 7,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 28,
            y: 10,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 30,
            y: 12,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 29,
            y: 14,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 31,
            y: 16,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 27,
            y: 3,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 27,
            y: 11,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 29,
            y: 16,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 30,
            y: 18,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 28,
            y: 20,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 31,
            y: 21,
            kind: WorldObjectKind::Pine,
        },
        // Scattered pines mid-map
        ObjectPlacement {
            x: 11,
            y: 6,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 19,
            y: 5,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 13,
            y: 11,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 20,
            y: 11,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 11,
            y: 16,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 19,
            y: 16,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 10,
            y: 20,
            kind: WorldObjectKind::Pine,
        },
        ObjectPlacement {
            x: 20,
            y: 21,
            kind: WorldObjectKind::Pine,
        },
        // Large rocks (mineable ore deposits)
        ObjectPlacement {
            x: 7,
            y: 2,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 10,
            y: 1,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 21,
            y: 1,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 11,
            y: 2,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 6,
            y: 10,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 12,
            y: 5,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 20,
            y: 7,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 25,
            y: 13,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 7,
            y: 17,
            kind: WorldObjectKind::LargeRock,
        },
        ObjectPlacement {
            x: 21,
            y: 17,
            kind: WorldObjectKind::LargeRock,
        },
        // Regular rocks
        ObjectPlacement {
            x: 9,
            y: 4,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 11,
            y: 3,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 17,
            y: 4,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 22,
            y: 6,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 5,
            y: 9,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 13,
            y: 7,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 19,
            y: 10,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 24,
            y: 11,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 20,
            y: 15,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 6,
            y: 19,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 25,
            y: 19,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 13,
            y: 17,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 18,
            y: 15,
            kind: WorldObjectKind::Rock,
        },
        ObjectPlacement {
            x: 9,
            y: 6,
            kind: WorldObjectKind::Rock,
        },
        // Stumps
        ObjectPlacement {
            x: 3,
            y: 8,
            kind: WorldObjectKind::Stump,
        },
        ObjectPlacement {
            x: 10,
            y: 12,
            kind: WorldObjectKind::Stump,
        },
        ObjectPlacement {
            x: 21,
            y: 10,
            kind: WorldObjectKind::Stump,
        },
        ObjectPlacement {
            x: 16,
            y: 16,
            kind: WorldObjectKind::Stump,
        },
        ObjectPlacement {
            x: 8,
            y: 21,
            kind: WorldObjectKind::Stump,
        },
        ObjectPlacement {
            x: 24,
            y: 20,
            kind: WorldObjectKind::Stump,
        },
        // Bushes (alpine shrubs)
        ObjectPlacement {
            x: 5,
            y: 14,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 7,
            y: 6,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 19,
            y: 12,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 25,
            y: 17,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 3,
            y: 20,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 15,
            y: 20,
            kind: WorldObjectKind::Bush,
        },
        ObjectPlacement {
            x: 27,
            y: 20,
            kind: WorldObjectKind::Bush,
        },
    ];

    let forage_points = vec![
        (4, 7),
        (6, 8),
        (3, 14),
        (5, 16),
        (7, 20),
        (12, 21),
        (16, 22),
        (20, 20),
        (26, 16),
        (27, 18),
        (13, 10),
        (18, 12),
        (11, 17),
        (22, 15),
    ];

    let transitions = vec![];

    MapDef {
        id: MapId::SnowMountain,
        width: w,
        height: h,
        tiles,
        transitions,
        objects,
        forage_points,
    }
}
