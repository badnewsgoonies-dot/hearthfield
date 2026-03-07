# Worker Report: Late Night Time Warnings

## Files Modified
- `src/calendar/mod.rs` — added `time_warnings` system (~32 lines)

## What Was Implemented
Added a `time_warnings` system to `src/calendar/mod.rs` that fires toast notifications when the player is playing late:
- **10 PM (hour >= 22):** "It's getting late. Head home and get some rest!"
- **Midnight (hour >= 24):** "You're exhausted! Get to bed before you collapse!"

Both warnings fire at most once per day, using `Local<bool>` flags that reset when `calendar.day` changes.

The system is registered in `CalendarPlugin` as a `Playing`-state `Update` system ordered `.after(tick_time)`.

## Shared Type Imports Used
- `ToastEvent` (via `crate::shared::*`)
- `Calendar` (via `crate::shared::*`)

## Validation
```
cargo check → Finished `dev` profile — 0 errors, 0 warnings
```

## Known Risks
None. System is purely additive and read-only on the Calendar resource.
