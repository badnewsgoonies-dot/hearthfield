# Worker: FIX-CITY-STRESS-PERSISTENCE (Cross-Day Stress + Inbox Balance)

## Scope
You may only modify files under: dlc/city/src/

## Required reading
1. dlc/city/src/game/systems/day_cycle.rs (FULL file — end-of-day rollover logic)
2. dlc/city/src/game/resources.rs (WorkerStats, PlayerMindState, OfficeRules)

## Bug B008: Stress Doesn't Persist Across Days

### Root Cause
In `day_cycle.rs` at the end-of-day rollover (~lines 373-383):
```rust
mind.stress = rules.starting_stress.clamp(0, rules.max_stress);
```

This resets stress to `starting_stress` (18) every day regardless of the previous day's ending stress. A player who ends day 1 at stress 50 starts day 2 at stress 18. No consequences carry over.

### Fix Required

1. **Carry stress forward with decay** — instead of resetting to starting_stress:
```rust
// Stress partially recovers overnight (half decay toward baseline)
let overnight_recovery = (mind.stress - rules.starting_stress) / 2;
mind.stress = (mind.stress - overnight_recovery).clamp(0, rules.max_stress);
```

This means:
- End day at stress 50, start at: 50 - (50-18)/2 = 50 - 16 = 34
- End day at stress 80, start at: 80 - (80-18)/2 = 80 - 31 = 49
- End day at stress 18 (default), start at: 18 (no change)

2. **Also fix B004** — reduce starting_inbox_items from 18 to 12 for more time pressure:
In `resources.rs`, in the OfficeRules Default impl:
```rust
starting_inbox_items: 12,
```

Also reduce `max_tasks_per_day` from 8 to 6:
```rust
max_tasks_per_day: 6,
```

3. Update any tests that assert specific stress reset values or inbox counts.

### Validation
```
cd dlc/city && cargo check && cargo test && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-city-stress-persistence.md
