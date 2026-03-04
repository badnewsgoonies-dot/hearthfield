# Worker Report: FIX-CITY-AUTO-INTERRUPTIONS

## Files Modified
- `dlc/city/src/game/systems/interruptions.rs` — added `auto_trigger_interruptions` system (~16 lines)
- `dlc/city/src/game/systems/mod.rs` — exported `auto_trigger_interruptions`
- `dlc/city/src/game/mod.rs` — registered `auto_trigger_interruptions` in `OfficeSimSet::Interruptions` chain (before `handle_interruption_requests`)
- `dlc/city/src/game/resources.rs` — changed `calm_stress_relief: 9` → `calm_stress_relief: 14` (B002 fix)
- `dlc/city/src/game/systems/tests.rs` — updated stress assertion in `interruption_and_npc_pressure_events_are_deterministic` from 56 → 51

## What Was Implemented

### B003: Auto-trigger interruptions
Added `auto_trigger_interruptions` system that fires once per hour boundary using deterministic seeding (`config.seed + day_number * 100 + hour_mark`). The system checks if the clock just crossed an hour boundary and if a derived chance value falls below `config.interruption_chance_per_hour` (default 0.2). Uses actual `DayClock` (`current_minute`, `day_number`) and `OfficeRunConfig` types.

### B002: Calm stress relief rebalance
Increased `calm_stress_relief` from 9 to 14 so a calm resolution exactly offsets the average interruption stress gain (+14 from `interruption_stress_increase`), making interruptions net-neutral for attentive players.

## Shared Type Imports Used
- `DayClock` (resources.rs)
- `OfficeRunConfig` (resources.rs)
- `InterruptionEvent` (events.rs)

## Validation Results
- `cargo check` — PASS
- `cargo test` — PASS (40/40 tests)
- `cargo clippy -- -D warnings` — PASS

## Known Risks
- `auto_trigger_interruptions` runs every frame but only fires on hour boundaries — no performance concern.
- The deterministic seed does not account for mid-day config changes; this is consistent with how the rest of the game uses `config.seed`.
