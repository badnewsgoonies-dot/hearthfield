# Foreman Lane: Tranche 1 Player Day Start Rerun

## Scope

You may modify only:

- `src/main.rs`
- `src/player/`
- `src/ui/main_menu.rs`
- `src/ui/tutorial.rs`
- `src/ui/intro_sequence.rs`
- `src/world/map_data.rs`
- `tests/headless.rs`
- `status/workers/tranche-1-player-day-start.md`

Do not edit anything else.
Do not introduce changes in `src/crafting/`, `src/mining/`, `src/npcs/`, or `src/world/objects.rs`.
If you touch any out-of-scope file, the run is a failure.

## Required Reading

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
4. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
5. `status/research/runtime_surface_manifest.csv`
6. `status/research/test_baseline_manifest.csv`
7. `status/research/reachable_surface_manifest.csv`

## Mission

This is a verification-first rerun.

Your first job is to determine whether the current code already satisfies the tranche surface.

Surface:

- boot -> main menu
- start game
- player spawns correctly in `PlayerHouse`
- player can exit to `Farm`
- player can re-enter `PlayerHouse`
- bed sleep advances the day without breaking spawn/map-transition ordering

## Hard Rules

- Do not add a new regression test unless one of the named validation gates fails or you find a concrete uncovered parity seam inside scope.
- The most likely seam is the sleep/day rollover path. Treat that as the only acceptable place for new targeted test coverage if the current tests do not prove it directly.
- Do not add unrelated hardening tests.
- Do not broaden the target surface.
- If the current code already satisfies the tranche, make no gameplay-code changes and write only the status report.

## Validation Order

Run exactly these first:

```bash
cargo check
cargo test --test headless test_headless_boot_smoke_transitions_and_ticks
cargo test --test headless test_save_roundtrip_preserves_current_map
cargo test --test headless test_starter_items_include_hoe
```

Only if those expose a real player-day-start gap, or if code inspection confirms there is no direct regression test for the bed sleep/day rollover seam, may you patch code or add one narrowly scoped test.

Focus if you inspect code:

- player spawn in `PlayerHouse`
- house exit -> `Farm`
- re-entry -> `PlayerHouse`
- bed interact -> day advance -> reposition
- any ordering dependency between `DayEndEvent` handling and cutscene activation

## When Done

Write:

- `status/workers/tranche-1-player-day-start.md`

It must say one of:

- `verification-only`
- `minimal-fix`

And include:

- files changed
- validation results
- exact reason for any code/test change
- remaining risk
