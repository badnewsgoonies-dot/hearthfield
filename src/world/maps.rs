//! Map data definitions for all game areas.
//!
//! Each map is defined as a 2D grid of TileKind values.
//! Maps also include transition zones and object spawn points.

use crate::shared::*;

/// Complete definition of a game map.
#[derive(Debug, Clone)]
pub struct MapDef {
    pub id: MapId,
    pub width: usize,
    pub height: usize,
    /// Row-major tile data: tiles[y * width + x]
    pub tiles: Vec<TileKind>,
    /// Transition zones linking to other maps.
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

    #[allow(dead_code)]
    pub fn set_tile(&mut self, x: i32, y: i32, kind: TileKind) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.tiles[y as usize * self.width + x as usize] = kind;
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldObjectKind {
    Tree,
    Rock,
    Stump,
    Bush,
    LargeRock,
    Log,
}

// ═══════════════════════════════════════════════════════════════════════
// MAP GENERATORS
// ═══════════════════════════════════════════════════════════════════════

pub fn generate_map(map_id: MapId) -> MapDef {
    match map_id {
        MapId::Farm => generate_farm(),
        MapId::Town => generate_town(),
        MapId::Beach => generate_beach(),
        MapId::Forest => generate_forest(),
        MapId::MineEntrance => generate_mine_entrance(),
        MapId::Mine => generate_mine_floor(),
        MapId::PlayerHouse => generate_player_house(),
        MapId::GeneralStore => generate_general_store(),
        MapId::AnimalShop => generate_animal_shop(),
        MapId::Blacksmith => generate_blacksmith(),
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

    let fill_rect = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
    fill_rect(&mut tiles, 3, 16, 5, 3, TileKind::Stone);   // Barn
    fill_rect(&mut tiles, 9, 17, 3, 2, TileKind::Stone);    // Chicken coop
    fill_rect(&mut tiles, 8, 19, 4, 1, TileKind::Path);     // Path connecting

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
        objects.push(ObjectPlacement { x: x as i32, y: 1, kind: WorldObjectKind::Tree });
    }
    for x in (22..30).step_by(3) {
        objects.push(ObjectPlacement { x: x as i32, y: 1, kind: WorldObjectKind::Tree });
    }

    // Rocks in the dirt (player clears to farm)
    let rock_positions = [
        (8, 7), (12, 8), (18, 7), (22, 9), (10, 12),
        (16, 11), (20, 13), (14, 14), (24, 8),
    ];
    for (rx, ry) in &rock_positions {
        objects.push(ObjectPlacement { x: *rx, y: *ry, kind: WorldObjectKind::Rock });
    }

    // Stumps
    objects.push(ObjectPlacement { x: 7, y: 14, kind: WorldObjectKind::Stump });
    objects.push(ObjectPlacement { x: 25, y: 14, kind: WorldObjectKind::Stump });

    // Bushes near pond
    objects.push(ObjectPlacement { x: 23, y: 17, kind: WorldObjectKind::Bush });

    let forage_points = vec![
        (2, 4), (29, 3), (2, 20), (29, 20), (5, 22),
    ];

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

    let fill_rect = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
    fill_rect(&mut tiles, 13, 9, 2, 2, TileKind::Water);  // Fountain

    // General Store (north-west)
    fill_rect(&mut tiles, 2, 2, 8, 5, TileKind::Stone);
    fill_rect(&mut tiles, 5, 7, 2, 2, TileKind::Path);   // Path to plaza

    // Animal Shop (north-east)
    fill_rect(&mut tiles, 18, 2, 8, 5, TileKind::Stone);
    fill_rect(&mut tiles, 21, 7, 2, 2, TileKind::Path);   // Path to plaza

    // Blacksmith (east, below plaza)
    fill_rect(&mut tiles, 20, 13, 6, 4, TileKind::Stone);
    fill_rect(&mut tiles, 17, 14, 3, 2, TileKind::Path);  // Path to main road

    // NPC houses (west, below plaza)
    fill_rect(&mut tiles, 2, 13, 5, 3, TileKind::Stone);  // House 1 (doc/librarian)
    fill_rect(&mut tiles, 8, 13, 5, 3, TileKind::Stone);  // House 2 (fisher/kid)

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
        objects.push(ObjectPlacement { x: x as i32, y: 0, kind: WorldObjectKind::Tree });
    }
    for x in (16..28).step_by(3) {
        objects.push(ObjectPlacement { x: x as i32, y: 0, kind: WorldObjectKind::Tree });
    }
    // Park trees
    objects.push(ObjectPlacement { x: 2, y: 17, kind: WorldObjectKind::Tree });
    objects.push(ObjectPlacement { x: 9, y: 17, kind: WorldObjectKind::Tree });
    // Bushes near plaza
    objects.push(ObjectPlacement { x: 10, y: 7, kind: WorldObjectKind::Bush });
    objects.push(ObjectPlacement { x: 17, y: 7, kind: WorldObjectKind::Bush });

