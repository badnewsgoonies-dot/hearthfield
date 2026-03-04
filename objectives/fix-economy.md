# Worker: FIX-ECONOMY

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/economy/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/economy/shop.rs (read the FULL file)
2. src/economy/gold.rs (read the FULL file)
3. src/economy/mod.rs (read the FULL file)
4. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Bug: Double Gold Deduction in Shops

### Root Cause
The shop UI in `src/ui/shop_screen.rs` directly mutates `player.gold -= listing.price` when buying (line ~590) and `player.gold += price` when selling (line ~609). Then `handle_shop_transaction_gold` in `src/economy/shop.rs:114` converts `ShopTransactionEvent` into `GoldChangeEvent`. Then `apply_gold_changes` in `src/economy/gold.rs:16` reads that `GoldChangeEvent` and modifies `player_state.gold` AGAIN. Result: buying costs 2x, selling gives 2x.

### Fix Required
In `src/economy/shop.rs`, change `handle_shop_transaction_gold` to:
- **NOT** send `GoldChangeEvent` (remove the `EventWriter<GoldChangeEvent>` parameter)
- Instead, take `ResMut<super::gold::EconomyStats>` directly
- Update `stats.total_gold_spent` (for purchases) or `stats.total_gold_earned` (for sales) using `saturating_add(ev.total_cost as u64)`
- Increment `stats.total_transactions += 1`
- Keep info! logging for debugging

The shop UI's direct gold mutation is correct (it needs immediate feedback for UI validation). We just need stats tracking without the double mutation through GoldChangeEvent.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-economy.md containing:
- Files modified
- What was changed and why
- Validation results (pass/fail)
