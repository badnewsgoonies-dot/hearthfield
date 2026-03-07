# Worker: FIX-PLAYER

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/player/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/player/item_use.rs (read the FULL file — this is where the fix goes)
2. src/player/mod.rs (read the FULL file — understand plugin structure)
3. src/animals/feeding.rs (read the FULL file — understand what event the feeding system listens for)
4. src/shared/mod.rs (the type contract — search for ItemRemovedEvent, FeedTrough)

## Bug: Animal Feeding Has No Code Path

### Root Cause
`src/animals/feeding.rs` (`handle_feed_trough_interact`) listens for `ItemRemovedEvent` where `item_id == "hay"`. But NOTHING in the codebase sends this event. `src/player/item_use.rs` (`dispatch_item_use`) handles: edible food, sprinklers, machines, bouquet, and mermaid_pendant — but NOT hay. So the player cannot feed animals.

### Fix Required
In `src/player/item_use.rs`, in the `dispatch_item_use` function, add a hay handling branch. Place it BEFORE the food (edible) check, since hay is not edible but needs special handling:

```rust
// ── HAY (animal feed trough) ────────────────────────────────────
if item_id == "hay" {
    let removed = inventory.try_remove("hay", 1);
    if removed > 0 {
        removed_events.send(ItemRemovedEvent {
            item_id: "hay".into(),
            quantity: 1,
        });
    }
    return;
}
```

You will need to:
1. Add `mut inventory: ResMut<Inventory>` — change the existing `inventory: Res<Inventory>` to `mut inventory: ResMut<Inventory>`
2. Add `mut removed_events: EventWriter<ItemRemovedEvent>` parameter
3. Add the hay branch before the `if def.edible {` block

Note: `ItemRemovedEvent` is defined in `src/shared/mod.rs` and already registered in `main.rs`. No new event registration needed.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-player.md containing:
- Files modified (with line counts)
- What was changed and why
- Validation results (pass/fail)
