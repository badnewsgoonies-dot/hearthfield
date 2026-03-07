# Worker: Add Late Night Time Warnings

## Context
The player gets no warning when it's getting late. The intro says "don't forget to sleep before midnight!" but there's no in-game reminder. If the player misses bedtime, they get an abrupt forced-sleep at hour 26.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/calendar/`
All out-of-scope edits will be reverted.

## Required reading
1. src/calendar/mod.rs — the `tick_time` system that advances the clock. Find where `calendar.hour` changes.
2. src/shared/mod.rs — READ ONLY. Find `ToastEvent`, `PlaySfxEvent`, `Calendar` struct.

## Task: Add time-of-day warning toasts

Add a new system `time_warnings` in src/calendar/mod.rs:

```rust
/// Warn the player when it gets late. Uses Local flags to fire each warning only once per day.
pub fn time_warnings(
    calendar: Res<Calendar>,
    mut toast_events: EventWriter<ToastEvent>,
    mut warned_10pm: Local<bool>,
    mut warned_midnight: Local<bool>,
    mut last_day: Local<u8>,
) {
    // Reset warnings on new day
    if calendar.day != *last_day {
        *warned_10pm = false;
        *warned_midnight = false;
        *last_day = calendar.day;
    }
    
    if calendar.hour >= 22 && !*warned_10pm {
        *warned_10pm = true;
        toast_events.send(ToastEvent {
            message: "It's getting late. Head home and get some rest!".into(),
            duration_secs: 4.0,
        });
    }
    
    if calendar.hour >= 24 && !*warned_midnight {
        *warned_midnight = true;
        toast_events.send(ToastEvent {
            message: "You're exhausted! Get to bed before you collapse!".into(),
            duration_secs: 4.0,
        });
    }
}
```

Register this system in the CalendarPlugin — add it to the Playing-state Update systems alongside `tick_time`.

## Do NOT
- Modify src/shared/mod.rs
- Change the forced-sleep logic or time values
- Modify any files outside src/calendar/

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write completion report to status/workers/feedback-time-warnings.md
