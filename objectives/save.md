# Worker: SAVE

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/save/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/save.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
All serializable resources from shared:
- `Calendar`, `PlayerState`, `Inventory`, `FarmState`, `AnimalState`
- `Relationships`, `MineState`, `UnlockedRecipes`, `ShippingBin`
- `HouseState`, `MarriageState`, `QuestLog`, `SprinklerState`, `ActiveBuffs`
- `EvaluationScore`, `RelationshipStages`, `Achievements`, `ShippingLog`
- `TutorialState`, `PlayStats`
- `GameState`
- Events: `DayEndEvent`

## Deliverables
- `src/save/mod.rs` — `SavePlugin` with save/load/autosave systems

## Quantitative targets (non-negotiable)
- 3 save slots
- 20+ resources serialized
- Round-trip fidelity: save → load → save = identical JSON
- Auto-save on DayEndEvent

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/save.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
