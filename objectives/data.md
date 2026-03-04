# Worker: DATA

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/data/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/data.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `ItemRegistry`, `ItemDef`, `ItemId`, `ItemCategory`
- `CropRegistry`, `CropDef`
- `FishRegistry`, `FishDef`, `FishLocation`, `Rarity`
- `NpcRegistry`, `NpcDef`, `NpcSchedule`, `ScheduleEntry`
- `RecipeRegistry`, `Recipe`
- `ShopData`, `ShopListing`, `ShopId`
- `Season`, `Weather`, `MapId`
- `GiftPreference`

## Deliverables
- `src/data/mod.rs` — `DataPlugin` with all data loading systems
- `src/data/items.rs` — ItemDef entries for 80+ items
- `src/data/crops.rs` — CropDef entries for 15 crops
- `src/data/fish.rs` — FishDef entries for 20 fish
- `src/data/npcs.rs` — NpcDef and NpcSchedule entries for 10 NPCs
- `src/data/recipes.rs` — Recipe entries for 35 recipes
- `src/data/shops.rs` — ShopListing entries for 3 shops

## Quantitative targets (non-negotiable)
- 80+ item definitions
- 15 crop definitions with full growth_days arrays
- 20 fish definitions with location, season, time, rarity
- 10 NPC definitions with gift preferences and schedules
- 35 recipes (20 crafting + 15 cooking)
- 3 shops with seasonal availability

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/data.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
