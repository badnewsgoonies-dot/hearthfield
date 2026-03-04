# Worker Report: fix-economy

## Files Modified
- `src/economy/shop.rs` (1 function changed, net ~5 lines added)

## What Was Changed and Why

### Bug: Double Gold Deduction in Shops
The shop UI (`src/ui/shop_screen.rs`) directly mutates `player.gold` when buying/selling for immediate UI feedback. The old `handle_shop_transaction_gold` function then sent a `GoldChangeEvent`, which was processed by `apply_gold_changes` in `gold.rs`, modifying `player_state.gold` a second time. This caused buying to cost 2x and selling to give 2x.

### Fix Applied
Changed `handle_shop_transaction_gold` in `src/economy/shop.rs`:
- **Removed** `EventWriter<GoldChangeEvent>` parameter
- **Added** `ResMut<super::gold::EconomyStats>` parameter to update stats directly
- For purchases: updates `stats.total_gold_spent` via `saturating_add(ev.total_cost as u64)`
- For sales: updates `stats.total_gold_earned` via `saturating_add(ev.total_cost as u64)`
- Increments `stats.total_transactions += 1` for each transaction
- Kept `info!` logging for debugging

This ensures EconomyStats are correctly tracked without triggering a second gold mutation through the GoldChangeEvent pipeline.

## Validation Results
- `cargo check` — PASS
- `cargo clippy -- -D warnings` — PASS (zero errors, zero warnings)

## Notes
- No changes were made to files outside `src/economy/`
- The `mod.rs` plugin registration required no changes (same function name and compatible signature)
- The `use crate::shared::*` wildcard import covers `ShopTransactionEvent`, so no import changes needed
