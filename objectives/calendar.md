# Worker: CALENDAR

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/calendar/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/calendar.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `GameState`
- `Calendar`, `Season`, `DayOfWeek`, `Weather`
- `DayEndEvent`, `SeasonChangeEvent`
- `PlayerInput`
- `MapId`
- `DAYS_PER_SEASON`, `SEASONS_PER_YEAR`
- `CutsceneQueue`, `CutsceneStep`

## Deliverables
- `src/calendar/mod.rs` — `CalendarPlugin` with all time management systems
- `src/calendar/festivals.rs` — Festival day detection, cutscene triggers
- Correct time progression: 6 AM → 2 AM, day/season/year advancement
- Weather generation per season with correct distributions
- `DayEndEvent` and `SeasonChangeEvent` fired at correct times
- Festival detection for Spring 13, Summer 11, Fall 16, Winter 25

## Quantitative targets (non-negotiable)
- 4 seasons × 28 days = 112-day year
- Time: hour 6..25, default time_scale = 10.0
- Weather: Spring (60% Sunny, 30% Rainy, 10% Stormy), Summer (70/20/10), Fall (50/35/15), Winter (40/30/20/10 with Snowy)
- 4 festival days at exact dates listed above
- Time pauses when `Calendar.time_paused == true`

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/calendar.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration
