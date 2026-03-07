# Worker: MINING

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/mining/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/mining.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `MineState`, `MineRock`, `MineMonster`, `MineEnemy`
- `PlayerState`, `PlayerInput`
- `Inventory`, `ItemId`
- `MapId`, `GameState`
- `GridPosition`, `LogicalPosition`, `YSorted`
- Events: `MapTransitionEvent`, `StaminaDrainEvent`, `ItemPickupEvent`, `ToolUseEvent`, `MonsterSlainEvent`, `PlaySfxEvent`, `ToastEvent`
- Constants: `TILE_SIZE`, `Z_ENTITY_BASE`, `MAX_HEALTH`

## Deliverables
- `src/mining/mod.rs` — `MiningPlugin`
- `src/mining/floor_gen.rs` — Procedural floor generation
- `src/mining/rock_breaking.rs` — Rock damage and ore drops
- `src/mining/combat.rs` — Player vs monster combat
- `src/mining/ladder.rs` — Ladder discovery and interaction
- `src/mining/transitions.rs` — Enter/exit mine, floor transitions
- `src/mining/movement.rs` — Mine-specific movement
- `src/mining/spawning.rs` — Rock and monster spawning
- `src/mining/components.rs` — Mining-specific components
- `src/mining/hud.rs` — Mine floor/health display

## Quantitative targets (non-negotiable)
- 20 mine floors
- Elevator: every 5 floors (5, 10, 15, 20)
- Rocks per floor: 8-15
- Monsters: GreenSlime (HP 20, DMG 5, SPD 30), Bat (HP 15, DMG 8, SPD 50), RockCrab (HP 40, DMG 12, SPD 15)
- Ore distribution: Copper floors 1-10, Iron 6-15, Gold 11-20, Iridium 16-20
- Gems: quartz 40%, amethyst 25%, emerald 15%, ruby 12%, diamond 8%
- Ladder: 5% base + 2% per rock broken, max 30%
- Pickaxe damage: Basic 1, Copper 2, Iron 3, Gold 4, Iridium 5

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/mining.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
