# Worker: FIX-CRAFTING-COUNTER (Artisan Achievement)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/crafting/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/crafting/bench.rs (read the FULL file — handle_craft_item is here)
2. src/crafting/cooking.rs (read the FULL file — handle_cook_item is here)
3. src/crafting/mod.rs (read the FULL file — plugin registration)
4. src/shared/mod.rs — search for "Achievements" to find the struct (has `progress: HashMap<String, u32>`)

## Bug: Artisan Achievement Impossible — Crafts Counter Never Incremented

### Root Cause
The "Artisan" achievement in economy/achievements.rs checks:
```rust
achievements.progress.get("crafts").copied().unwrap_or(0) >= 20
```

But NEITHER `handle_craft_item` (bench.rs) NOR `handle_cook_item` (cooking.rs) ever increments this counter. The `"crafts"` key in `achievements.progress` is never written to, so it always returns 0.

### Fix Required

#### In src/crafting/bench.rs — `handle_craft_item`:
1. Add `mut achievements: ResMut<Achievements>` parameter
2. After the successful craft (after the ItemPickupEvent send, around line 183-186), add:
```rust
let count = achievements.progress.entry("crafts".to_string()).or_insert(0);
*count += 1;
```

#### In src/crafting/cooking.rs — `handle_cook_item`:
1. Add `mut achievements: ResMut<Achievements>` parameter
2. After the successful cook (after the ItemPickupEvent send, around line 137-140), add:
```rust
let count = achievements.progress.entry("crafts".to_string()).or_insert(0);
*count += 1;
```

Note: `Achievements` is in `crate::shared::Achievements`. Import it from there.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-crafting-counter.md
