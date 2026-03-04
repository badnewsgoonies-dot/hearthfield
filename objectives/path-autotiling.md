# Worker: Path Autotiling

## Context
Every path tile in Hearthfield currently renders as the same crossroads sprite (index 5 of `assets/sprites/paths.png`). The atlas is 4×4 (16 tiles of 16×16px). We need neighbor-aware autotiling so paths form connected shapes.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/world/`
All out-of-scope edits will be reverted.

## Required reading
1. `src/world/mod.rs` — especially:
   - `fn tile_atlas_info(kind, season, atlases, map_id)` (~line 230+) — currently returns hardcoded index 5 for Path
   - `fn spawn_tile_sprites(commands, map_def, season, atlases)` (~line 515) — iterates tiles and calls tile_atlas_info
   - The `TerrainAtlases` struct (paths_image, paths_layout fields)
2. `src/shared/mod.rs` — READ ONLY. Find `TileKind`, `MapDef` (with `tiles: Vec<TileKind>`, `width`, `height`)

## Algorithm: 4-bit Cardinal Bitmask

For each Path tile at (x, y), check 4 cardinal neighbors:
- North (x, y+1): if Path → set bit 0 (value 1)
- East  (x+1, y): if Path → set bit 1 (value 2)
- South (x, y-1): if Path → set bit 2 (value 4)
- West  (x-1, y): if Path → set bit 3 (value 8)

Out-of-bounds neighbors count as NOT path (bit = 0).
Bridge tiles should also count as path-connected (they connect to paths).

The bitmask (0-15) maps to a tile index. The standard mapping for a 4×4 path atlas laid out in "blob" style is:

```
Row 0: isolated(0), end-N(1), end-E(2), corner-NE(3)
Row 1: end-S(4), vert(5), corner-SE(6), tee-E(7)
Row 2: end-W(8), corner-NW(9), horiz(10), tee-N(11)
Row 3: corner-SW(12), tee-W(13), tee-S(14), cross(15)
```

Where the bitmask-to-index mapping is:
```rust
fn path_autotile_index(bitmask: u8) -> usize {
    match bitmask {
        0b0000 => 0,   // isolated (no neighbors)
        0b0001 => 1,   // only north
        0b0010 => 2,   // only east
        0b0011 => 3,   // north + east
        0b0100 => 4,   // only south
        0b0101 => 5,   // north + south (vertical)
        0b0110 => 6,   // east + south
        0b0111 => 7,   // north + east + south (tee facing east)
        0b1000 => 8,   // only west
        0b1001 => 9,   // north + west
        0b1010 => 10,  // east + west (horizontal)
        0b1011 => 11,  // north + east + west (tee facing north)
        0b1100 => 12,  // south + west
        0b1101 => 13,  // north + south + west (tee facing west)
        0b1110 => 14,  // east + south + west (tee facing south)
        0b1111 => 15,  // all four (crossroads)
        _ => 5,        // fallback to vertical
    }
}
```

NOTE: This mapping assumes the atlas follows the standard blob pattern. If the visual result looks wrong (e.g., corners are swapped), the mapping may need adjustment — but ship this first and we'll tune later.

## Implementation Steps

### Step 1: Modify `tile_atlas_info` signature
Add parameters for neighbor context:
```rust
fn tile_atlas_info(
    kind: TileKind,
    _season: Season,
    atlases: &TerrainAtlases,
    map_id: MapId,
    // NEW: autotile context
    x: usize,
    y: usize,
    tiles: &[TileKind],
    width: usize,
    height: usize,
) -> Option<(Handle<Image>, Handle<TextureAtlasLayout>, usize)>
```

### Step 2: Add the bitmask helper
Add the `path_autotile_index` function and a neighbor-check helper:
```rust
fn is_path_neighbor(tiles: &[TileKind], x: usize, y: usize, width: usize, height: usize, dx: i32, dy: i32) -> bool {
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
        return false;
    }
    let tile = tiles[ny as usize * width + nx as usize];
    matches!(tile, TileKind::Path | TileKind::Bridge)
}
```

### Step 3: In the Path arm of `tile_atlas_info`
Replace the hardcoded `5` with bitmask computation:
```rust
TileKind::Path => {
    let mut mask: u8 = 0;
    if is_path_neighbor(tiles, x, y, width, height, 0, 1)  { mask |= 1; } // north
    if is_path_neighbor(tiles, x, y, width, height, 1, 0)  { mask |= 2; } // east
    if is_path_neighbor(tiles, x, y, width, height, 0, -1) { mask |= 4; } // south
    if is_path_neighbor(tiles, x, y, width, height, -1, 0) { mask |= 8; } // west
    let index = path_autotile_index(mask);
    Some((
        atlases.paths_image.clone(),
        atlases.paths_layout.clone(),
        index,
    ))
}
```

### Step 4: Update the call site in `spawn_tile_sprites`
Pass the new parameters:
```rust
match tile_atlas_info(tile, season, atlases, map_def.id, x, y, &map_def.tiles, map_def.width, map_def.height) {
```

## IMPORTANT: Y-axis direction
Check which direction y increases. The code uses `tiles[y * width + x]` and the MANIFEST says "y=0 is back wall (north), y=h-1 is front/door (south)". So:
- y+1 = one row toward south (NOT north in world terms)
- y-1 = one row toward north

Adjust the bitmask bits accordingly:
- North neighbor check: (x, y-1) — set bit 0
- South neighbor check: (x, y+1) — set bit 2

Or just use the index mapping consistently — what matters is that the visual result is correct connections, not the label.

## Do NOT
- Modify src/shared/mod.rs
- Modify any files outside src/world/
- Change other tile types (Grass, Sand, etc.)
- Add new tile kinds or atlas files

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write completion report to status/workers/path-autotiling.md listing:
- Functions added/modified
- How the bitmask mapping works
- Which tiles count as "path neighbor" (Path + Bridge)
