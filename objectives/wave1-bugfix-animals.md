# Worker: Wave 1-A — Animal Test Fix + Outside Happiness

## Objective
Fix the failing test and implement the missing "being outside" happiness bonus for animals.

## Bug 1: Failing test has stale expectations
The test `test_baby_animal_grows_to_adult` in `tests/headless.rs` line 703-738 expects animals to mature after 5 days, but the code correctly uses 7 days per the game spec (GAME_SPEC.md line 84).

**Fix required:**
- Change `days_old: 4` to `days_old: 6` in the test
- Change the assertion message from "5 days" to "7 days"
- Ensure `animal.days_old` assertion expects `7` not `5`

## Bug 2: Missing "being outside" happiness bonus
Per GAME_SPEC.md line 81: animals should gain happiness from "being outside". This is not implemented.

**Location:** `src/animals/day_end.rs` in the `handle_day_end_for_animals` system.

**Implementation:**
- Add a check if animal was outside during the day (you may need to add a field `was_outside_today: bool` to the `Animal` component in `src/shared/mod.rs`)
- If the animal was outside, add +5 to +10 happiness in the day-end processing
- The field should reset to false at day end like `fed_today` and `petted_today`

## Files to modify
1. `tests/headless.rs` — Fix test expectations (lines 719, 735-737)
2. `src/animals/day_end.rs` — Add outside happiness bonus logic
3. `src/shared/mod.rs` — Add `was_outside_today: bool` to `Animal` struct if needed

## Validation
Run these commands before reporting done:
```bash
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

All must pass with zero errors and zero warnings. The test `test_baby_animal_grows_to_adult` must pass.

## Do NOT modify
- Any files outside the scope above
- Game balance constants beyond what's specified
- Other tests unless they break due to your changes
