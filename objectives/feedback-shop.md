# Worker: Add Shop Transaction Feedback (Toasts + SFX)

## Context
The shop UI silently does nothing when the player can't afford an item or when their inventory is full. No toast, no SFX. Also, successful buy/sell has no audio confirmation. This makes the shop feel broken.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/ui/`
All out-of-scope edits will be reverted.

## Required reading
1. src/ui/shop_screen.rs — the `update_shop_input` function, specifically the buy/sell branches (~line 570-630)
2. src/shared/mod.rs — READ ONLY. Find `ToastEvent`, `PlaySfxEvent`

## Task 1: Add toast + SFX parameters

In the `update_shop_input` system, add these parameters:
- `mut toast_events: EventWriter<ToastEvent>`
- `mut sfx_events: EventWriter<PlaySfxEvent>`

## Task 2: Add "can't afford" feedback

In the buy branch, after the `if player.gold >= listing.price {` check, add an else:
```rust
if player.gold >= listing.price {
    // ... existing buy logic ...
} else {
    toast_events.send(ToastEvent {
        message: "Not enough gold!".into(),
        duration_secs: 2.0,
    });
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".into(),
    });
}
```

## Task 3: Add "inventory full" feedback

Inside the buy branch, after `let overflow = inventory.try_add(...)`, add feedback for overflow:
```rust
let overflow = inventory.try_add(&listing.item_id, 1, max_stack);
if overflow == 0 {
    // ... existing success logic (gold deduction, transaction event) ...
    sfx_events.send(PlaySfxEvent {
        sfx_id: "sfx_coin_single1".into(),
    });
} else {
    inventory.try_remove(&listing.item_id, 1); // undo the partial add
    toast_events.send(ToastEvent {
        message: "Inventory is full!".into(),
        duration_secs: 2.0,
    });
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".into(),
    });
}
```

Wait — check if try_add actually modifies inventory when overflow > 0. Read the try_add implementation first. If it adds what fits and returns leftover, you need to remove the partial. If it's all-or-nothing, you just need the toast.

## Task 4: Add buy/sell success SFX

After successful purchase (gold deducted), add:
```rust
sfx_events.send(PlaySfxEvent {
    sfx_id: "sfx_coin_single1".into(),
});
```

After successful sell (gold added), add:
```rust
sfx_events.send(PlaySfxEvent {
    sfx_id: "sfx_coin_single1".into(),
});
```

## Do NOT
- Modify src/shared/mod.rs
- Modify any files outside src/ui/
- Change prices, shop logic, or game balance

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write completion report to status/workers/feedback-shop.md
