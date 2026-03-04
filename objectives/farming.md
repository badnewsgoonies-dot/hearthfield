# Worker: FARMING

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/farming/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/farming.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `SoilState`, `SoilTile`, `CropTile`, `CropDef`, `CropRegistry`, `FarmState`, `FarmObject`
- `Inventory`, `ItemCategory`, `ItemId`
- `SprinklerKind`, `SprinklerState`, `PlacedSprinkler`, `PlaceSprinklerEvent`
- `ItemQuality`
- Events: `ToolUseEvent`, `DayEndEvent`, `CropHarvestedEvent`, `ItemPickupEvent`, `SeasonChangeEvent`
- Constants: `TILE_SIZE`, `Z_FARM_OVERLAY`, `Z_GROUND`
- Functions: `grid_to_world_center()`, `world_to_grid()`, `watering_can_area()`

## Deliverables
- `src/farming/mod.rs` — `FarmingPlugin`
- `src/farming/soil.rs` — Tilling and watering systems
- `src/farming/crops.rs` — Planting and growth stage advancement
- `src/farming/harvest.rs` — Harvest interaction
- `src/farming/events_handler.rs` — Tool use event handling for farming
- `src/farming/sprinkler.rs` — Sprinkler placement
- `src/farming/sprinklers.rs` — Auto-watering at day start
- `src/farming/render.rs` — Farm tile rendering

## Quantitative targets (non-negotiable)
- Support all 15 crops from spec
- Wither: 2 days without water → wilted, 3 → dead
- Sprinklers: Basic (4 adj), Quality (8 adj+diag), Iridium (24 tiles)
- Season kill: out-of-season crops die on SeasonChangeEvent
- Farm map: 32×24 tile bounds

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/farming.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
