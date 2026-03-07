# Data-Driven Maps — Design Spec

## Goal

Replace all hardcoded `generate_*()` functions in `src/world/maps.rs` with RON files loaded from `assets/maps/`. The tile layouts, object placements, transitions, forage points, spawn positions, and door triggers should all live in `.ron` data files so non-programmers can edit maps without recompiling.

## Data Format

Each map gets one RON file: `assets/maps/{map_id}.ron`

```ron
MapData(
    id: Farm,
    width: 32,
    height: 24,
    // Row-major tile grid. One TileKind per cell, y*width+x ordering.
    tiles: [Grass, Grass, ..., Stone, Stone, ...],
    // World objects (trees, rocks, stumps, etc.)
    objects: [
        (x: 1, y: 1, kind: Tree),
        (x: 8, y: 7, kind: Rock),
    ],
    // Forageable spawn positions
    forage_points: [(2, 4), (29, 3)],
    // Default player spawn position for this map
    spawn_pos: (16, 12),
    // Transitions: walking onto from_rect teleports to (to_map, to_x, to_y)
    transitions: [
        (from_rect: (13, 23, 5, 1), to_map: Town, to_x: 12, to_y: 1),
        (from_rect: (31, 8, 1, 4), to_map: Forest, to_x: 1, to_y: 7),
    ],
    // Door triggers: walking onto (x_range, y) warps to interior map
    doors: [
        (x_min: 15, x_max: 16, y: 2, to_map: PlayerHouse, to_x: 8, to_y: 14),
    ],
    // Edge transitions: which maps border this one on each edge
    edges: (
        north: None,
        south: Some((Town, ClampX)),
        east: Some((Forest, ClampY)),
        west: Some((MineEntrance, Fixed(12, 6))),
    ),
    // Building visual definitions for this map
    buildings: [
        (x: 13, y: 0, w: 6, h: 3, roof_tint: (0.6, 0.3, 0.2)),
    ],
)
```

## Shared Types (already in src/shared/mod.rs — DO NOT MODIFY)

These already have `Serialize, Deserialize`:
- `MapId` — Farm, Town, Beach, Forest, MineEntrance, Mine, PlayerHouse, GeneralStore, AnimalShop, Blacksmith
- `TileKind` — Grass, Dirt, TilledSoil, WateredSoil, Water, Sand, Stone, WoodFloor, Path, Bridge, Void

## New Types (define in src/world/maps.rs or a new src/world/map_data.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapData {
    pub id: MapId,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileKind>,
    pub objects: Vec<ObjectDef>,
    pub forage_points: Vec<(i32, i32)>,
    pub spawn_pos: (i32, i32),
    pub transitions: Vec<TransitionDef>,
    pub doors: Vec<DoorDef>,
    pub edges: EdgeDefs,
    pub buildings: Vec<BuildingDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectDef {
    pub x: i32,
    pub y: i32,
    pub kind: WorldObjectKind,  // Must add Serialize/Deserialize to this enum
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionDef {
    pub from_rect: (i32, i32, i32, i32),  // (x, y, w, h)
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorDef {
    pub x_min: i32,
    pub x_max: i32,
    pub y: i32,
    pub to_map: MapId,
    pub to_x: i32,
    pub to_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeTarget {
    ClampX,       // clamp player x to target map width
    ClampY,       // clamp player y to target map height
    Fixed(i32, i32),  // fixed spawn position
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefs {
    pub north: Option<(MapId, EdgeTarget)>,
    pub south: Option<(MapId, EdgeTarget)>,
    pub east: Option<(MapId, EdgeTarget)>,
    pub west: Option<(MapId, EdgeTarget)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDef {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub roof_tint: (f32, f32, f32),
}
```

## Implementation Plan

### Worker 1: Export + Loader (src/world/)
1. Add `Serialize, Deserialize` to `WorldObjectKind`
2. Create `MapData` struct and related types in `src/world/map_data.rs`
3. Write an export function that converts each existing `generate_*()` output to RON and writes to `assets/maps/`
4. Write `load_map_data(map_id: MapId) -> MapData` that reads RON from disk (or embedded)
5. Convert `generate_map()` to call the loader instead of hardcoded functions
6. Keep `MapDef` as the runtime type, convert `MapData` → `MapDef` at load time
7. Keep the `generate_*()` functions as fallbacks (if RON file missing, use hardcoded)

### Worker 2: Data-Drive Transitions (src/player/)
1. Replace hardcoded `edge_transition()` and `map_bounds()` in `src/player/interaction.rs`
2. Load door triggers and edge transitions from `MapData` instead
3. Store loaded map data in a `MapRegistry` resource so `interaction.rs` can query it
4. `map_bounds()` reads width/height from `MapRegistry`
5. `edge_transition()` checks doors first, then edges, all from data

### Worker 3: NPC Schedule Coordinates (src/npcs/) — FUTURE
NPC schedules reference specific (MapId, x, y) positions. These stay hardcoded for now but a future pass could extract them to RON. The map data change doesn't break schedules because MapId values don't change.

## Files Changed

- `src/world/map_data.rs` — NEW: MapData types + loader
- `src/world/maps.rs` — MODIFIED: generate_map() calls loader, fallback to hardcoded
- `src/world/mod.rs` — MODIFIED: register MapRegistry resource
- `src/player/interaction.rs` — MODIFIED: read transitions from MapRegistry
- `assets/maps/*.ron` — NEW: 10 map data files
- `Cargo.toml` — UNCHANGED (ron already a dependency)

## Validation

```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
shasum -a 256 -c .contract.sha256
```

All must pass. The RON files must produce identical MapDef output to the hardcoded generators.
