# Mining Combat Implementation Report

**Date:** 2026-03-02
**File modified:** `src/mining/combat.rs`

---

## Changes Made

### 1. Y-axis bounds off-by-one fix (`enemy_ai_movement`, line 183)
- **Before:** `ny < 0`
- **After:** `ny < 1`
- Prevents enemies from walking onto wall row y=0, matching the x-axis lower-bound convention (`nx < 1`).

### 2. Reset `PlayerIFrames` on knockout (`check_player_knockout`)
- Added `mut iframes: ResMut<PlayerIFrames>` to system parameters.
- After `player_state.health = player_state.max_health * 0.5;`, added:
  ```rust
  iframes.timer = Timer::from_seconds(0.0, TimerMode::Once);
  ```
- Ensures any active iframe window from the fatal hit is cleared before re-entry into the mine.

### 3. `despawn_recursive()` on enemy kill (`handle_player_attack`, line 66)
- **Before:** `commands.entity(entity).despawn()`
- **After:** `commands.entity(entity).despawn_recursive()`
- Safely handles future cases where enemies gain child entities (health bars, animations, shadows).

### 4. Stamina drain on player attack (`handle_player_attack`)
- Added `mut stamina_events: EventWriter<StaminaDrainEvent>` to system parameters.
- After each pickaxe `ToolUseEvent` is processed (regardless of whether an enemy was hit), sends:
  ```rust
  stamina_events.send(StaminaDrainEvent { amount: 2.0 });
  ```
- `StaminaDrainEvent` was confirmed present in `src/shared/mod.rs` (line 893).

---

## Assumptions

- **Stamina drain fires per swing (not per hit):** The task said "after the ToolUseEvent processing" — interpreted as once per pickaxe event processed, matching the loop body boundary. This means swinging into empty space also costs 2 stamina (consistent with how tool use stamina costs typically work).
- **`Timer::from_seconds(0.0, TimerMode::Once)` resets iframes:** A 0-second already-elapsed timer puts `iframes.timer.finished()` into `true` state immediately, effectively clearing any remaining invincibility window. This is consistent with the pattern used in `enemy_attack_player` for creating new timers.
- **`cargo check --lib` passed with exit code 0** — no compilation errors introduced.
