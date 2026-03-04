# Worker Report: FIX-NPCS-SPOUSE

## Files Modified
- `src/npcs/romance.rs` (1 file, ~6 lines changed in `update_spouse_happiness`)

## What Was Implemented
Fixed the spouse happiness bug where only gifting counted as daily interaction.
The `update_spouse_happiness` system now checks both `gifted_today` (from Relationships)
and `daily_talks.talked` (from DailyTalkTracker) to determine if the player interacted
with their spouse. This means simply talking to your spouse now properly increases
happiness at end-of-day, matching player expectations.

### Changes in detail:
1. Added `daily_talks: Res<super::dialogue::DailyTalkTracker>` parameter to `update_spouse_happiness`
2. Changed `talked_today` check from only checking `gifted_today` to checking either gifted OR talked:
   ```rust
   let talked_today = spouse_id.is_some_and(|id| {
       relationships.gifted_today.get(id).copied().unwrap_or(false)
           || daily_talks.talked.contains(id)
   });
   ```
3. Updated doc comment to say "gift or conversation" instead of "using gifted_today as a proxy for talking"

## Shared Type Imports Used
- No new shared type imports needed; `DailyTalkTracker` is a local NPC-domain resource accessed via `super::dialogue::DailyTalkTracker`

## Validation Results
- `cargo check` -- PASS (zero errors)
- `cargo clippy -- -D warnings` -- PASS (zero errors, zero warnings)
  - Note: clippy flagged `map_or(false, ...)` and suggested `is_some_and(...)` instead; fixed accordingly

## Known Risks for Integration
- None. The `DailyTalkTracker` resource is already initialized in `NpcPlugin::build()` and reset daily by `reset_daily_talks`. The system parameter addition is fully compatible with Bevy's ECS.
