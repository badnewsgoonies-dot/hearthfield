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

    // Biome from the seed — each seed grows a different KIND of place. Deterministic.
    let biome = seed % 4; // 0 meadow, 1 forest, 2 beach, 3 rocky

    match biome {
        2 => {
            // BEACH: sand band along the south, water margin below it.
            let shore = (height * 2) / 3;
            for y in 0..height {
                for x in 0..width {
                    let i = idx(x as i32, y as i32);
                    if y >= height.saturating_sub(2) {
                        tiles[i] = Water;
                    } else if y >= shore {
                        tiles[i] = Sand;
                    }
                }
            }
        }
        3 => {
            // ROCKY: several large stone massifs.
            for _ in 0..(2 + rng.below(3) as usize) {
                let cx = rng.below(width as u64) as i32;
                let cy = rng.below(height as u64) as i32;
                let r = 2 + rng.below(3) as i32;
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
        }
        _ => {
            // MEADOW / FOREST: a pond or two with sand shores.
            for _ in 0..(1 + rng.below(2) as usize) {
                let cx = rng.below(width as u64) as i32;
                let cy = rng.below(height as u64) as i32;
                let r = 1 + rng.below(3) as i32;
                for y in 0..height as i32 {
                    for x in 0..width as i32 {
                        let d2 = (x - cx) * (x - cx) + (y - cy) * (y - cy);
                        if d2 <= r * r {
                            tiles[idx(x, y)] = Water;
                        } else if d2 <= (r + 1) * (r + 1) {
                            let i = idx(x, y);
                            if tiles[i] == Grass {
                                tiles[i] = Sand;
                            }
                        }
                    }
                }
            }
        }
    }

    // crossing paths through the center (traversability spine) — carve over terrain, kept clear of objects.
    // Bridge over water so the spine is ALWAYS connected and the center spawn is always walkable.
    let py = height / 2;
    let px = width / 2;
    for x in 0..width {
        let i = py * width + x;
        tiles[i] = if tiles[i] == Water { Bridge } else { Path };
    }
    for y in 0..height {
        let i = y * width + px;
        tiles[i] = if tiles[i] == Water { Bridge } else { Path };
    }

    // Connectivity repair: guarantee every walkable cell is reachable from the spawn. Flood from the
    // center; for any stranded walkable region, carve a Path (Bridge over water) to the central column.
    // Each pass connects at least one stranded row to the spine, so this terminates.
    let is_walkable = |t: TileKind| !matches!(t, Water | Stone | Void);
    loop {
        let start = py * width + px;
        let mut seen = vec![false; width * height];
        let mut stack = vec![start];
        seen[start] = true;
        while let Some(i) = stack.pop() {
            let (x, y) = ((i % width) as i32, (i / width) as i32);
            for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                if nx >= 0 && (nx as usize) < width && ny >= 0 && (ny as usize) < height {
                    let j = (ny as usize) * width + nx as usize;
                    if !seen[j] && is_walkable(tiles[j]) {
                        seen[j] = true;
                        stack.push(j);
                    }
                }
            }
        }
        match (0..width * height).find(|&i| is_walkable(tiles[i]) && !seen[i]) {
            None => break,
            Some(i) => {
                let (cx, cy) = (i % width, i / width);
                let (lo, hi) = if cx < px { (cx, px) } else { (px, cx) };
                for x in lo..=hi {
                    let k = cy * width + x;
                    tiles[k] = if tiles[k] == Water { Bridge } else { Path };
                }
            }
        }
    }

    // Objects: biome-specific palette and density, placed on open ground (Grass/Sand) — never on the
    // Path spine (so it stays walkable) or on Water/Stone.
    let (density, palette): (u64, &[WorldObjectKind]) = match biome {
        0 => (10, &[WorldObjectKind::Tree, WorldObjectKind::Bush, WorldObjectKind::Stump]),
        1 => (28, &[WorldObjectKind::Tree, WorldObjectKind::Pine, WorldObjectKind::Bush, WorldObjectKind::Stump]),
        2 => (14, &[WorldObjectKind::PalmTree, WorldObjectKind::Driftwood, WorldObjectKind::Dock]),
        _ => (16, &[WorldObjectKind::Rock, WorldObjectKind::LargeRock, WorldObjectKind::Log]),
    };
    let mut objects = Vec::new();
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            if matches!(tiles[idx(x, y)], Grass | Sand) && rng.below(100) < density {
                let kind = palette[rng.below(palette.len() as u64) as usize];
                objects.push(ObjectDef { x, y, kind });
            }
        }
    }

    // a few forage points on open ground
    let mut forage_points = Vec::new();
    for _ in 0..(2 + rng.below(5)) {
        let x = rng.below(width as u64) as i32;
        let y = rng.below(height as u64) as i32;
        if matches!(tiles[idx(x, y)], Grass | Sand) {
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
        let sp = a.spawn_pos;
        let spawn_tile = a.tiles[(sp.1 as usize) * 24 + sp.0 as usize];
        assert!(matches!(spawn_tile, TileKind::Path | TileKind::Bridge), "spawn is on the walkable spine");
    }

    #[test]
    fn fully_traversable_from_spawn() {
        // Every generated map must be fully walkable from the spawn — no isolated pockets.
        // (Verified across diverse seeds standalone: spawn reaches 100% of walkable cells.)
        let walkable = |t: &TileKind| !matches!(t, TileKind::Water | TileKind::Stone | TileKind::Void);
        for seed in [1000u64, 2001, 4242, 7777, 31337] {
            let m = generate_procedural_map(seed, 24, 18, MapId::Procedural(seed));
            let (w, h) = (m.width, m.height);
            let total = m.tiles.iter().filter(|t| walkable(t)).count();
            let (sx, sy) = m.spawn_pos;
            let start = (sy as usize) * w + sx as usize;
            assert!(walkable(&m.tiles[start]), "spawn must be walkable (seed {seed})");
            let mut seen = vec![false; w * h];
            let mut stack = vec![start];
            seen[start] = true;
            let mut reached = 0usize;
            while let Some(i) = stack.pop() {
                reached += 1;
                let (x, y) = ((i % w) as i32, (i / w) as i32);
                for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
                    if nx >= 0 && (nx as usize) < w && ny >= 0 && (ny as usize) < h {
                        let j = (ny as usize) * w + nx as usize;
                        if !seen[j] && walkable(&m.tiles[j]) {
                            seen[j] = true;
                            stack.push(j);
                        }
                    }
                }
            }
            assert_eq!(reached, total, "all walkable cells reachable from spawn (seed {seed})");
        }
    }
}
