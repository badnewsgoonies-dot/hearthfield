# Hearthfield 2.0 Worker — Critical Bugs Wave

## Mission
Implement and test the six critical bugs identified in `objectives/orchestrator-hearthfield-2.md` with minimal behavioral side effects.

## Required fixes
1. Time scale mismatch
- Update the default calendar timing so spec behavior is `1 real minute = 10 game minutes`.
- Target files:
  - `src/shared/mod.rs`
  - `src/calendar/mod.rs` (docs/comments/tests if needed)
- Ensure comments reflect actual behavior after fix.

2. NPC cleanup O(n^2)
- In map transition cleanup, replace `Vec::contains` inside `retain` with `HashSet<Entity>` membership.
- Target file:
  - `src/npcs/map_events.rs`

3. Farm west-edge routing
- Fix farm west-edge transition so it routes toward mine pathing intent (via Forest / MineEntrance route), not Beach.
- Target file:
  - `src/player/interaction.rs`
- Update/add map transition tests in this module.

4. Animal baby->adult age
- Change baby->adult threshold from 5 days to 7 days.
- Target file:
  - `src/animals/day_end.rs`
- Update/add tests covering exact threshold behavior.

5. Outside happiness bonus for animals
- Implement explicit happiness bonus for being outside to align with spec.
- Keep implementation deterministic and bounded (saturating math, no runaway growth).
- Suggested approach: use existing map/location signals on animal state; if unavailable, add minimal safe field/logic.
- Target files likely:
  - `src/animals/day_end.rs`
  - `src/shared/mod.rs` (only if type changes are needed)
  - relevant tests

6. Main menu `expect()` crash risk
- Replace production `expect()` path resolution crash with safe fallback behavior and user-facing status.
- Target file:
  - `src/ui/main_menu.rs`

## Constraints
- Preserve existing architecture style unless change is required by bug fix.
- Do not perform unrelated refactors.
- Keep all changes compiling on native target.

## Validation
Run and pass:
- `cargo check`
- `cargo test --test headless`

## Deliverables
1. Code changes implementing all 6 fixes.
2. New/updated tests for edge cases above.
3. Worker report file:
- `status/workers/hearthfield2-worker-critical-bugs-report.md`
- Include: summary, files changed, tests run, any follow-ups.
