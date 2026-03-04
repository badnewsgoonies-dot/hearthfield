# Worker Report: fix-bed-sleep

## Files Modified
- `src/player/interact_dispatch.rs` — ~8 lines changed

## Lines Changed
- Removed `mut day_end_events: EventWriter<DayEndEvent>` parameter (line 35)
- Restructured `InteractionKind::Bed` arm (lines 118–136): moved `interaction_claimed.0 = true` inside the `hour < 18` branch, removed the `else` branch entirely (toast + DayEndEvent)

## Old Behavior
1. Player presses F near bed
2. `dispatch_world_interaction` fires, sets `interaction_claimed.0 = true` unconditionally
3. If hour >= 18: sends "Goodnight!" toast + DayEndEvent — no cutscene, day transitions silently
4. `trigger_sleep` in calendar sees `interaction_claimed.0 == true`, returns early — **cutscene never fires**

## New Behavior
1. Player presses F near bed
2. `dispatch_world_interaction` fires
3. If hour < 18: claims interaction, shows "too early" toast (unchanged)
4. If hour >= 18: does NOT claim interaction, does NOT send DayEndEvent
5. `trigger_sleep` in calendar sees unclaimed interaction, proceeds with full cutscene flow: fade out → season text → day label → DayEndEvent → fade in

## Why This Works
`trigger_sleep` (`src/calendar/mod.rs`) already implements the complete sleep flow with cutscene queuing. The only reason it was skipped was because `interact_dispatch` was claiming the interaction first. By leaving the interaction unclaimed when hour >= 18, `trigger_sleep` runs normally and handles everything.

## Validation
`cargo check` — passes with zero errors, zero warnings.
