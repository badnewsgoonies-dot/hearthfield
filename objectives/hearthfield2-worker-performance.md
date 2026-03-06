# Hearthfield 2.0 Worker — Performance Wave

## Mission
Implement focused performance improvements from the retrospective without gameplay regressions.

## Required optimizations
1. Minimap redraw optimization
- Avoid full texture rewrite every frame when static layers unchanged.
- Use dirty-state / map-change / actor-movement-driven updates.
- Target file:
  - `src/ui/minimap.rs`

2. HUD proximity scan throttling
- `update_interaction_prompt` should not rescan all interactables/NPC/chests every frame when player tile did not change.
- Add caching keyed by player tile and map.
- Target file:
  - `src/ui/hud.rs`

3. Y-sort pass consolidation
- Replace 3-query pass pattern with single-pass approach where feasible.
- Preserve output behavior (rounded XY + y-based Z rules).
- Target file:
  - `src/world/ysort.rs`

4. Weather particle accounting cleanup
- Remove expensive count pattern (`iter().count()` on two queries each frame) and keep hard cap behavior.
- Maintain indoor-map suppression and weather-specific spawn cadence.
- Target file:
  - `src/world/weather_fx.rs`

5. Farming render change detection (incremental)
- Reduce unnecessary full reconciliation work in farming render sync by introducing minimal dirty checks / early exits.
- Do not break sprite correctness.
- Target files:
  - `src/farming/mod.rs`
  - `src/farming/render.rs`

## Constraints
- No broad architecture rewrite in this wave.
- Keep memory usage reasonable; prefer small caches/resources.
- Preserve deterministic behavior where it exists.

## Validation
Run and pass:
- `cargo check`
- `cargo test --test headless`

## Deliverables
1. Implemented optimizations in listed files.
2. Any targeted tests added for cache invalidation/dirty behavior.
3. Worker report:
- `status/workers/hearthfield2-worker-performance-report.md`
