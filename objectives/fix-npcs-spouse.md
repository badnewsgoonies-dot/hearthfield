# Worker: FIX-NPCS (Spouse Happiness)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/npcs/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/npcs/romance.rs (read the FULL file — the fix is in update_spouse_happiness)
2. src/npcs/dialogue.rs (read just the DailyTalkTracker struct)
3. src/npcs/mod.rs (check imports)

## Bug: Spouse Happiness Only Increases When Gifting

### Root Cause
`update_spouse_happiness` in `src/npcs/romance.rs:527-564` checks `relationships.gifted_today.get(spouse_id)` to determine if the player interacted with their spouse. This means ONLY gifting counts — simply talking to your spouse does nothing for happiness. Now that we have `DailyTalkTracker` (from the dialogue fix), talking should also count.

### Fix Required
In `update_spouse_happiness`:
1. Add `daily_talks: Res<super::dialogue::DailyTalkTracker>` parameter
2. Change the `talked_today` check to use EITHER gifted OR talked:
```rust
let talked_today = spouse_id.map_or(false, |id| {
    relationships.gifted_today.get(id).copied().unwrap_or(false)
        || daily_talks.talked.contains(id)
});
```
3. Update the comment to say "Check if the spouse was interacted with today (gift or conversation)"

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-npcs-spouse.md
