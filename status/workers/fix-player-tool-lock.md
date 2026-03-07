# Worker Report: FIX-PLAYER (Tool Lock During Upgrade)

## Files Modified
- `src/player/tools.rs` (+8 lines)

## What Was Implemented
Added `upgrade_queue: Res<crate::economy::blacksmith::ToolUpgradeQueue>` parameter to `tool_use` function. After the cooldown check and before the stamina check, added a guard that calls `upgrade_queue.is_upgrading(tool)` — if the tool is currently being upgraded at the blacksmith, an error SFX is played and the function returns early.

## Shared Type Imports Used
- `ToolKind` (via `crate::shared::*`, already imported)
- `crate::economy::blacksmith::ToolUpgradeQueue` (referenced directly)

## Validation Results
- `cargo check`: PASS
- `cargo test --test headless`: PASS (88 passed, 0 failed)
- `cargo clippy -- -D warnings`: PASS

## Known Risks
None. The fix is minimal and purely additive — it only adds a read-only resource parameter and an early-return guard. No existing logic was altered.
