# Worker: FIX-CHEF-ACHIEVEMENT

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/economy/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/economy/achievements.rs (read the FULL file)
2. src/shared/mod.rs — search for "Achievements" to find the struct with `progress: HashMap<String, u32>`

## Bug: "Chef" Achievement Checks Wrong Stat

### Root Cause
The "Chef" achievement is defined as "Cook 20 recipes" but the check at line ~245 in achievements.rs uses:
```rust
"chef" => stats.food_eaten >= 20,
```

This checks how many foods the player has EATEN, not how many they've COOKED. The correct counter is `achievements.progress["crafts"]` which is now incremented by both handle_craft_item and handle_cook_item (fixed in wave 2b).

However, using "crafts" would conflate cooking with general crafting. The Chef achievement should track COOKING specifically.

### Fix Required

#### Option A (Simpler — recommended):
Since cooking increments `achievements.progress["crafts"]` and the "Artisan" achievement already checks `progress["crafts"] >= 20`, we need a SEPARATE counter for cooking.

1. In `src/economy/achievements.rs`, change the "chef" check from:
```rust
"chef" => stats.food_eaten >= 20,
```
to:
```rust
"chef" => {
    achievements.progress.get("recipes_cooked").copied().unwrap_or(0) >= 20
}
```

2. This requires `achievements` to be available in the check function. Look at how the "artisan" check accesses `achievements.progress` — use the same pattern for "chef".

NOTE: The `"recipes_cooked"` counter does NOT yet exist in the crafting code. That's OK — the cooking.rs worker (in a future fix) will need to add:
```rust
*achievements.progress.entry("recipes_cooked".to_string()).or_insert(0) += 1;
```
But that's in src/crafting/ (out of scope for this worker). For now, just fix the CHECK to look at the right key. The increment will be added by a crafting worker.

Actually — re-read src/crafting/cooking.rs to see if it already increments "crafts". If it does, you could just use "crafts" for now. But ideally we want a separate "recipes_cooked" key.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-chef-achievement.md
