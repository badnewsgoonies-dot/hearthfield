# Foreman Lane: Tranche 2 Collision and Lighting

## Scope

You may modify only:

- `src/world/mod.rs`
- `src/world/lighting.rs`
- `src/world/weather_fx.rs`
- `src/player/camera.rs`
- `tests/headless.rs`
- `status/workers/tranche-2-collision-lighting.md`

Do not edit anything else.
If you touch unrelated files, the run is a failure.

## Required Reading

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
4. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
5. `status/research/runtime_surface_manifest.csv`
6. `status/research/test_baseline_manifest.csv`
7. `status/launch/tranche-1-report.md`

## Mission

This is a verification-first tranche.

Target surface:

- collision stays coherent after map loads and invalidation
- indoor/outdoor lighting stays correct
- indoor maps suppress outdoor weather particles
- camera/map-load handoff stays stable enough for travel parity

## Hard Rules

- Do not broaden into map graph ownership, door routing, or save semantics. That belongs to the other tranche-2 lane.
- Do not add a regression unless one of the named validations fails or you find a concrete uncovered seam inside scope.
- If the current code already satisfies the tranche, make no gameplay-code changes and write only the status report.

## Validation Order

Run exactly these first:

```bash
cargo check
cargo test --test headless test_collision_map_solid_tiles_blocking
cargo test --test headless test_collision_map_cleared_on_invalidation
cargo test --test headless test_world_map_set_solid_toggles
cargo test --test headless test_library_and_tavern_use_indoor_lighting_tint
```

Only if those expose a real collision/lighting gap, or if code inspection proves a concrete missing direct regression for indoor/outdoor parity, may you patch code or add one narrowly scoped test.

## When Done

Write:

- `status/workers/tranche-2-collision-lighting.md`

It must say one of:

- `verification-only`
- `minimal-fix`

And include:

- files changed
- validation results
- exact reason for any code/test change
- remaining risk
