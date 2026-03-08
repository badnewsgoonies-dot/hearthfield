# Worker: World Domain (Wave 1)

## Scope (mechanically enforced)
You may only create/modify files under: `dlc/police/src/domains/world/`

## Required reading (read BEFORE writing code)
1. `dlc/police/docs/spec.md`
2. `dlc/police/docs/domains/world.md` (CRITICAL)
3. `dlc/police/src/shared/mod.rs`
4. `dlc/police/src/main.rs`

## Interpretation contract
Before coding, extract from world.md:
- Hardcoded Rust arrays for map data (not RON files)
- Colored rectangles for tiles (not sprite atlas)
- CollisionMap as HashSet of solid positions

## Required imports (from crate::shared)
- `MapId`, `TileKind`, `GridPosition`, `MapTransition`, `MapTransitionEvent`
- `PlayerState`
- `GameState`, `UpdatePhase`
- `TILE_SIZE`, `PIXEL_SCALE`

## Deliverables
Create `dlc/police/src/domains/world/mod.rs` containing:
- `pub struct WorldPlugin;` implementing `Plugin`
- `CollisionMap` resource: `HashSet<(i32, i32)>` of solid tile positions
- `MapTile` component: marker on tile entities for cleanup
- `precinct_interior_data()` — returns 32x24 grid of TileKind values
  - Layout: outer walls, central hallway, 6 rooms (lobby, case board, evidence room, captain's office, break room, locker room), doors between rooms and hallway
  - Transition zone at bottom-center → PrecinctExterior
- `spawn_map` — OnEnter(GameState::Playing)
  - Read PlayerState.position_map
  - Spawn tile entities: each tile gets Sprite (colored by kind) + Transform + MapTile
  - Colors: Floor=#3a3a4a, Wall=#2a1a0a, Door=#5a3a1a, Sidewalk=#6a6a7a, Road=#4a4a4a, Grass=#2a4a2a, Interactable=#4a4a6a
  - World position: x = grid_x * TILE_SIZE * PIXEL_SCALE, y = grid_y * TILE_SIZE * PIXEL_SCALE
  - Z-ordering: floor at z=0, walls at z=1
  - Populate CollisionMap with Wall positions
- `handle_map_transition` — read MapTransitionEvent
  - Despawn all MapTile entities
  - Update PlayerState.position_map
  - Respawn map for new MapId (Wave 1: only PrecinctInterior exists, log warning for others)
- `cleanup_map` — OnExit(GameState::Playing) — despawn all MapTile entities

## Quantitative targets
- PrecinctInterior: exactly 32x24 = 768 tiles
- 6 named rooms connected by doors
- CollisionMap contains all Wall tile coordinates
- 1 transition zone defined (south entrance)

## Failure patterns to avoid
- Creating a RON file parser or asset loader
- Loading PNG textures or sprite sheets
- Making CollisionMap a Component instead of a Resource
- Forgetting to populate CollisionMap during spawn_map
- Using f32 for collision coordinates (use i32 grid positions)

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```

## When done
Write report to `dlc/police/status/workers/world.md`
