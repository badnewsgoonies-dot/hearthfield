# Worker: WORLD — Water Edge Autotile Transitions (M6)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/world/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/world/mod.rs — tile rendering system, tile_atlas_info(), spawn_map_tiles(), WaterTile component, WaterAnimationTimer, animate_water_tiles()
2. src/world/maps.rs — MapDef, generate_map(), TileKind layout per map
3. src/shared/mod.rs — TileKind enum, MapId, WorldMap, TerrainAtlases

## Context
Currently water tiles are rendered as flat blue sprites using `tilesets/water.png` atlas index 0. There are 4 frames in the water atlas (for animation), but NO edge/transition tiles. Water bodies have hard rectangular borders — water tile next to grass tile is a jarring pixel boundary.

Available asset: `tilesets/water.png` — 4 frames for animation. No dedicated water-edge tileset exists.

## Deliverables

### 1. Procedural water edge blending
Since no water-edge tileset exists, implement edge blending using **alpha-blended overlay sprites**:
- For each water tile that borders a non-water tile, spawn an additional overlay sprite on top
- The overlay uses a semi-transparent gradient (water color fading to transparent) on the edge sides
- Use Bevy's `Sprite` with `color` set to water blue with alpha, positioned at the tile edge

### 2. Edge detection system
Add a system that runs after map spawn (or during spawn) that checks each water tile's 4 cardinal neighbors:
- If neighbor is non-water (grass, sand, dirt, etc.), mark that edge as needing a transition
- Use a bitmask approach: bit 0=north, 1=east, 2=south, 3=west
- Store as a component `WaterEdgeMask(u8)` on the water tile entity

### 3. Spawn edge overlay sprites
For each water tile with WaterEdgeMask != 0, spawn thin overlay sprites:
- North edge (bit 0): 16×4 sprite at tile top, water color with alpha 0.4, z = Z_GROUND + 0.1
- East edge (bit 1): 4×16 sprite at tile right edge
- South edge (bit 2): 16×4 sprite at tile bottom
- West edge (bit 3): 4×16 sprite at tile left edge
- Corner overlaps are fine (they'll blend)
- Tag overlays with MapTile so they despawn with the map

### 4. Make edge overlays participate in water animation
The existing animate_water_tiles system cycles atlas index for WaterTile entities. Edge overlays don't have atlas frames, but should pulse alpha slightly (0.3 to 0.5) in sync with the water animation timer to feel alive.

## Quantitative targets
- Every water tile bordering non-water must have edge overlays
- Edge overlays must be tagged MapTile for proper cleanup
- Water animation timer must affect edge overlay alpha
- Zero clippy warnings

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/world_water_edges_wave8.md
