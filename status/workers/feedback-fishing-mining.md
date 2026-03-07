# Worker Report: Fishing + Mining Feedback Toasts

## Status: COMPLETE

## Files Modified
- `src/fishing/cast.rs` (1 line changed): Updated cast-fail toast message from "You need to cast into water!" to "Can't fish here." to match spec.

## Task Results

### Task 1: Cast fails (wrong location) - ALREADY PRESENT, MESSAGE UPDATED
- Location: `src/fishing/cast.rs` line 96-100 (`handle_tool_use_for_fishing`)
- Toast fires when `is_water` check fails (target tile is not `TileKind::Water`)
- Updated message to "Can't fish here." per spec. Duration: 2.0s

### Task 2: Fish escaped (reaction window missed) - ALREADY PRESENT
- Location: `src/fishing/cast.rs` line 314-317 (`handle_bite_reaction_window`)
- Message: "The fish got away!", duration 2.0s

### Task 3: Successful catch toast - ALREADY PRESENT
- Location: `src/fishing/resolve.rs` line 77-80 (`catch_fish`)
- Message: "Caught a {fish_name}!", duration 2.5s

### Task 4: Player takes damage toast - ALREADY PRESENT
- Location: `src/mining/combat.rs` line 257-260 (`enemy_attack_player`)
- Message: "-{damage} HP", duration 1.5s

### Task 5: Floor transition toast - ALREADY PRESENT
- Location: `src/mining/ladder.rs` line 71-74 (`handle_ladder_interaction`)
- Message: "Floor {next_floor}", duration 1.5s

## Validation
- `cargo check`: PASS

## Notes
All five feedback toasts were already implemented. The only change was updating the cast-fail message text to "Can't fish here." to match the spec.
