# Worker: ANIMALS

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/animals/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/animals.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `Animal`, `AnimalKind`, `AnimalAge`, `AnimalState`
- `Inventory`, `InventorySlot`, `ItemId`
- `PlayerInput`, `PlayerState`
- `LogicalPosition`, `YSorted`, `GridPosition`
- Events: `DayEndEvent`, `AnimalProductEvent`, `ItemPickupEvent`
- Constants: `TILE_SIZE`, `Z_ENTITY_BASE`

## Deliverables
- `src/animals/mod.rs` — `AnimalPlugin`
- `src/animals/day_end.rs` — Daily happiness decay/gain, aging
- `src/animals/feeding.rs` — Feed trough interaction
- `src/animals/interaction.rs` — Petting, product collection
- `src/animals/movement.rs` — Random wandering in bounds
- `src/animals/products.rs` — Product readiness and collection
- `src/animals/rendering.rs` — Animal sprite rendering
- `src/animals/spawning.rs` — Spawn animals after purchase

## Quantitative targets (non-negotiable)
- 3 livestock types: Chicken (800g, egg daily 50g), Cow (1500g, milk daily 125g), Sheep (4000g, wool every 3 days 340g)
- Happiness: 0-255, +10 fed, +5 petted, -20 unfed, +5 sunny outdoor
- Baby → Adult: 7 days
- Product quality by happiness: 0-99 Normal, 100-149 Silver, 150-199 Gold, 200-255 Iridium
- Building capacity: Basic 4, Big 8, Deluxe 12

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/animals.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
