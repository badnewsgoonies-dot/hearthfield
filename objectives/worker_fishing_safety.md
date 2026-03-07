# Worker: Fishing Division Safety Guard

## Scope (hard allowlist — enforced mechanically)
You may ONLY modify: src/fishing/mod.rs
All other file edits will be reverted.

## Problem
In src/fishing/mod.rs around line 437, there is a division that can produce NaN:
```rust
let ratio = self.overlap_time_total / self.minigame_total_time;
```
`minigame_total_time` is initialized to 0.0 and reset to 0.0 in the reset() method. There IS a guard at line 433:
```rust
if self.minigame_total_time < 0.5 {
```
that returns early, preventing the division when time is too small. However, the 0.5 threshold means values between 0.0 and 0.5 trigger the early return — which is correct behavior.

## Task
Add a safety guard to the division itself as defense-in-depth. Change:
```rust
let ratio = self.overlap_time_total / self.minigame_total_time;
```
to:
```rust
let ratio = if self.minigame_total_time > 0.0 {
    self.overlap_time_total / self.minigame_total_time
} else {
    0.0
};
```

This is a belt-and-suspenders fix — the early return at line 433 already prevents this path, but the guard makes the code safe even if the early return logic changes in the future.

## Also check: src/fishing/render.rs line 84
```rust
let y_scale = bar_h_world / MINIGAME_BAR_HEIGHT;
```
MINIGAME_BAR_HEIGHT is a constant — verify it's non-zero. If it is a const > 0, no fix needed.

## Validation
```bash
grep "overlap_time_total / self.minigame_total_time" src/fishing/mod.rs
# Should find 0 matches (replaced with guarded version)
```

## When Done
Write completion report to status/workers/fishing_safety.md
