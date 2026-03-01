//! Map data definitions for all game areas.
//!
//! Each map is defined as a 2D grid of TileKind values.
//! Maps also include transition zones and object spawn points.

use crate::shared::*;

/// Complete definition of a game map.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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
// Farm map: 64x64
// Layout: grass border, large central dirt/tillable area, pond in SE corner,
// paths leading to exits (south to town, east to forest, north-west to mine)
// ---------------------------------------------------------------------------
fn generate_farm() -> MapDef {
    let w = 64usize;
    let h = 64usize;
    let mut tiles = vec![TileKind::Grass; w * h];

    // Helper closure to set a rectangular region
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

    // Central farming area: large dirt region (columns 8..56, rows 8..48)
    fill_rect(&mut tiles, 8, 8, 48, 40, TileKind::Dirt);

    // Path from farmhouse (top-center) downward
    fill_rect(&mut tiles, 30, 2, 4, 8, TileKind::Path);
    // Player house entrance marker at top
    fill_rect(&mut tiles, 29, 0, 6, 3, TileKind::Stone);

    // Path south to town exit (bottom center)
    fill_rect(&mut tiles, 30, 48, 4, 16, TileKind::Path);

    // Path east to forest exit (right edge, middle)
    fill_rect(&mut tiles, 56, 28, 8, 4, TileKind::Path);

    // Path north-west to mine entrance
    fill_rect(&mut tiles, 0, 10, 10, 4, TileKind::Path);

    // Pond in the southeast corner
    fill_rect(&mut tiles, 50, 50, 10, 10, TileKind::Water);
    // Pond shoreline (sand around water)
    for dy in 0..12 {
        for dx in 0..12 {
            let xx = 49 + dx;
            let yy = 49 + dy;
            if xx < w && yy < h {
                let is_edge = dx == 0 || dy == 0 || dx == 11 || dy == 11;
                let is_water = tiles[yy * w + xx] == TileKind::Water;
                if is_edge && !is_water {
                    tiles[yy * w + xx] = TileKind::Sand;
                }
            }
        }
    }

    // Shipping bin area (near house)
    fill_rect(&mut tiles, 36, 3, 2, 2, TileKind::WoodFloor);

    // Animal buildings
    fill_rect(&mut tiles, 8, 38, 5, 4, TileKind::Stone);   // Barn
    fill_rect(&mut tiles, 20, 38, 3, 3, TileKind::Stone);   // Chicken coop
    fill_rect(&mut tiles, 13, 40, 7, 2, TileKind::Path);    // Path connecting barn to coop

    // Small bridge over a stream running east-west at row 52
    fill_rect(&mut tiles, 20, 52, 2, 1, TileKind::Bridge);

    // Grass borders are already the default

    let transitions = vec![
        // South exit -> Town
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (28, 63, 8, 1),
            to_map: MapId::Town,
            to_pos: (24, 0),
        },
        // East exit -> Forest
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (63, 26, 1, 8),
            to_map: MapId::Forest,
            to_pos: (0, 20),
        },
        // North-west exit -> Mine Entrance
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (0, 8, 1, 8),
            to_map: MapId::MineEntrance,
            to_pos: (22, 12),
        },
        // Top entrance -> Player House
        MapTransition {
            from_map: MapId::Farm,
            from_rect: (31, 0, 2, 1),
            to_map: MapId::PlayerHouse,
            to_pos: (8, 14),
        },
    ];

    // Objects: scatter trees around border, rocks in farming area
    let mut objects = Vec::new();

    // Trees along the top edge
    for x in (2..28).step_by(3) {
        objects.push(ObjectPlacement { x: x as i32, y: 2, kind: WorldObjectKind::Tree });
    }
    for x in (36..62).step_by(3) {
        objects.push(ObjectPlacement { x: x as i32, y: 2, kind: WorldObjectKind::Tree });
    }

    // Trees along left edge
    for y in (16..60).step_by(4) {
        objects.push(ObjectPlacement { x: 2, y: y as i32, kind: WorldObjectKind::Tree });
        objects.push(ObjectPlacement { x: 4, y: y as i32, kind: WorldObjectKind::Tree });
    }

    // Trees along right edge (above forest path)
    for y in (2..26).step_by(3) {
        objects.push(ObjectPlacement { x: 60, y: y as i32, kind: WorldObjectKind::Tree });
    }
    for y in (34..60).step_by(3) {
        objects.push(ObjectPlacement { x: 60, y: y as i32, kind: WorldObjectKind::Tree });
    }

    // Rocks scattered in the dirt area (player must clear to farm)
    let rock_positions = [
        (12, 12), (18, 10), (25, 15), (35, 11), (42, 14),
        (15, 22), (28, 25), (40, 20), (50, 18), (20, 35),
        (38, 32), (45, 38), (13, 40), (48, 42), (22, 44),
    ];
    for (rx, ry) in &rock_positions {
        objects.push(ObjectPlacement { x: *rx, y: *ry, kind: WorldObjectKind::Rock });
    }

    // Stumps
    let stump_positions = [(10, 30), (30, 40), (52, 10), (46, 46)];
    for (sx, sy) in &stump_positions {
        objects.push(ObjectPlacement { x: *sx, y: *sy, kind: WorldObjectKind::Stump });
    }

    // Bushes at the pond edge
    objects.push(ObjectPlacement { x: 48, y: 50, kind: WorldObjectKind::Bush });
    objects.push(ObjectPlacement { x: 48, y: 55, kind: WorldObjectKind::Bush });

    // Forageable points scattered around non-farming areas
    let forage_points = vec![
        (3, 5), (5, 3), (60, 5), (62, 8), (3, 55), (5, 60),
        (58, 55), (60, 50), (7, 50), (55, 5),
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
// Town map: 48x48
// Layout: central plaza with fountain, paths connecting buildings,
// shops along north side, houses scattered, park area
// ---------------------------------------------------------------------------
fn generate_town() -> MapDef {
    let w = 48usize;
    let h = 48usize;
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

    // Main road running north-south through center
    fill_rect(&mut tiles, 22, 0, 4, 48, TileKind::Path);

    // East-west road through middle
    fill_rect(&mut tiles, 0, 22, 48, 4, TileKind::Path);

    // Central plaza / fountain area (stone square)
    fill_rect(&mut tiles, 20, 20, 8, 8, TileKind::Stone);
    // Fountain water in the center of the plaza
    fill_rect(&mut tiles, 22, 22, 4, 4, TileKind::Water);

    // General Store building footprint (north-west)
    fill_rect(&mut tiles, 4, 4, 10, 8, TileKind::Stone);
    // Store entrance path
    fill_rect(&mut tiles, 8, 12, 2, 10, TileKind::Path);

    // Animal Shop building footprint (north-east)
    fill_rect(&mut tiles, 34, 4, 10, 8, TileKind::Stone);
    // Animal shop entrance path
    fill_rect(&mut tiles, 38, 12, 2, 10, TileKind::Path);

    // Blacksmith building (east side)
    fill_rect(&mut tiles, 38, 30, 8, 6, TileKind::Stone);
    // Blacksmith path to main road
    fill_rect(&mut tiles, 26, 32, 12, 2, TileKind::Path);

    // Park area with grass and a small pond (south-west)
    fill_rect(&mut tiles, 2, 34, 16, 12, TileKind::Grass);
    fill_rect(&mut tiles, 6, 38, 4, 4, TileKind::Water);
    // Sand around park pond
    fill_rect(&mut tiles, 5, 37, 6, 1, TileKind::Sand);
    fill_rect(&mut tiles, 5, 42, 6, 1, TileKind::Sand);
    fill_rect(&mut tiles, 5, 38, 1, 4, TileKind::Sand);
    fill_rect(&mut tiles, 10, 38, 1, 4, TileKind::Sand);

    // Residential path (south side)
    fill_rect(&mut tiles, 4, 28, 16, 2, TileKind::Path);

    // NPC houses (just stone footprints for now)
    fill_rect(&mut tiles, 2, 30, 6, 4, TileKind::Stone); // House 1
    fill_rect(&mut tiles, 10, 30, 6, 4, TileKind::Stone); // House 2

    // Beach exit path (south-east)
    fill_rect(&mut tiles, 40, 44, 8, 4, TileKind::Path);

    let transitions = vec![
        // North exit -> Farm
        MapTransition {
            from_map: MapId::Town,
            from_rect: (22, 0, 4, 1),
            to_map: MapId::Farm,
            to_pos: (30, 62),
        },
        // General Store entrance
        MapTransition {
            from_map: MapId::Town,
            from_rect: (8, 4, 2, 1),
            to_map: MapId::GeneralStore,
            to_pos: (6, 10),
        },
        // Animal Shop entrance
        MapTransition {
            from_map: MapId::Town,
            from_rect: (38, 4, 2, 1),
            to_map: MapId::AnimalShop,
            to_pos: (6, 10),
        },
        // Blacksmith entrance
        MapTransition {
            from_map: MapId::Town,
            from_rect: (41, 30, 2, 1),
            to_map: MapId::Blacksmith,
            to_pos: (6, 10),
        },
        // South-east exit -> Beach
        MapTransition {
            from_map: MapId::Town,
            from_rect: (47, 44, 1, 4),
            to_map: MapId::Beach,
            to_pos: (0, 16),
        },
    ];

    // Trees along edges and in park
    let mut objects = Vec::new();
    for x in (1..20).step_by(4) {
        objects.push(ObjectPlacement { x: x as i32, y: 1, kind: WorldObjectKind::Tree });
    }
    for x in (28..46).step_by(4) {
        objects.push(ObjectPlacement { x: x as i32, y: 1, kind: WorldObjectKind::Tree });
    }
    // Park trees
    objects.push(ObjectPlacement { x: 3, y: 36, kind: WorldObjectKind::Tree });
    objects.push(ObjectPlacement { x: 14, y: 36, kind: WorldObjectKind::Tree });
    objects.push(ObjectPlacement { x: 3, y: 44, kind: WorldObjectKind::Tree });
    objects.push(ObjectPlacement { x: 14, y: 44, kind: WorldObjectKind::Tree });
    // Bushes near plaza
    objects.push(ObjectPlacement { x: 19, y: 20, kind: WorldObjectKind::Bush });
    objects.push(ObjectPlacement { x: 28, y: 20, kind: WorldObjectKind::Bush });
    objects.push(ObjectPlacement { x: 19, y: 27, kind: WorldObjectKind::Bush });
    objects.push(ObjectPlacement { x: 28, y: 27, kind: WorldObjectKind::Bush });

    let forage_points = vec![
        (2, 40), (16, 42), (44, 46), (1, 46),
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
// Beach map: 32x32
// Layout: sand area with ocean on the south, dock on east, path to town on west
// ---------------------------------------------------------------------------
fn generate_beach() -> MapDef {
    let w = 32usize;
    let h = 32usize;
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

    // Ocean water on bottom half with wave-like shoreline
    for y in 20..h {
        for x in 0..w {
            // Create a slightly wavy shoreline
            let shore_y = 20.0 + 1.5 * (x as f32 * 0.3).sin();
            if y as f32 >= shore_y {
                tiles[y * w + x] = TileKind::Water;
            }
        }
    }

    // Dock extending into the water (wood floor)
    fill_rect(&mut tiles, 24, 14, 3, 14, TileKind::WoodFloor);
    // Dock end platform
    fill_rect(&mut tiles, 23, 26, 5, 3, TileKind::WoodFloor);

    // Grass strip along the top (transition to land)
    fill_rect(&mut tiles, 0, 0, 32, 4, TileKind::Grass);

    // Path from town (west side)
    fill_rect(&mut tiles, 0, 14, 6, 4, TileKind::Path);

    // Small rocky area near cliff (north-east corner)
    fill_rect(&mut tiles, 26, 2, 4, 4, TileKind::Stone);

    // Tidal pools
    fill_rect(&mut tiles, 4, 17, 2, 2, TileKind::Water);
    fill_rect(&mut tiles, 14, 18, 2, 1, TileKind::Water);

    let transitions = vec![
        // West exit -> Town
        MapTransition {
            from_map: MapId::Beach,
            from_rect: (0, 14, 1, 4),
            to_map: MapId::Town,
            to_pos: (46, 44),
        },
    ];

    let mut objects = Vec::new();
    // Rocks on the rocky area
    objects.push(ObjectPlacement { x: 27, y: 3, kind: WorldObjectKind::Rock });
    objects.push(ObjectPlacement { x: 28, y: 4, kind: WorldObjectKind::Rock });
    // Driftwood
    objects.push(ObjectPlacement { x: 10, y: 16, kind: WorldObjectKind::Log });
    objects.push(ObjectPlacement { x: 18, y: 15, kind: WorldObjectKind::Log });

    let forage_points = vec![
        (6, 12), (12, 10), (20, 8), (8, 16), (16, 14),
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
// Forest map: 40x40
// Layout: dense trees with clearings, winding paths, river, forage areas
// ---------------------------------------------------------------------------
fn generate_forest() -> MapDef {
    let w = 40usize;
    let h = 40usize;
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

    // Winding path from west (farm entrance) to east, with a split going south
    // Main east-west path
    fill_rect(&mut tiles, 0, 18, 18, 3, TileKind::Path);
    fill_rect(&mut tiles, 18, 16, 6, 3, TileKind::Path); // slight curve up
    fill_rect(&mut tiles, 24, 14, 16, 3, TileKind::Path);

    // South branch path
    fill_rect(&mut tiles, 14, 21, 3, 12, TileKind::Path);

    // River running north-south on the east side
    for y in 0..h {
        let river_x = 32.0 + 2.0 * (y as f32 * 0.15).sin();
        let rx = river_x as usize;
        for dx in 0..3 {
            if rx + dx < w {
                tiles[y * w + rx + dx] = TileKind::Water;
            }
        }
    }

    // Bridge over river
    fill_rect(&mut tiles, 31, 14, 5, 3, TileKind::Bridge);

    // Clearings (open grass areas)
    fill_rect(&mut tiles, 6, 6, 8, 6, TileKind::Grass);   // North clearing
    fill_rect(&mut tiles, 20, 26, 8, 8, TileKind::Grass);  // South clearing
    fill_rect(&mut tiles, 4, 30, 6, 6, TileKind::Dirt);    // Mushroom grove (dirt floor)

    let transitions = vec![
        // West exit -> Farm
        MapTransition {
            from_map: MapId::Forest,
            from_rect: (0, 18, 1, 4),
            to_map: MapId::Farm,
            to_pos: (62, 28),
        },
    ];

    // Dense tree cover
    let mut objects = Vec::new();

    // Top dense trees
    for x in (0..40).step_by(2) {
        for y in (0..6).step_by(2) {
            // Leave clearings open
            if x >= 6 && x < 14 && y >= 4 {
                continue;
            }
            objects.push(ObjectPlacement { x: x as i32, y: y as i32, kind: WorldObjectKind::Tree });
        }
    }

    // Trees between paths (scattered)
    let tree_positions = [
        (1, 8), (3, 10), (1, 14), (3, 16), (5, 14),
        (10, 12), (12, 10), (16, 8), (18, 10),
        (22, 8), (26, 10), (28, 8), (30, 10),
        (1, 22), (3, 24), (5, 26), (7, 24), (9, 22),
        (18, 24), (12, 28), (10, 32), (8, 34),
        (20, 34), (22, 36), (26, 34), (28, 36),
        (36, 4), (38, 6), (36, 10), (38, 12),
        (36, 20), (38, 22), (36, 28), (38, 30),
        (36, 34), (38, 36),
    ];
    for (tx, ty) in &tree_positions {
        objects.push(ObjectPlacement { x: *tx, y: *ty, kind: WorldObjectKind::Tree });
    }

    // Rocks near river
    objects.push(ObjectPlacement { x: 30, y: 6, kind: WorldObjectKind::Rock });
    objects.push(ObjectPlacement { x: 31, y: 20, kind: WorldObjectKind::Rock });
    objects.push(ObjectPlacement { x: 30, y: 30, kind: WorldObjectKind::Rock });

    // Stumps in clearings
    objects.push(ObjectPlacement { x: 8, y: 8, kind: WorldObjectKind::Stump });
    objects.push(ObjectPlacement { x: 24, y: 30, kind: WorldObjectKind::Stump });

    // Bushes
    let bush_positions = [
        (2, 12), (8, 14), (18, 22), (26, 24), (4, 36), (14, 34),
    ];
    for (bx, by) in &bush_positions {
        objects.push(ObjectPlacement { x: *bx, y: *by, kind: WorldObjectKind::Bush });
    }

    // Rich forageable spawning
    let forage_points = vec![
        (7, 7), (9, 9), (11, 7), (13, 9),     // North clearing
        (5, 31), (6, 33), (7, 35), (8, 31),    // Mushroom grove
        (21, 27), (23, 29), (25, 31), (27, 27), // South clearing
        (15, 22), (15, 26), (15, 30),           // Along south path
        (2, 20), (36, 16), (28, 12),            // Random forest spots
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
// Mine Entrance: 24x24
// Layout: mountain rock walls, cave entrance, path from farm
// ---------------------------------------------------------------------------
fn generate_mine_entrance() -> MapDef {
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

    // Open ground area (dirt and grass)
    fill_rect(&mut tiles, 4, 8, 16, 14, TileKind::Dirt);
    fill_rect(&mut tiles, 6, 16, 12, 6, TileKind::Grass);

    // Cave entrance (dark void area)
    fill_rect(&mut tiles, 9, 2, 6, 4, TileKind::Void);
    // Stone frame around cave
    fill_rect(&mut tiles, 8, 1, 8, 1, TileKind::Stone);
    fill_rect(&mut tiles, 8, 6, 8, 2, TileKind::Stone);

    // Path from cave to the open area
    fill_rect(&mut tiles, 10, 6, 4, 4, TileKind::Path);

    // Path leading east to farm
    fill_rect(&mut tiles, 20, 10, 4, 4, TileKind::Path);

    // Small water feature (mountain spring)
    fill_rect(&mut tiles, 4, 10, 2, 3, TileKind::Water);

    let transitions = vec![
        // East exit -> Farm
        MapTransition {
            from_map: MapId::MineEntrance,
            from_rect: (23, 10, 1, 4),
            to_map: MapId::Farm,
            to_pos: (1, 10),
        },
        // Cave entrance -> Mine floor 1
        MapTransition {
            from_map: MapId::MineEntrance,
            from_rect: (10, 2, 4, 2),
            to_map: MapId::Mine,
            to_pos: (8, 14),
        },
    ];

    let mut objects = Vec::new();
    // Large rocks around the mountain
    let rock_positions = [
        (2, 4), (3, 6), (18, 4), (19, 6), (6, 8), (17, 8),
        (2, 14), (20, 14), (8, 20), (15, 20),
    ];
    for (rx, ry) in &rock_positions {
        objects.push(ObjectPlacement { x: *rx, y: *ry, kind: WorldObjectKind::LargeRock });
    }

    let forage_points = vec![(6, 18), (16, 18), (12, 20)];

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
// Player House: 16x16 interior
// Layout: wood floor, bed, table, fireplace, door
// ---------------------------------------------------------------------------
fn generate_player_house() -> MapDef {
    let w = 16usize;
    let h = 16usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

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

    // Walls (void) around the perimeter
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Stone fireplace on north wall
    fill_rect(&mut tiles, 6, 1, 4, 2, TileKind::Stone);

    // Rug / carpet area (use path as rug)
    fill_rect(&mut tiles, 5, 6, 6, 4, TileKind::Path);

    // Door at bottom center
    fill_rect(&mut tiles, 7, 15, 2, 1, TileKind::WoodFloor);

    let transitions = vec![
        // Door -> Farm
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
// General Store: 12x12 interior
// ---------------------------------------------------------------------------
fn generate_general_store() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    // Walls
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Counter (stone)
    for x in 3..9 {
        tiles[3 * w + x] = TileKind::Stone;
    }

    // Shelves along walls (stone)
    for y in 1..4 {
        tiles[y * w + 1] = TileKind::Stone;
        tiles[y * w + (w - 2)] = TileKind::Stone;
    }

    // Door at bottom center
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

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
// Animal Shop: 12x12 interior
// ---------------------------------------------------------------------------
fn generate_animal_shop() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::WoodFloor; w * h];

    // Walls
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Hay/feed storage area (dirt floor in corner)
    for y in 1..4 {
        for x in 1..4 {
            tiles[y * w + x] = TileKind::Dirt;
        }
    }

    // Counter
    for x in 4..9 {
        tiles[3 * w + x] = TileKind::Stone;
    }

    // Door
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    let transitions = vec![
        MapTransition {
            from_map: MapId::AnimalShop,
            from_rect: (5, 11, 2, 1),
            to_map: MapId::Town,
            to_pos: (38, 5),
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
// Blacksmith: 12x12 interior
// ---------------------------------------------------------------------------
fn generate_blacksmith() -> MapDef {
    let w = 12usize;
    let h = 12usize;
    let mut tiles = vec![TileKind::Stone; w * h];

    // Walls
    for x in 0..w {
        tiles[x] = TileKind::Void;
        tiles[(h - 1) * w + x] = TileKind::Void;
    }
    for y in 0..h {
        tiles[y * w] = TileKind::Void;
        tiles[y * w + (w - 1)] = TileKind::Void;
    }

    // Work floor
    for y in 1..(h - 1) {
        for x in 1..(w - 1) {
            tiles[y * w + x] = TileKind::Stone;
        }
    }

    // Forge area (hot! use dirt to represent heated stone)
    for y in 1..4 {
        for x in 7..10 {
            tiles[y * w + x] = TileKind::Dirt;
        }
    }

    // Anvil area (wood floor)
    tiles[5 * w + 3] = TileKind::WoodFloor;
    tiles[5 * w + 4] = TileKind::WoodFloor;
    tiles[6 * w + 3] = TileKind::WoodFloor;
    tiles[6 * w + 4] = TileKind::WoodFloor;

    // Counter
    for x in 2..7 {
        tiles[3 * w + x] = TileKind::WoodFloor;
    }

    // Door
    tiles[11 * w + 5] = TileKind::WoodFloor;
    tiles[11 * w + 6] = TileKind::WoodFloor;

    let transitions = vec![
        MapTransition {
            from_map: MapId::Blacksmith,
            from_rect: (5, 11, 2, 1),
            to_map: MapId::Town,
            to_pos: (41, 31),
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
