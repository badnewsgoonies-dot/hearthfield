# Tranche 2 Report

Date: 2026-03-13
Status: complete

## Covered Surfaces

- `world_graph_travel`
- `collision_and_lighting`

## Outcome

- World graph travel closed as `verification-only`.
- Collision and lighting closed as `verification-only` with one added regression for the map-transition handoff seam.
- No gameplay-code changes were required to close tranche 2.

## Validation Highlights

- `cargo test --test headless test_save_roundtrip_preserves_current_map`
- `cargo test --test headless test_snow_mountain_map_registered`
- `cargo test --test headless test_farm_to_snow_mountain_transition`
- `cargo test --test headless test_town_west_map_registered_and_reachable_from_town`
- `cargo test --test headless test_town_houses_are_accessed_from_town_west_not_town`
- `cargo test --test headless test_collision_map_solid_tiles_blocking`
- `cargo test --test headless test_collision_map_cleared_on_invalidation`
- `cargo test --test headless test_world_map_set_solid_toggles`
- `cargo test --test headless test_library_and_tavern_use_indoor_lighting_tint`
- `cargo test --test headless test_library_and_tavern_suppress_and_cleanup_weather_particles`
- `cargo test --test headless test_map_transition_primes_camera_snap_and_invalidates_collision_map`

All passed.

## What We Learned

- The tranche is distinguishing reachable-world proof gaps from actual missing implementation.
- Indoor/outdoor lighting and collision invalidation were already structurally correct; the weak point was direct proof at the map-transition handoff seam.
- The reconstruction target continues to behave more like quality-floor verification than broad rebuild work.

## Residual Risk

- Tranche 2 still does not directly prove every reachable travel loop in the shipped graph, especially the broader outdoor paths beyond the named reruns.
- Camera behavior is now verified at the resource/event seam, but not yet through a rendered end-to-end travel capture.

## Next Step

Launch tranche 3:

- `shipping_and_gold`
- `town_shop_loop`
