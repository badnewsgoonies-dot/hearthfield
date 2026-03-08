# Worker Report: Structure Sprites

## Files Modified
- `src/farming/mod.rs` — Added `sprinkler_image` and `scarecrow_image` handles to `FarmingAtlases`; load them in `load_farming_atlases`
- `src/farming/render.rs` — Rewrote sprinkler/scarecrow branch in `sync_farm_objects_sprites` to use standalone `Sprite::from_image()` from `FarmingAtlases`; added `FarmingAtlases` as a system parameter; marked `farm_object_atlas_index` as `#[allow(dead_code)]`
- `src/crafting/machines.rs` — Replaced `Sprite::from_atlas_image(furniture.image)` with `Sprite::from_image(furniture.processing_machine_image)` in `handle_place_machine`
- `src/world/objects.rs` — Added 5 standalone image handles (`shipping_bin_image`, `crafting_bench_image`, `carpenter_board_image`, `processing_machine_image`, `chest_image`) to `FurnitureAtlases`; load them in `ensure_furniture_atlases_loaded`; replaced atlas-based sprite creation with `Sprite::from_image()` for shipping bin, crafting bench, and carpenter board spawn functions
- `src/save/mod.rs` — Replaced atlas-based machine sprite with `Sprite::from_image(processing_machine_image)` in save-load machine restoration; removed unused `machine_atlas_index` import

## What Was Implemented
- Sprinkler entities now render using `sprites/sprinkler.png` (16x16) instead of furniture atlas index 36
- Scarecrow entities now render using `sprites/scarecrow.png` (48x48) instead of furniture atlas index 45
- Shipping bin renders using `sprites/shipping_bin.png` (32x32) instead of furniture atlas index 18
- Crafting bench renders using `sprites/crafting_bench.png` (48x48) instead of furniture atlas index 27
- Carpenter board renders using `sprites/carpenter_board.png` (48x48) instead of furniture atlas index 20
- Processing machines (furnace, keg, etc.) render using `sprites/processing_machine.png` (64x48) instead of furniture atlas with per-machine indices
- Save/load machine restoration uses the same standalone sprite
- Color-rectangle fallbacks retained as last-resort for the brief window before assets finish loading

## Assets Wired
| Asset | Size | Used For |
|-------|------|----------|
| `sprites/sprinkler.png` | 16x16 | Sprinkler farm objects |
| `sprites/scarecrow.png` | 48x48 | Scarecrow farm objects |
| `sprites/shipping_bin.png` | 32x32 | Shipping bin on farm |
| `sprites/crafting_bench.png` | 48x48 | Crafting bench on farm |
| `sprites/carpenter_board.png` | 48x48 | Carpenter board in town |
| `sprites/processing_machine.png` | 64x48 | All processing machines (furnace, keg, etc.) |

## Shared Type Imports Used
- `FarmObject`, `FarmState`, `TILE_SIZE`, `Z_ENTITY_BASE`, `grid_to_world_center`, `LogicalPosition`, `YSorted`, `MapId`, `PlayerState`, `Interactable`, `InteractionKind`, `GridPosition`

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (128 passed, 0 failed, 2 ignored)
- `cargo clippy -- -D warnings` — PASS (0 warnings)

## Known Risks for Integration
- Interior storage chest still uses furniture atlas index 21 (not changed — `chest.png` is a multi-frame atlas, not a single sprite)
- Fence rendering unchanged (already uses autotile atlas properly)
- All processing machines now share a single sprite image; per-machine-type sprites would need individual asset files
