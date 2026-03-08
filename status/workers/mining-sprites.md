# Mining Sprites Worker Report

## Summary
Replaced colored-rectangle placeholders in the mining domain with real sprites from the Fungus Cave tileset (`assets/tilesets/fungus_cave.png`).

## Files Modified
- `src/mining/spawning.rs` — Major rewrite: replaced `MiningAtlas` (single atlas) with `MiningAtlases` (dual atlas: cave environment + rock/ore sprites), added `cave_tiles` module with tile index constants, updated all spawn functions
- `src/mining/rock_breaking.rs` — Updated ladder reveal to swap atlas sprite instead of changing color, added `MiningAtlases` resource parameter
- `src/mining/mod.rs` — Updated resource registration from `MiningAtlas` to `MiningAtlases`

## Architecture
- `MiningAtlases` resource holds two atlas pairs:
  - **Cave atlas**: `tilesets/fungus_cave.png` (8x35 grid, 280 tiles) for floor, wall, ladder, exit
  - **Rock atlas**: `sprites/mining_atlas.png` (8x6 grid, 48 tiles) for ore/gem rock sprites
- `cave_tiles` module defines named constants for tile indices (FLOOR=164, FLOOR_ALT=165, WALL=24, LADDER=64, EXIT=264)
- Fallback colored-rectangle rendering preserved for when atlases haven't loaded yet

## Changes Detail
1. **Floor tiles**: Now use `cave_tiles::FLOOR` (164) and `cave_tiles::FLOOR_ALT` (165) from fungus_cave.png for checkerboard pattern
2. **Wall tiles**: Now use `cave_tiles::WALL` (24) — brick/stone wall tiles
3. **Rock sprites**: Unchanged, still use `mining_atlas.png` indices (copper=8, iron=9, gold=11, etc.)
4. **Ladder**: Uses `cave_tiles::LADDER` (64) when revealed, `cave_tiles::FLOOR` (164) when hidden
5. **Exit tile**: Uses `cave_tiles::EXIT` (264) without green color tint
6. **Ladder reveal**: Now swaps the entire sprite (image + atlas index) instead of just changing color

## Validation
- `cargo check` — PASS
- `cargo test --test headless` — 128 passed, 0 failed, 2 ignored
- `cargo clippy -- -D warnings` — PASS (zero warnings)

## Shared Type Imports Used
- `TILE_SIZE`, `GameState`, `MineRock`, `MineMonster`, `MineEnemy`, `ToolUseEvent`, `ToolKind`, `ToolTier`, `ItemPickupEvent`, `PlaySfxEvent`, `StaminaDrainEvent`, `MonsterSlainEvent`

## Known Risks
- Tile indices (164, 165, 24, 64, 264) chosen by visual inspection of fungus_cave.png — may need adjustment if the tileset layout differs from what was inspected
- Enemy sprites still use the separate `mine_enemies.png` atlas (unchanged)
