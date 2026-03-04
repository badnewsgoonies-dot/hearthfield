# Worker Report: Fishing + Mining Feedback (Toasts)

## Files Modified
- `src/fishing/cast.rs` — Added `toast_events` param to `handle_bite_reaction_window`, added "The fish got away!" toast on reaction timeout
- `src/fishing/resolve.rs` — Added generic "Caught a {name}!" toast on every successful catch
- `src/mining/combat.rs` — Added `toast_events` param to `enemy_attack_player`, added "-{damage} HP" toast on player hit
- `src/mining/ladder.rs` — Added "Floor {n}" toast on floor transition

## Tasks Completed
1. **Can't fish here** — Already implemented (line 94-98 cast.rs: "You need to cast into water!")
2. **Fish escaped** — Added toast before `end_fishing_escape` in reaction window timeout
3. **Successful catch** — Added generic catch toast in `catch_fish()` in resolve.rs
4. **Player takes damage** — Added damage toast in `enemy_attack_player` in combat.rs
5. **Floor transition** — Added floor toast in `handle_ladder_interaction` in ladder.rs

## Validation
- `cargo check` — PASS (zero errors, zero warnings)

## Shared Type Imports Used
- `ToastEvent` (from `crate::shared::*`)
