# Worker: CRAFTING

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/crafting/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/crafting.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `Recipe`, `RecipeRegistry`, `UnlockedRecipes`
- `Inventory`, `InventorySlot`, `ItemId`, `ItemCategory`
- `ItemRegistry`, `ItemDef`
- `FoodBuff`, `BuffType`, `ActiveBuffs`
- `HouseState`, `HouseTier`
- `PlayerState`, `PlayerInput`
- `GameState`, `InputBlocks`
- `Interactable`, `InteractionKind`
- Events: `EatFoodEvent`, `ItemPickupEvent`, `ItemRemovedEvent`, `ToastEvent`
- Constants: `TILE_SIZE`

## Deliverables
- `src/crafting/mod.rs` — `CraftingPlugin`
- `src/crafting/bench.rs` — Crafting bench interaction
- `src/crafting/recipes.rs` — Recipe validation and execution
- `src/crafting/cooking.rs` — Kitchen cooking system
- `src/crafting/machines.rs` — Processing machine timers
- `src/crafting/buffs.rs` — Food buff application and decay
- `src/crafting/unlock.rs` — Recipe unlock triggers

## Quantitative targets (non-negotiable)
- 20 crafting recipes
- 15 cooking recipes
- Machine processing times: Furnace 30min, Preserves 240min, Cheese 180min, Loom 240min
- Buff durations: 2-10 game minutes
- Cooking requires HouseState.has_kitchen == true (Big+ house)

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/crafting.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
