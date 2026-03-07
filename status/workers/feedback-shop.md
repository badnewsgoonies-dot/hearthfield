# Worker Report: Shop Transaction Feedback (Toasts + SFX)

## Files Modified
- `src/ui/shop_screen.rs` — added toast/sfx feedback to `shop_navigation` function

## What Was Implemented

### Parameters added to `shop_navigation`
- `mut toast_events: EventWriter<ToastEvent>`
- `mut sfx_events: EventWriter<PlaySfxEvent>`

### Buy branch — can't afford
When `player.gold < listing.price`, now sends:
- `ToastEvent { message: "Not enough gold!", duration_secs: 2.0 }`
- `PlaySfxEvent { sfx_id: "error" }`

### Buy branch — inventory full
When `inventory.try_add(...)` returns `overflow > 0` (partial add), now:
1. Calls `inventory.try_remove` to undo the partial add
2. Sends `ToastEvent { message: "Inventory is full!", duration_secs: 2.0 }`
3. Sends `PlaySfxEvent { sfx_id: "error" }`

### Buy success SFX
After successful purchase (gold deducted), sends:
- `PlaySfxEvent { sfx_id: "sfx_coin_single1" }`

### Sell success SFX
After successful sell (gold added), sends:
- `PlaySfxEvent { sfx_id: "sfx_coin_single1" }`

## Quantitative Targets
- 4 feedback paths added (can't afford, inventory full, buy success, sell success) ✅

## Shared Type Imports Used
- `ToastEvent` (src/shared/mod.rs:1088)
- `PlaySfxEvent` (src/shared/mod.rs:940)
Both already available via `use crate::shared::*;`

## Validation Results
```
~/.cargo/bin/cargo check
Finished `dev` profile in 42.57s — 0 errors, 0 warnings
```
✅ PASS

## Notes
- `try_add` adds what fits and returns leftover count. When overflow > 0, a partial add may have occurred. The fix removes the partial via `try_remove` before showing the "full" toast.
- No changes to game balance, prices, or shop logic.
