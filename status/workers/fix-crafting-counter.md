# Worker Report: FIX-CRAFTING-COUNTER

## Files Modified
- `src/crafting/bench.rs` — added `mut achievements: ResMut<Achievements>` param and counter increment
- `src/crafting/cooking.rs` — added `mut achievements: ResMut<Achievements>` param and counter increment

## What Was Implemented
- `handle_craft_item`: after emitting `ItemPickupEvent`, increments `achievements.progress["crafts"]`
- `handle_cook_item`: after emitting `ItemPickupEvent`, increments `achievements.progress["crafts"]`
- Both files already use `use crate::shared::*;` so no new imports were required

## Quantitative Targets
- 2 functions patched (handle_craft_item, handle_cook_item) ✓
- "crafts" counter now increments on every successful craft/cook ✓

## Shared Type Imports Used
- `Achievements` (via `crate::shared::*`)

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (88 passed, 0 failed)
- `cargo clippy -- -D warnings` — PASS

## Known Risks
None — changes are minimal, additive, and scoped to the success path only.
