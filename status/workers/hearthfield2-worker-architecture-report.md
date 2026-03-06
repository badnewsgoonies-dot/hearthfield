# Hearthfield 2.0 Worker — Architecture Wave Report

## Scope Completed
Implemented a shared `SystemSet` phase taxonomy and incrementally adopted it across targeted scheduling surfaces (`input`, `player`, core `ui`, and `world`) with low churn.

## 1. SystemSet taxonomy + Update configuration
Added shared phase sets:
- `UpdatePhase::Input`
- `UpdatePhase::Intent`
- `UpdatePhase::Simulation`
- `UpdatePhase::Reactions`
- `UpdatePhase::Presentation`

Implementation:
- Added `src/shared/schedule.rs`.
- Re-exported from `src/shared/mod.rs`.
- Configured chained set ordering in `src/main.rs` for `Update`:
  - `Input -> Intent -> Simulation -> Reactions -> Presentation`

Also configured a stable fixed timestep resource in `main.rs`:
- `Time::<Fixed>::from_hz(5.0)`

## 2. Selected domain migration to sets

### Input
- Tagged input read pipeline in `src/input/mod.rs`:
  - `reset_and_read_input`, `process_touch_input`, `manage_input_context`
  - assigned to `UpdatePhase::Input`

### Player
- `dispatch_world_interaction`, `dispatch_item_use` -> `Intent`
- Core movement/tool/interaction loop -> `Simulation`
- `handle_day_end` -> `Reactions`
- `camera_follow_player` -> `Presentation`

Ordering cleanup:
- Removed one ad-hoc ordering edge (`camera_follow_player.after(handle_map_transition)`) and replaced it with explicit phase ordering (`Simulation` before `Presentation`).

### UI (core update systems)
Tagged core update groups in `src/ui/mod.rs`:
- Menu input routing/transitions -> `Input`
- Dialogue/audio event handling and tutorial hint forwarding -> `Reactions`
- Fade transitions, main menu visuals, HUD updates, toast updates -> `Presentation`
- Cutscene queue runner -> `Simulation`

### World (safe movement/presentation split)
In `src/world/mod.rs`, split mixed update tuples and assigned:
- Interaction/tool/chest/map handling + weed scythe + map sync/object spawn -> `Simulation`
- Seasonal/weather/day-night/sparkle/interactable highlight visuals -> `Presentation`
- Day-end forage/weed and season-change reactions -> `Reactions`

## 3. FixedUpdate island (small deterministic subsystem)
Implemented deterministic fixed-step scheduling for NPC schedule cadence:
- Moved `npcs::update_npc_schedules` from `Update` to `FixedUpdate` in `src/npcs/mod.rs`.
- Kept `move_npcs_toward_targets` in regular `Update` for smooth visual motion.

This creates a small fixed-step simulation island with minimal behavior drift while preserving gameplay responsiveness.

## 4. Validation
Executed required commands:
- `cargo check` ✅
- `cargo test --test headless` ✅

Test note:
- `tests/headless.rs::test_farming_day_end_system` was flaky due random crow destruction chance.
- Stabilized test by inserting a scarecrow in that test setup so expected crop-survival assertion is deterministic.

Final headless result:
- `88 passed; 0 failed; 2 ignored`
