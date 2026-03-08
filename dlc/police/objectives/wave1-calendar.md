# Worker: Calendar Domain (Wave 1)

## Scope (mechanically enforced — edits outside this path will be reverted)
You may only create/modify files under: `dlc/police/src/domains/calendar/`
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk BEFORE writing any code)
1. `dlc/police/docs/spec.md` — full game spec
2. `dlc/police/docs/domains/calendar.md` — your domain spec (CRITICAL — read every line)
3. `dlc/police/src/shared/mod.rs` — the type contract. Import from here. Do NOT redefine types.
4. `dlc/police/src/main.rs` — see how plugins are registered

## Interpretation contract
Before coding, extract from calendar.md:
- The preferred approach for each decision (Update with delta, not FixedUpdate)
- The tempting alternatives and what they break
- The drift cues that signal wrong interpretation

## Required imports (use exactly, do not redefine locally)
From `crate::shared`:
- `ShiftClock`, `ShiftType`, `DayOfWeek`, `Weather`, `Rank`
- `ShiftEndEvent`
- `GameState`, `UpdatePhase`
- `TIME_SCALE`, `SHIFT_DURATION_HOURS`

## Deliverables
Create `dlc/police/src/domains/calendar/mod.rs` containing:
- `pub struct CalendarPlugin;` implementing `Plugin`
- `tick_clock` system: advances ShiftClock time using delta_secs * TIME_SCALE
- `check_shift_end` system: emits ShiftEndEvent when shift hours elapse
- `handle_new_day_weather` system: randomizes weather on day transition
- `check_rank_progression` system: updates rank based on shift_number thresholds
- All systems in UpdatePhase::Simulation, gated on GameState::Playing

## Quantitative targets (non-negotiable)
- TIME_SCALE = 2.0 game-minutes per real-second
- SHIFT_DURATION_HOURS = 8
- Rank tiers: shifts 1-28 Patrol, 29-56 Detective, 57-84 Sergeant, 85+ Lieutenant
- Weather weights: Clear 60%, Rainy 20%, Foggy 10%, Snowy 10%
- Clock pauses when ShiftClock.time_paused == true

## Failure patterns to avoid
- Redefining ShiftClock, Weather, Rank, or any shared type locally
- Using FixedUpdate instead of Update with delta
- Hardcoded frame counts instead of real-time delta
- Forgetting to gate systems on GameState::Playing
- Creating ShiftEndEvent handler here (other domains handle that)

## Validation (run from dlc/police/ directory)
```bash
cargo check -p precinct
cargo test -p precinct
```
Done = both commands pass with zero errors.

## When done
Write a completion report to `dlc/police/status/workers/calendar.md` containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit
- Shared type imports used
- What is now player-reachable because of this work
- Any assumptions made
