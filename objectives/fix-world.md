# Worker: FIX-WORLD

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/world/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/world/mod.rs (read the FULL file — the fix is here)
2. src/main.rs (read lines 80-120 — see that ToastEvent is already registered there)

## Bug: Duplicate ToastEvent Registration

### Root Cause
`src/world/mod.rs:56` registers `app.add_event::<ToastEvent>()`. But `src/main.rs:118` ALSO registers `app.add_event::<ToastEvent>()`. While Bevy 0.15 handles duplicate event registration gracefully (idempotent), it's a code smell and could cause confusion.

### Fix Required
In `src/world/mod.rs`, in the `Plugin::build` method, remove the `.add_event::<ToastEvent>()` call. The event is already registered in main.rs.

Look at line 56 (or nearby). It's likely:
```rust
app.add_event::<ToastEvent>()
    .init_resource::<WorldMap>()
```

Change to:
```rust
app.init_resource::<WorldMap>()
```

Make sure the chaining still works syntactically after removal.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-world.md containing:
- Files modified
- What was changed and why
- Validation results (pass/fail)
