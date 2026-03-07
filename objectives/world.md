# Worker: WORLD

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/world/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/world.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `MapId`, `TileKind`, `MapTransition`, `GridPosition`
- `StorageChest`, `QualityStack`
- `DayNightTint`
- `YSorted`, `LogicalPosition`
- `Calendar`, `Season`, `Weather`
- `PlayerState`
- `Interactable`, `InteractionKind`
- Events: `MapTransitionEvent`, `DayEndEvent`, `SeasonChangeEvent`, `ItemPickupEvent`
- Constants: `TILE_SIZE`, `Z_GROUND`, `Z_FARM_OVERLAY`, `Z_ENTITY_BASE`, `Z_Y_SORT_SCALE`, `Z_EFFECTS`, `Z_SEASONAL`, `Z_WEATHER`
- Functions: `grid_to_world_center()`, `world_to_grid()`

## Deliverables
- `src/world/mod.rs` — `WorldPlugin`
- `src/world/maps.rs` — Map generation for all 10 MapIds
- `src/world/objects.rs` — Breakable objects (trees, rocks)
- `src/world/chests.rs` — Storage chest interaction
- `src/world/lighting.rs` — Day/night tint cycle
- `src/world/weather_fx.rs` — Rain/snow particle effects
- `src/world/seasonal.rs` — Seasonal visual changes
- `src/world/ysort.rs` — Y-sort depth ordering system

## Quantitative targets (non-negotiable)
- 10 maps generated with correct tile dimensions (Farm 32×24, Town 28×22, etc.)
- Map transitions wired between all connected maps
- Day/night tint: dawn → daylight → dusk → night cycle
- Y-sort: Z = Z_ENTITY_BASE - world_y * Z_Y_SORT_SCALE
- Forage: 4-8 items per outdoor map per season
- Chest capacity: 36 slots

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/world.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
