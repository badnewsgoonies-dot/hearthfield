# Worker Report: Path Autotiling

## Files Modified
- `src/world/mod.rs`

## Functions Added/Modified

### Added: `is_path_neighbor`
Helper that checks whether a cardinal neighbor tile (offset by `dx, dy`) is a path-connected tile, returning `false` for out-of-bounds positions.

### Added: `path_autotile_index`
Maps a 4-bit cardinal bitmask (0–15) to a sprite atlas index in `paths.png`. Each bit represents a cardinal neighbor: bit 0 = north, bit 1 = east, bit 2 = south, bit 3 = west.

### Modified: `tile_atlas_info`
Added 5 new parameters: `x: usize`, `y: usize`, `tiles: &[TileKind]`, `width: usize`, `height: usize`. The `TileKind::Path` arm now computes the 4-bit bitmask and calls `path_autotile_index` instead of returning hardcoded index 5.

Both call sites updated:
- `spawn_tile_sprites`: passes `x`, `y`, `&map_def.tiles`, `map_def.width`, `map_def.height`
- `handle_season_change`: passes `gx as usize`, `gy as usize`, `&map_def.tiles`, `map_def.width`, `map_def.height`

## Bitmask Mapping
- Bit 0 (value 1): north neighbor (y-1)
- Bit 1 (value 2): east neighbor (x+1)
- Bit 2 (value 4): south neighbor (y+1)
- Bit 3 (value 8): west neighbor (x-1)

Out-of-bounds neighbors count as 0 (not path). The 16 possible bitmask values map directly to atlas indices 0–15.

## Path-connected Tile Types
Both `TileKind::Path` and `TileKind::Bridge` count as path neighbors (bridges connect to paths visually).

## Validation
`cargo check` — passed, zero errors.
