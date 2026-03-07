# impl_save_position — Done

## Changes

### src/shared/mod.rs — PlayerState
- Added `save_grid_x: i32` and `save_grid_y: i32` fields with `#[serde(default)]` for backward-compatible save loading.
- Default impl sets both to `8`.

### src/save/mod.rs — load handler
- Replaced the `match player_state.current_map { ... }` block that mapped MapId → hardcoded spawn coords with:
  ```rust
  let spawn_x = player_state.save_grid_x;
  let spawn_y = player_state.save_grid_y;
  ```
- The `MapTransitionEvent` now uses the persisted grid position instead of a per-map default.

## Notes
- Writing `save_grid_x`/`save_grid_y` must be done in the player module (out of scope). These fields default to `(8, 8)` until the player module is updated to write them.
- Old saves without these fields will deserialize to `(0, 0)` via `serde(default)` for `i32`; consider whether the player module should clamp/validate on load.
