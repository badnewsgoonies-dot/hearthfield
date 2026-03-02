# Feeding Proximity Fix

## What changed
`src/animals/feeding.rs` — `handle_feed_trough_interact`

## Problem
The system fed ALL productive animals whenever any `hay` `ItemRemovedEvent` fired, regardless of where the player was standing.

## Fix
Added a proximity check before processing the hay removal:

- Added `player_query: Query<&GridPosition, With<Player>>` to fetch the player's current grid cell.
- Changed `_trough_query: Query<&FeedTrough>` (unused) to an active query.
- Before iterating animals, compute whether the player is within Manhattan distance ≤ 2 of any `FeedTrough` entity (using its `grid_x`/`grid_y` fields).
- If the player is **not** near a trough, the event is consumed but feeding is skipped.

## Pattern used
Same Manhattan distance check used in `src/mining/combat.rs` and `src/mining/ladder.rs` (`dist <= 1`/`<= 2`). FeedTrough stores `grid_x`/`grid_y` directly, so no Transform comparison was needed.

## Files modified
- `src/animals/feeding.rs` (only)
