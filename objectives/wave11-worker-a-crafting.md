# Worker A: Crafting Feedback (Toasts + SFX)

## Scope: src/crafting/ ONLY. All other edits will be reverted.

## Read first (DO NOT MODIFY):
- src/shared/mod.rs — find ToastEvent, PlaySfxEvent, Recipe, Inventory

## Task 1: Craft failure feedback (bench.rs)
In handle_craft_item, find where ingredients are checked. If the player tries to craft
but lacks ingredients, add:
```rust
toast_events.send(ToastEvent { message: "Missing ingredients!".into(), duration_secs: 2.0 });
sfx_events.send(PlaySfxEvent { sfx_id: "error".into() });
```
Add EventWriter<ToastEvent> and EventWriter<PlaySfxEvent> to system params if not present.

## Task 2: Craft success feedback (bench.rs)
After successful craft (item added to inventory), add:
```rust
toast_events.send(ToastEvent { message: format!("{} crafted!", recipe.name), duration_secs: 2.0 });
sfx_events.send(PlaySfxEvent { sfx_id: "sfx_coin_single1".into() });
```

## Task 3: Cooking same pattern (cooking.rs)
Same as tasks 1-2 but in the cooking handler.

## Task 4: Machine collection (machines.rs)
When player collects output from a processing machine, add:
```rust
toast_events.send(ToastEvent { message: format!("{} collected!", item_name), duration_secs: 2.0 });
```

## Validation: cargo check must pass.
## When done: write report to status/workers/feedback-crafting.md
