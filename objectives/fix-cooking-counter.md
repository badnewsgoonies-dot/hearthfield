# Worker: FIX-COOKING-COUNTER (recipes_cooked for Chef Achievement)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/crafting/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/crafting/cooking.rs (read the FULL file — add counter here)
2. src/shared/mod.rs — search for "Achievements" struct (has `progress: HashMap<String, u32>`)

## Task: Add "recipes_cooked" Counter to Cooking

### Context
The "Chef" achievement (in economy/achievements.rs) will check:
```rust
achievements.progress.get("recipes_cooked").copied().unwrap_or(0) >= 20
```

The cooking code already increments `achievements.progress["crafts"]` (for the "Artisan" achievement). We also need a SEPARATE `"recipes_cooked"` counter for cooking specifically.

### Fix Required
In `src/crafting/cooking.rs`, in `handle_cook_item`, RIGHT AFTER the existing line:
```rust
*achievements.progress.entry("crafts".to_string()).or_insert(0) += 1;
```

Add:
```rust
*achievements.progress.entry("recipes_cooked".to_string()).or_insert(0) += 1;
```

That's it — one line addition. The `achievements` parameter already exists in the function.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-cooking-counter.md
