# Worker: FIX-CITY-TASK-PROGRESS (Priority Progress Formula)

## Scope
You may only modify files under: dlc/city/src/

## Required reading
1. dlc/city/src/game/systems/tasks.rs (FULL file — the progress formula is here)
2. dlc/city/src/game/systems/tests.rs (understand existing tests)

## Bug B001: Task Progress Formula Is Backwards

### Root Cause
In `tasks.rs` lines 13-25, `priority_progress_multiplier` gives:
- Low: 1.0 (fastest)
- Critical: 0.62 (slowest)

This is **backwards**. Critical tasks should be HARDER (require more work), meaning lower progress per action. Wait — actually this IS correct if you think about it: critical tasks need more effort, so progress per action is lower. BUT the clamping at [0.12, 1.0] masks all difficulty because:
- With focus 76, required_focus 30: `0.52 * (76/30) * 1.0 = 1.32` → clamped to 1.0 (LOW task completed in 1 action!)
- With focus 76, required_focus 66: `0.52 * (76/66) * 0.62 = 0.37` → 3 actions max

The real problem is that LOW priority tasks complete in 1 action (too easy), and the clamp range [0.12, 1.0] is too generous.

### Fix Required

1. **Reduce base progress factor** from 0.52 to 0.28 — this makes tasks take 2-5 actions instead of 1-3
2. **Narrow clamp range** from [0.12, 1.0] to [0.08, 0.55] — cap single-action progress at 55%
3. **Keep priority multipliers as-is** — they're actually correct (critical = harder = less progress)

In `progress_delta_for_task`:
```rust
fn progress_delta_for_task(required_focus: i32, priority: TaskPriority, current_focus: i32) -> f32 {
    let focus_ratio = (current_focus.max(0) as f32 / required_focus.max(1) as f32).min(1.0);
    (0.28 * focus_ratio * priority_progress_multiplier(priority)).clamp(0.08, 0.55)
}
```

This means:
- Low priority, high focus: `0.28 * 1.0 * 1.0 = 0.28` → 4 actions to complete
- Critical priority, decent focus: `0.28 * (76/66) * 0.62 = 0.20` → 5 actions
- Low focus, any task: `0.28 * 0.5 * 0.88 = 0.12` → 8-9 actions

4. **Update any tests** that assert specific progress values — they will need new expected values.

### Validation
```
cd dlc/city && cargo check && cargo test && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-city-task-progress.md
