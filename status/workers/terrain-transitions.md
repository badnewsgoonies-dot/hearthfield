# Worker Report: Terrain Edge Transitions

## Summary
Added terrain edge transitions to eliminate blocky tile borders between grass/dirt and grass/water. Dirt and water tiles that border grass now use the appropriate transition tiles from the `modern_farm_terrain.png` atlas instead of uniform center tiles.

## Files Modified
- `src/world/mod.rs` (~120 lines added/changed)

## What Was Implemented

### 1. Generic neighbor bitmask system
- `neighbor_is()`: checks if a neighbor tile satisfies a predicate
- `neighbor_bitmask()`: computes a 4-bit NESW mask for any tile predicate
- Convention: bit0=N(+y), bit1=E(+x), bit2=S(-y), bit3=W(-x) — matches existing `water_edge_mask`

### 2. Grass-dirt transitions (16 bitmask combinations)
- `dirt_grass_transition_index()`: maps grass-neighbor bitmask to atlas index
- Uses the 3x3 core block (cols 0-2, rows 0-2) for single edges and 2-edge corners
- Uses extended tiles (cols 3-4, rows 0-2 and row 3 cols 3-4) for 3-edge and opposing-edge combos
- Dirt tiles with no grass neighbors still use idx 291 (plain dirt)

### 3. Grass-water transitions (16 bitmask combinations)
- `water_grass_transition_index()`: maps land-neighbor bitmask to atlas index
- Uses the 3x3 core block (cols 16-18, rows 0-2) and extended tiles (cols 19-20)
- Pure water centers (no land neighbors) now use `water.png` atlas for proper 4-frame animation
- Transition water tiles use `modern_farm_terrain.png` and are static (no animation cycling)

### 4. Water animation fix
- Added `WaterBaseIndex` component to track the base atlas index for animatable water tiles
- Only pure water center tiles (using `water.png`) receive animation cycling
- Transition tiles with grass edges are excluded from animation to avoid index corruption
- Fixed a pre-existing bug where `(atlas.index + 1) % 4` would cycle all water tiles through unrelated atlas indices

### 5. Improved water edge overlays
- Changed from single 4px rectangle per edge to dual-layer feathered overlays:
  - Inner band: 3px wide, alpha 0.25
  - Outer band: 6px wide, alpha 0.12
- Reduced pulse alpha range (0.18-0.30 vs old 0.30-0.50) for subtler animation

## Atlas Index Mapping

### Dirt transition tiles (grass-neighbor bitmask -> atlas index)
| Mask | Direction(s) | Index | Atlas Position |
|------|-------------|-------|----------------|
| 0000 | none | 33 | r1c1 (center) |
| 0001 | N | 1 | r0c1 |
| 0010 | E | 34 | r1c2 |
| 0100 | S | 65 | r2c1 |
| 1000 | W | 32 | r1c0 |
| 0011 | N+E | 2 | r0c2 |
| 0110 | S+E | 66 | r2c2 |
| 1100 | S+W | 64 | r2c0 |
| 1001 | N+W | 0 | r0c0 |
| 0101 | N+S | 99 | r3c3 |
| 1010 | E+W | 100 | r3c4 |
| 0111 | N+E+S | 4 | r0c4 |
| 1011 | N+E+W | 3 | r0c3 |
| 1101 | N+S+W | 35 | r1c3 |
| 1110 | E+S+W | 36 | r1c4 |
| 1111 | all | 33 | r1c1 |

### Water transition tiles (land-neighbor bitmask -> atlas index)
Same layout offset to cols 16-22 (add 16 to dirt block indices for corresponding positions).

## Validation Results
- `cargo check`: PASS
- `cargo test --test headless`: PASS (128 passed, 0 failed, 2 ignored)
- `cargo clippy -- -D warnings`: PASS (0 warnings)

## Shared Type Imports Used
- `TileKind`, `Season`, `MapId`, `TILE_SIZE`, `Z_GROUND`, `world_to_grid`
- No changes to `src/shared/mod.rs`

## Known Risks
- The N/S direction mapping assumes top-of-atlas = north on screen. If the Y convention is inverted in some maps, transitions would appear mirrored (N/S swapped). The existing `water_edge_mask` uses the same convention, so this should be consistent.
- Transition tile colors from the atlas (lighter dirt ~170,143,90) differ slightly from the plain dirt tile (idx 291, ~112,83,55). This is inherent to the atlas art and not a code issue.
- The `handle_season_change` system re-calls `tile_atlas_info` which correctly returns transition tiles, but does not re-set `custom_size` on the sprite (pre-existing limitation).