    let forage_points = vec![
        (1, 19), (9, 20), (26, 19),
    ];

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
// Beach map: 20x14  (Harvest Moon scale — ~1.3 screens)
// Layout: grass top, sand middle, ocean bottom, dock on right
// ---------------------------------------------------------------------------
fn generate_beach() -> MapDef {
    let w = 20usize;
    let h = 14usize;
    let mut tiles = vec![TileKind::Sand; w * h];

    let fill_rect = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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

    let mut objects = Vec::new();
    objects.push(ObjectPlacement { x: 17, y: 1, kind: WorldObjectKind::Rock });
    objects.push(ObjectPlacement { x: 7, y: 7, kind: WorldObjectKind::Log });
    objects.push(ObjectPlacement { x: 12, y: 6, kind: WorldObjectKind::Log });

    let forage_points = vec![
        (4, 5), (8, 4), (13, 5), (6, 7),
    ];

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

    let fill_rect = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
    fill_rect(&mut tiles, 10, 6, 4, 2, TileKind::Path);  // Curve up
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
            objects.push(ObjectPlacement { x: x as i32, y: y as i32, kind: WorldObjectKind::Tree });
        }
    }
    // Scattered forest trees
    let tree_positions = [
        (1, 4), (1, 10), (3, 8), (5, 10), (7, 4),
        (12, 3), (14, 3), (11, 10), (13, 14), (15, 10),
        (19, 2), (20, 8), (20, 12), (20, 16),
    ];
    for (tx, ty) in &tree_positions {
        objects.push(ObjectPlacement { x: *tx, y: *ty, kind: WorldObjectKind::Tree });
    }

    // Rocks near river
    objects.push(ObjectPlacement { x: 16, y: 3, kind: WorldObjectKind::Rock });
    objects.push(ObjectPlacement { x: 16, y: 10, kind: WorldObjectKind::Rock });
    // Stump
    objects.push(ObjectPlacement { x: 5, y: 3, kind: WorldObjectKind::Stump });
    objects.push(ObjectPlacement { x: 13, y: 14, kind: WorldObjectKind::Stump });
    // Bushes
    objects.push(ObjectPlacement { x: 1, y: 12, kind: WorldObjectKind::Bush });
    objects.push(ObjectPlacement { x: 9, y: 12, kind: WorldObjectKind::Bush });

    let forage_points = vec![
        (4, 3), (6, 4), (3, 14), (4, 16), (5, 14),  // Mushroom grove
        (12, 13), (14, 15), (8, 11), (10, 14),
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
// Mine Entrance: 14x12  (compact)
// ---------------------------------------------------------------------------
fn generate_mine_entrance() -> MapDef {
    let w = 14usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::Stone; w * h];

    let fill_rect = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
            from_rect: (6, 1, 2, 2),
            to_map: MapId::Mine,
            to_pos: (8, 14),
        },
    ];

    let mut objects = Vec::new();
    let rock_positions = [(1, 2), (11, 2), (3, 6), (10, 6), (5, 10), (9, 10)];
    for (rx, ry) in &rock_positions {
        objects.push(ObjectPlacement { x: *rx, y: *ry, kind: WorldObjectKind::LargeRock });
    }

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

    let fill_rect = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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

    let fill = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
        tiles[x] = TileKind::Void;               // y=0 back wall
        tiles[(h - 1) * w + x] = TileKind::Void;  // y=15 front wall
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;            // x=0 left wall
        tiles[y * w + (w - 1)] = TileKind::Void;  // x=15 right wall
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

    let transitions = vec![
        MapTransition {
            from_map: MapId::PlayerHouse,
            from_rect: (7, 15, 2, 1),
            to_map: MapId::Farm,
            to_pos: (31, 2),
        },
    ];

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
// General Store: 12x12 — organized shop with counter, shelves, displays
// ---------------------------------------------------------------------------
fn generate_general_store() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    let fill = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
    for x in 0..w { tiles[x] = TileKind::Void; tiles[(h-1)*w+x] = TileKind::Void; }
    for y in 0..h { tiles[y*w] = TileKind::Void; tiles[y*w+(w-1)] = TileKind::Void; }

    // Door at y=11 (front wall)
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    // Sales counter — stone strip at y=4
    for x in 3..9 { tiles[4 * w + x] = TileKind::Stone; }

    // Behind-counter shelving area — stone along back wall
    fill(&mut tiles, 1, 1, 10, 1, TileKind::Stone);

    // Welcome mat at door
    tiles[10 * w + 5] = TileKind::Path;
    tiles[10 * w + 6] = TileKind::Path;

    // Display shelves along side walls (stone)
    for y in [3, 5, 7] { tiles[y * w + 1] = TileKind::Stone; tiles[y * w + 10] = TileKind::Stone; }

    let transitions = vec![
        MapTransition {
            from_map: MapId::GeneralStore,
            from_rect: (5, 11, 2, 1),
            to_map: MapId::Town,
            to_pos: (8, 5),
        },
    ];

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

    let fill = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
    for x in 0..w { tiles[x] = TileKind::Void; tiles[(h-1)*w+x] = TileKind::Void; }
    for y in 0..h { tiles[y*w] = TileKind::Void; tiles[y*w+(w-1)] = TileKind::Void; }

    // Door
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    // Hay/feed storage — dirt floor in back-left corner (y=1-3, x=1-4)
    fill(&mut tiles, 1, 1, 4, 3, TileKind::Dirt);

    // Sales counter — stone at y=4
    for x in 4..9 { tiles[4 * w + x] = TileKind::Stone; }

    // Back shelves — stone along back wall right side
    fill(&mut tiles, 5, 1, 5, 1, TileKind::Stone);

    // Entrance mat
    tiles[10 * w + 5] = TileKind::Path;
    tiles[10 * w + 6] = TileKind::Path;

    let transitions = vec![
        MapTransition {
            from_map: MapId::AnimalShop,
            from_rect: (5, 11, 2, 1),
            to_map: MapId::Town,
            to_pos: (22, 3),
        },
    ];

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

    let fill = |tiles: &mut Vec<TileKind>, x0: usize, y0: usize, rw: usize, rh: usize, kind: TileKind| {
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
    for x in 0..w { tiles[x] = TileKind::Void; tiles[(h-1)*w+x] = TileKind::Void; }
    for y in 0..h { tiles[y*w] = TileKind::Void; tiles[y*w+(w-1)] = TileKind::Void; }

    // Interior stone (already base)
    for y in 1..(h-1) { for x in 1..(w-1) { tiles[y*w+x] = TileKind::Stone; } }

    // Door
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    // Forge area — hot dirt in back-right (y=1-3, x=7-10)
    fill(&mut tiles, 7, 1, 3, 3, TileKind::Dirt);

    // Anvil workspace — wood floor island (y=5-7, x=4-6)
    fill(&mut tiles, 4, 5, 3, 3, TileKind::WoodFloor);

    // Counter / reception — wood floor at y=4, x=2-6
    for x in 2..7 { tiles[4 * w + x] = TileKind::WoodFloor; }

    // Entrance area — wood floor near door
    fill(&mut tiles, 4, 9, 4, 2, TileKind::WoodFloor);

    // Storage corner back-left (dirt)
    fill(&mut tiles, 1, 1, 3, 2, TileKind::Dirt);

    let transitions = vec![
        MapTransition {
            from_map: MapId::Blacksmith,
            from_rect: (5, 11, 2, 1),
            to_map: MapId::Town,
            to_pos: (22, 14),
        },
    ];

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
