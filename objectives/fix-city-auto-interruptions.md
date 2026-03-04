# Worker: FIX-CITY-AUTO-INTERRUPTIONS (Add Automatic Interruption Trigger)

## Scope
You may only modify files under: dlc/city/src/

## Required reading
1. dlc/city/src/game/systems/interruptions.rs (FULL file — interruption handling)
2. dlc/city/src/game/systems/mod.rs (system registration and ordering)
3. dlc/city/src/game/resources.rs (search for interruption_chance_per_hour, OfficeRules, OfficeClock)
4. dlc/city/src/game/events.rs (InterruptionEvent)

## Bug B003: Interruptions Are Manual-Only

### Root Cause
Interruptions only fire when the player presses I. The config `interruption_chance_per_hour: 0.2` exists in OfficeRules but is never used in any system. This means the player can ignore interruptions entirely and just grind tasks.

### Fix Required

1. Add a new system `auto_trigger_interruptions` in `interruptions.rs`:

```rust
pub fn auto_trigger_interruptions(
    clock: Res<OfficeClock>,
    rules: Res<OfficeRules>,
    config: Res<OfficeRunConfig>,
    mut interruption_writer: EventWriter<InterruptionEvent>,
) {
    // Only trigger during work hours (InDay state is already gated by system registration)
    // Check every 60 game-minutes if an interruption should fire
    let current_minute = clock.minute_of_day;
    let hour_mark = current_minute / 60;

    // Use deterministic seeding: day + hour to decide if interruption fires
    let seed = config.seed.wrapping_add(clock.day_index as u64 * 100 + hour_mark as u64);
    let chance = ((seed % 100) as f32) / 100.0;

    // Only fire once per hour mark — check if we just crossed an hour boundary
    let prev_minute = current_minute.saturating_sub(1);
    let prev_hour = prev_minute / 60;
    if hour_mark != prev_hour && chance < rules.interruption_chance_per_hour {
        interruption_writer.send(InterruptionEvent);
    }
}
```

2. Register this system in `mod.rs` in the Interruptions system set, running in the `InDay` state.

3. **Also fix B002** — increase `calm_stress_relief` from 9 to 14 in `resources.rs` (OfficeRules default) so interruptions are net-neutral when handled calmly:
   - Interruption: +14 stress avg
   - Calm resolve: -14 stress
   - Net: 0 stress (fair trade for +12 min lost)

4. Update any tests that depend on the old calm_stress_relief value.

### Validation
```
cd dlc/city && cargo check && cargo test && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-city-auto-interruptions.md
