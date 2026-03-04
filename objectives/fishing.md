# Worker: FISHING

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/fishing/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/fishing.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `FishDef`, `FishRegistry`, `FishLocation`, `Rarity`
- `Inventory`, `ItemId`
- `PlayerState`, `PlayerInput`
- `Calendar`, `Season`, `Weather`, `MapId`
- `GameState`, `InputContext`, `InputBlocks`
- Events: `ToolUseEvent`, `ItemPickupEvent`, `PlaySfxEvent`, `ToastEvent`
- Constants: `TILE_SIZE`, `Z_EFFECTS`

## Deliverables
- `src/fishing/mod.rs` — `FishingPlugin`
- `src/fishing/cast.rs` — Cast initiation near water
- `src/fishing/bite.rs` — Random bite timer
- `src/fishing/fish_select.rs` — Fish selection from valid pool
- `src/fishing/minigame.rs` — Fishing bar minigame
- `src/fishing/resolve.rs` — Catch/fail resolution
- `src/fishing/skill.rs` — Fishing XP and level tracking
- `src/fishing/legendaries.rs` — Legendary fish tracking
- `src/fishing/treasure.rs` — Treasure chest catches
- `src/fishing/render.rs` — Fishing visual rendering

## Quantitative targets (non-negotiable)
- 20 fish species across 4 locations
- Rarity: 50% Common, 25% Uncommon, 15% Rare, 10% Legendary
- 3 legendary fish
- Minigame: 10-second timer, 80% overlap threshold
- Bar size: 40px base + 3px per skill level
- XP per catch: Common 3, Uncommon 8, Rare 15, Legendary 25
- Skill levels: 1-10 with thresholds 10/25/50/100/200/350/550/800/1100/1500
- Treasure: 10% chance per catch
- Bite wait: 3.0 + random(0.0, 7.0) seconds - 0.5 per level

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/fishing.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
