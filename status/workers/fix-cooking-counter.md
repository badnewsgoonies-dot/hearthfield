# Worker Report: FIX-COOKING-COUNTER

## Files Modified
- `src/crafting/cooking.rs` (+1 line, line 143)

## What Was Implemented
Added `"recipes_cooked"` progress counter in `handle_cook_item`, immediately after the existing `"crafts"` counter increment. This enables the "Chef" achievement check (`recipes_cooked >= 20`).

## Change
```rust
*achievements.progress.entry("recipes_cooked".to_string()).or_insert(0) += 1;
```

## Validation Results
- `cargo check`: PASS
- `cargo test --test headless`: 87 passed, 1 failed (`test_farming_day_end_system` — pre-existing, unrelated to this change)
- `cargo clippy -- -D warnings`: PASS
