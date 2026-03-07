# Worker Report: Crafting Feedback (Toasts + SFX)

## Status: ALREADY IMPLEMENTED

All four tasks were already present in the codebase:

### Task 1: Craft failure feedback (bench.rs)
- `handle_craft_item` lines 160-167: sends `ToastEvent { message: "Missing ingredients!" }` and `PlaySfxEvent { sfx_id: "craft_fail" }` when `has_all_ingredients` returns false.

### Task 2: Craft success feedback (bench.rs)
- `handle_craft_item` lines 209-215: sends `ToastEvent { message: format!("{} crafted!", recipe.name) }` and `PlaySfxEvent { sfx_id: "craft_success" }` after successful craft.

### Task 3: Cooking same pattern (cooking.rs)
- `handle_cook_item` lines 97-104: missing ingredients toast + "craft_fail" SFX for non-wildcard ingredients.
- `handle_cook_item` lines 113-121: same for missing fish wildcard.
- `handle_cook_item` lines 184-190: success toast "{recipe.name} crafted!" + "cook_success" SFX.

### Task 4: Machine collection (machines.rs)
- `handle_collect_machine_output` lines 516-519: sends `ToastEvent { message: format!("{} collected from your {}!", output_display, machine_name) }`.
- Lines 527-529: sends `PlaySfxEvent { sfx_id: "item_pickup" }`.

## Files modified: None (0 changes needed)

## Validation
- `cargo check`: PASS (no errors, no warnings)

## Shared type imports already in use
- `ToastEvent` (from `crate::shared::*`)
- `PlaySfxEvent` (from `crate::shared::*`)
- `EventWriter<ToastEvent>` and `EventWriter<PlaySfxEvent>` already in system params
