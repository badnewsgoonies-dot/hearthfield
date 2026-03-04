# Worker: FIX-PLAYER (Tool Lock During Upgrade)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/player/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/player/tools.rs (read the FULL file)
2. src/economy/blacksmith.rs (read lines 1-32 — the ToolUpgradeQueue resource)
3. src/shared/mod.rs — search for ToolKind

## Bug: Player Can Use Tools While Being Upgraded at Blacksmith

### Root Cause
`tool_use` in `src/player/tools.rs` never checks `ToolUpgradeQueue.is_upgrading(tool)`. The player pays gold and bars for an upgrade but can keep using the tool at its old tier as if nothing happened. The `ToolUpgradeQueue` resource and `is_upgrading()` method exist but are never read in the player domain.

### Fix Required
In `src/player/tools.rs`, in the `tool_use` function:
1. Add `upgrade_queue: Res<crate::economy::blacksmith::ToolUpgradeQueue>` parameter
2. After the cooldown check (around line 64) and before the stamina check, add:
```rust
// Block usage if tool is being upgraded at the blacksmith
if upgrade_queue.is_upgrading(tool) {
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".to_string(),
    });
    return;
}
```

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-player-tool-lock.md
