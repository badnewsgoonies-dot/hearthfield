# Worker: PLAYER

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/player/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/player.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `Player`, `PlayerMovement`, `PlayerAnimState`, `Facing`
- `PlayerState`, `PlayerInput`, `InputBlocks`, `InteractionClaimed`
- `ToolKind`, `ToolTier`
- `Inventory`, `InventorySlot`
- `Interactable`, `InteractionKind`
- `LogicalPosition`, `YSorted`
- `GameState`, `MapId`
- Events: `ToolUseEvent`, `StaminaDrainEvent`, `ToolImpactEvent`, `MapTransitionEvent`, `EatFoodEvent`
- Constants: `TILE_SIZE`, `PIXEL_SCALE`, `MAX_STAMINA`, `MAX_HEALTH`
- Functions: `grid_to_world_center()`, `world_to_grid()`, `tool_stamina_cost()`, `watering_can_area()`

## Deliverables
- `src/player/mod.rs` — `PlayerPlugin` with all player systems
- `src/player/movement.rs` — 4-directional movement with collision
- `src/player/camera.rs` — Smooth camera follow
- `src/player/tools.rs` — Tool use dispatching
- `src/player/tool_anim.rs` — Tool animation state machine
- `src/player/interaction.rs` — F-key interaction detection
- `src/player/interact_dispatch.rs` — Route interactions to correct handler
- `src/player/item_use.rs` — Secondary action (eat food, place item)
- `src/player/spawn.rs` — Player entity spawning

## Quantitative targets (non-negotiable)
- Player speed: 80.0 px/sec
- 6 tools functional: Hoe, WateringCan, Axe, Pickaxe, FishingRod, Scythe
- Interaction range: 1 tile in facing direction
- Stamina cost: base 4.0 × tier multiplier
- Starting: 500g, 100 stamina, 100 health, grid (8,8) on PlayerHouse

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/player.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
