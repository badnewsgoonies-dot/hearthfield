# Worker: FIX-FISHING (Night Fish Time Wrapping)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/fishing/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/fishing/fish_select.rs (read the FULL file — the bug is here)
2. src/data/fish.rs (search for Eel, Squid, Anglerfish to see their time_range values)

## Bug: Night Fish Time Range Wrapping

### Root Cause
`is_fish_available` in `src/fishing/fish_select.rs` at lines 77-81 checks time ranges with a simple linear comparison:
```rust
let (t_min, t_max) = f.time_range;
if time < t_min || time > t_max {
    return false;
}
```

This does NOT handle midnight-crossing ranges where `t_min > t_max`. Night fish have wraparound time ranges:
- Eel: (16.0, 2.0) — 4PM to 2AM
- Squid: (18.0, 2.0) — 6PM to 2AM
- Anglerfish: (18.0, 2.0) — 6PM to 2AM

At 22:00 (10PM), the check becomes `22.0 < 18.0 || 22.0 > 2.0` → `false || true` → filtered out (WRONG).
These three fish are completely uncatchable.

### Fix Required
In `src/fishing/fish_select.rs`, replace lines 77-81 with:
```rust
// Must be within time range (handles midnight wraparound)
let (t_min, t_max) = f.time_range;
let time_ok = if t_min <= t_max {
    // Normal range: e.g., 6.0 to 20.0
    time >= t_min && time <= t_max
} else {
    // Wraparound range: e.g., 18.0 to 2.0
    time >= t_min || time <= t_max
};
if !time_ok {
    return false;
}
```

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-fishing-nighttime.md
