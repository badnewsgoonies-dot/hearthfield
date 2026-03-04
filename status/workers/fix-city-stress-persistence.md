# Worker Report: FIX-CITY-STRESS-PERSISTENCE

## Files Modified
- `dlc/city/src/game/systems/day_cycle.rs` — line 372: replaced stress reset with overnight decay formula
- `dlc/city/src/game/resources.rs` — line 239: `max_tasks_per_day` 8 → 6; line 377: `starting_inbox_items` 18 → 12

## What Was Implemented

### B008 — Stress cross-day persistence
Replaced the hard reset (`mind.stress = rules.starting_stress.clamp(...)`) with a half-decay toward baseline:
```rust
let overnight_recovery = (mind.stress - rules.starting_stress) / 2;
mind.stress = (mind.stress - overnight_recovery).clamp(0, rules.max_stress);
```
Stress now carries forward with 50% recovery toward the baseline (18). A player ending at stress 50 starts the next day at 34; ending at 80 starts at 49.

### B004 — Inbox/task balance
- `starting_inbox_items`: 18 → 12
- `max_tasks_per_day`: 8 → 6

## Tests Updated
No test assertions referenced the old stress reset value or the old inbox count directly (the one inbox test reads the value dynamically from `rules.starting_inbox_items`). All 40 existing tests pass without modification.

## Validation
- `cargo check` ✅
- `cargo test` ✅ (40/40 passed)
- `cargo clippy -- -D warnings` ✅

## Known Risks
None. The overnight recovery formula is purely arithmetic on existing fields; no new types or events introduced.
