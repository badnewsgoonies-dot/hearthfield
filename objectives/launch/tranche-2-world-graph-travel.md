# Foreman Lane: Tranche 2 World Graph Travel

## Scope

You may modify only:

- `src/world/map_data.rs`
- `src/world/mod.rs`
- `src/player/interaction.rs`
- `src/save/mod.rs`
- `assets/maps/`
- `tests/headless.rs`
- `status/workers/tranche-2-world-graph-travel.md`

Do not edit anything else.
If you touch unrelated files, the run is a failure.

## Required Reading

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
4. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
5. `status/research/runtime_surface_manifest.csv`
6. `status/research/reachable_surface_manifest.csv`
7. `status/research/test_baseline_manifest.csv`
8. `status/launch/tranche-1-report.md`

## Mission

This is a verification-first tranche.

Target surface:

- map registry/loading stays correct
- world travel preserves map identities and spawn positions
- interior/exterior door and edge travel stay coherent
- reachable surface links in the shipped world graph remain intact
- save/load still preserves `current_map` across world travel

## Hard Rules

- Do not broaden into lighting, weather particles, or y-sort polish. That belongs to the other tranche-2 lane.
- Do not add a regression unless one of the named validations fails or you find a concrete uncovered travel seam inside scope.
- If the current code already satisfies the tranche, make no gameplay-code changes and write only the status report.

## Validation Order

Run exactly these first:

```bash
cargo check
cargo test --test headless test_save_roundtrip_preserves_current_map
cargo test --test headless test_snow_mountain_map_registered
cargo test --test headless test_farm_to_snow_mountain_transition
cargo test --test headless test_town_west_map_registered_and_reachable_from_town
cargo test --test headless test_town_houses_are_accessed_from_town_west_not_town
```

Only if those expose a real travel/world-graph gap, or if code inspection proves a concrete missing direct regression for a reachable travel seam, may you patch code or add one narrowly scoped test.

Focus if you inspect code:

- `MapRegistry` build/load path
- map transition detection and application
- door versus edge ownership
- `current_map` persistence on transition and save/load

## When Done

Write:

- `status/workers/tranche-2-world-graph-travel.md`

It must say one of:

- `verification-only`
- `minimal-fix`

And include:

- files changed
- validation results
- exact reason for any code/test change
- remaining risk
