# Foreman Lane: Tranche 1 Player Day Start

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

Do not edit the shared contract unless the task proves it is impossible otherwise.
Do not broaden into economy, shops, social, fishing, mining, or visual-polish work.

## Required Reading

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
4. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
5. `status/research/runtime_surface_manifest.csv`
6. `status/research/test_baseline_manifest.csv`
7. `status/research/reachable_surface_manifest.csv`

## Target Surface

Preserve or reconstruct this spine:

- boot -> main menu
- start game
- player spawns correctly in `PlayerHouse`
- player can exit house to `Farm`
- player can act in the world
- sleep advances the day

## Non-Goals

- farm crop lifecycle beyond what is necessary for spawn/exit/day-start flow
- shop/economy systems
- social systems
- fishing/mining/crafting breadth
- broad UI or visual polish

## Required Behavior

- If the current code is already at parity for this surface, treat the lane as verification + targeted hardening only.
- If you find a real gap, make the smallest coherent correction that restores the target surface.
- Do not invent new systems.

## Validation

Run:

```bash
cargo check
cargo test --test headless test_headless_boot_smoke_transitions_and_ticks
cargo test --test headless test_save_roundtrip_preserves_current_map
cargo test --test headless test_starter_items_include_hoe
```

## When Done

Write:

- `status/workers/tranche-1-player-day-start.md`

Must include:

- files changed
- whether this was verification-only or fix work
- validation results
- remaining risk
