# Calendar Domain Spec — Precinct

## Purpose
The shift clock and time system. Drives the game's temporal rhythm: shift on/off duty, time-of-day, weather, day-of-week progression.

## Scope
`src/domains/calendar/` — owns all time progression logic.

## What This Domain Does
- Tick the `ShiftClock` resource forward based on `time.delta_secs() * TIME_SCALE`
- Advance hour/minute, handle day rollover (hour >= 24 → new day, advance day_of_week)
- Detect shift boundaries: when on_duty and shift_duration elapsed → emit `ShiftEndEvent`
- Pause time when `ShiftClock.time_paused` is true (menus, dialogue, etc.)
- Randomize weather each new day (weighted: 60% Clear, 20% Rainy, 10% Foggy, 10% Snowy)
- Track shift_number (total shifts worked) and rank tier

## What This Domain Does NOT Do
- UI rendering (that's ui domain)
- Player state changes (that's player domain)
- Anything with cases, evidence, NPCs, economy, skills

## Key Types (import from crate::shared)
- `ShiftClock` (Resource) — the main clock resource, already initialized in main.rs
- `ShiftEndEvent` (Event) — emit when shift ends
- `ShiftType`, `DayOfWeek`, `Weather`, `Rank`
- `UpdatePhase` — systems go in `UpdatePhase::Simulation`
- Constants: `TIME_SCALE`, `SHIFT_DURATION_HOURS`

## Systems to Implement
1. `tick_clock` — runs in `Update`, `UpdatePhase::Simulation`, gated on `GameState::Playing`
   - Skip if `time_paused`
   - Increment: `elapsed_real_seconds += delta; minute += (delta * TIME_SCALE) as u8`
   - Handle minute overflow → hour increment
   - Handle hour >= 24 → new day (day += 1, advance day_of_week)
2. `check_shift_end` — runs after `tick_clock`
   - If on_duty and shift hours elapsed >= SHIFT_DURATION_HOURS → emit ShiftEndEvent, set on_duty = false
3. `handle_new_day_weather` — on day transition, randomize weather
4. `check_rank_progression` — track whether shift_number crosses rank tier boundaries (28 shifts per tier)

## Quantitative Targets
- TIME_SCALE = 2.0 game-minutes per real-second
- SHIFT_DURATION_HOURS = 8 (so 1 shift = 4 real minutes)
- Rank tiers: shifts 1-28 Patrol, 29-56 Detective, 57-84 Sergeant, 85-112 Lieutenant
- Weather weights: Clear 60%, Rainy 20%, Foggy 10%, Snowy 10%

## Decision Fields
- **Preferred**: tick_clock in Update with delta_secs multiplication
- **Tempting alternative**: FixedUpdate with fixed timestep
- **Consequence**: FixedUpdate causes time drift when framerate varies; Update + delta is correct for game clock
- **Drift cue**: worker uses `FixedUpdate` or hardcoded frame count instead of real-time delta

## Plugin Export
```rust
pub struct CalendarPlugin;
impl Plugin for CalendarPlugin {
    fn build(&self, app: &mut App) {
        // Register systems in UpdatePhase::Simulation, gated on GameState::Playing
    }
}
```

## Tests (minimum 5)
1. Clock ticks forward when not paused
2. Clock does NOT tick when time_paused = true
3. Day advances when hour reaches 24
4. ShiftEndEvent emits after 8 hours on duty
5. Weather randomizes on new day
