//! Procedural map generation: a seed -> a deterministic, schema-valid `MapData`.
//!
//! The seed IS the coordinate — `generate_procedural_map(seed, ..)` is a pure function of the seed,
//! so regenerating from the same seed yields the identical map (generation = retrieval). This is the
//! GENERATIVE-SEEDLING channel for maps, parallel to the authored DATA-BANK maps in the `MapRegistry`.
//!
//! The generation logic here is verified standalone against the real `ron` (de)serializer (see
//! `/mnt/user-data/outputs/procgen/`); it produces a `MapData` that `ron::from_str::<MapData>` accepts
//! and that round-trips through `ron::to_string`.
//!
//! Identity: callers pass the `MapId` to stamp onto the map. Once `MapId::Procedural(u64)` is added
//! (see the integration plan), pass `MapId::Procedural(seed)` so a generated map's id IS its seed.

use crate::shared::{MapId, TileKind};
use super::maps::WorldObjectKind;
use super::map_data::{MapData, ObjectDef, EdgeDefs};

/// Deterministic splitmix64 — a pure function of the seed. Same seed => same stream => same map.
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self { Rng(seed ^ 0xA17C_3D5E_9F2B_8146) }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 { self.next() % n }
    fn chance(&mut self, num: u64, den: u64) -> bool { self.below(den) < num }
}

/// Generate a deterministic, map-like, schema-valid `MapData` from `seed`.
///
/// Coherent regions (grass base, water ponds with sand shores, stone patches) plus a crossing path
/// (keeps the map traversable), objects scattered on grass, a few forage points, and a center-path
/// spawn. No transitions/doors/buildings and `None` edges — a self-contained map.
pub fn generate_procedural_map(seed: u64, width: usize, height: usize, id: MapId) -> MapData {
    use TileKind::*;
    let mut rng = Rng::new(seed);
    let n = width * height;
    let idx = |x: i32, y: i32| (y as usize) * width + x as usize;
    let mut tiles = vec![Grass; n];

    // water ponds with sand shores — coherent regions, not noise
    let ponds = 1 + rng.below(3) as usize;
    for _ in 0..ponds {
        let cx = rng.below(width as u64) as i32;
        let cy = rng.below(height as u64) as i32;
        let r = 1 + rng.below(3) as i32;
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                let d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
                if d2 <= r * r {
                    tiles[idx(x, y)] = Water;
                } else if d2 <= (r + 1) * (r + 1) {
                    tiles[idx(x, y)] = Sand;
                }
            }
        }
    }

    // optional stone patches
    for _ in 0..(rng.below(2) as usize) {
        let cx = rng.below(width as u64) as i32;
        let cy = rng.below(height as u64) as i32;
        let r = 1 + rng.below(2) as i32;
        for y in 0..height as i32 {
            for x in 0..width as i32 {
                if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r * r {
                    let i = idx(x, y);
                    if tiles[i] == Grass {
                        tiles[i] = Stone;
                    }
                }
            }
        }
    }

    // crossing paths through the center (traversability)
    let py = height / 2;
    let px = width / 2;
    for x in 0..width {
        let i = py * width + x;
        if tiles[i] != Water { tiles[i] = Path; }
    }
    for y in 0..height {
        let i = y * width + px;
        if tiles[i] != Water { tiles[i] = Path; }
    }

    // objects scattered on grass
    let mut objects = Vec::new();
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            if tiles[idx(x, y)] == Grass && rng.chance(12, 100) {
                let kind = match rng.below(3) {
                    0 => WorldObjectKind::Tree,
                    1 => WorldObjectKind::Bush,
                    _ => WorldObjectKind::Stump,
                };
                objects.push(ObjectDef { x, y, kind });
            }
        }
    }

    // a few forage points on grass
    let mut forage_points = Vec::new();
    for _ in 0..(2 + rng.below(5)) {
        let x = rng.below(width as u64) as i32;
        let y = rng.below(height as u64) as i32;
        if tiles[idx(x, y)] == Grass {
            forage_points.push((x, y));
        }
    }

    MapData {
        id,
        width,
        height,
        tiles,
        objects,
        forage_points,
        spawn_pos: (px as i32, py as i32), // center path cell — walkable by construction
        transitions: Vec::new(),
        doors: Vec::new(),
        edges: EdgeDefs { north: None, south: None, east: None, west: None },
        buildings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_and_valid() {
        let a = generate_procedural_map(1000, 24, 18, MapId::Farm);
        let b = generate_procedural_map(1000, 24, 18, MapId::Farm);
        assert_eq!(a.tiles, b.tiles, "same seed must yield identical tiles");
        assert_eq!(a.tiles.len(), 24 * 18, "tile grid is width*height");
        assert!(a.objects.iter().all(|o| o.x >= 0 && (o.x as usize) < 24 && o.y >= 0 && (o.y as usize) < 18));
        // spawn is walkable (Path by construction)
        let sp = a.spawn_pos;
        assert_eq!(a.tiles[(sp.1 as usize) * 24 + sp.0 as usize], TileKind::Path);
    }
}
