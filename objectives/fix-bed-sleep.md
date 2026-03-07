# Worker: Unify Bed Sleep Path to Use Cutscene Flow

## Context
There are TWO sleep paths that conflict:

1. `src/calendar/mod.rs` → `trigger_sleep` system: checks interact + PlayerHouse + proximity to bed → sends DayEndEvent + queues full cutscene (fade out, season text, day label, fade in). This is the GOOD path.

2. `src/player/interact_dispatch.rs` → `InteractionKind::Bed` handler: checks interact on Bed entity → sets `interaction_claimed.0 = true` → sends DayEndEvent + toast "Goodnight!". NO cutscene. This is the BROKEN path.

Because the Bed handler sets `interaction_claimed.0 = true` FIRST, `trigger_sleep` sees `interaction_claimed.0 == true` and returns early. The cutscene NEVER fires when using the proximity prompt.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/player/`
All out-of-scope edits will be reverted.

## Required reading
1. src/player/interact_dispatch.rs — the `InteractionKind::Bed` arm (~lines 118-137)
2. src/calendar/mod.rs — READ ONLY. Understand that `trigger_sleep` already handles the full sleep flow with cutscene, DayEndEvent, etc. It checks: interact pressed, not claimed, PlayerHouse map, near bed (12,3), cutscene not active.

## Task

In `src/player/interact_dispatch.rs`, modify the `InteractionKind::Bed` handler:

**Keep** the hour < 18 check — this is the "too early" guard that trigger_sleep lacks.
When hour < 18: claim interaction, show toast. (Same as now.)

**Change** the else branch (hour >= 18): Do NOT claim interaction, do NOT send DayEndEvent, do NOT send toast. Simply return/break so that `trigger_sleep` in calendar handles the full sleep + cutscene.

The result should look like:
```rust
InteractionKind::Bed => {
    if calendar.hour < 18 {
        interaction_claimed.0 = true;
        toast_events.send(ToastEvent {
            message: "It's too early to sleep. Come back after 6 PM.".into(),
            duration_secs: 3.0,
        });
    }
    // If hour >= 18, intentionally do NOT claim interaction.
    // This allows trigger_sleep (in calendar) to handle the full
    // sleep flow with cutscene, DayEndEvent, and day transition.
}
```

## Do NOT
- Modify src/shared/mod.rs or src/calendar/
- Add cutscene logic to this file
- Change any other InteractionKind handlers
- Move interaction_claimed.0 = true outside the hour < 18 branch

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write a brief report to status/workers/fix-bed-sleep.md listing:
- Lines changed
- Old behavior vs new behavior
- Why this works (trigger_sleep handles the rest)
