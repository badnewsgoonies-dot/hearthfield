Status: verification-only

Files changed:
- `tests/headless.rs`
- `status/workers/tranche-2-collision-lighting.md`

Validation results:
- `cargo check` — PASS
- `cargo test --test headless test_collision_map_solid_tiles_blocking` — PASS
- `cargo test --test headless test_collision_map_cleared_on_invalidation` — PASS
- `cargo test --test headless test_world_map_set_solid_toggles` — PASS
- `cargo test --test headless test_library_and_tavern_use_indoor_lighting_tint` — PASS
- `cargo test --test headless test_library_and_tavern_suppress_and_cleanup_weather_particles` — PASS
- `cargo test --test headless test_map_transition_primes_camera_snap_and_invalidates_collision_map` — PASS

Exact reason for any code/test change:
- Added one test-only regression in `tests/headless.rs` because the lane had direct coverage for indoor lighting and indoor weather suppression, but no direct proof that `MapTransitionEvent` clears stale collision data and primes `CameraSnap` for the next map-load handoff.
- No gameplay-code changes were required. Code inspection found the existing handoff path already aligns with tranche intent: `src/player/interaction.rs` invalidates collision state and sets `CameraSnap`, `src/world/mod.rs` repopulates `CollisionMap` from `WorldMap`, `src/world/lighting.rs` applies indoor tint for `Library` and `Tavern`, and `src/world/weather_fx.rs` suppresses indoor weather particles.

Remaining risk:
- Camera behavior is now directly verified at the handoff resource/event seam, but not with a full rendered travel loop; the final snap/clamp behavior still relies on code inspection of `src/player/camera.rs` plus existing map-travel coverage elsewhere in the suite.
