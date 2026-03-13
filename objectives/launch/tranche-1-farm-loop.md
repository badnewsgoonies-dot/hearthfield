# Foreman Lane: Tranche 1 Farm Core Loop

## Scope

You may modify only:

- `src/farming/`
- `src/economy/gold.rs`
- `src/economy/shipping.rs`
- `src/player/tools.rs`
- `tests/headless.rs`
- `status/workers/tranche-1-farm-loop.md`

Do not broaden into shops, social systems, fishing, mining, crafting breadth, or visual-polish work.
Do not edit the shared contract unless it is strictly unavoidable.

## Required Reading

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
4. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
5. `status/research/runtime_surface_manifest.csv`
6. `status/research/test_baseline_manifest.csv`
7. `status/research/visual_mapping_manifest.csv`

## Target Surface

Preserve or reconstruct the day-one farm loop:

- till
- plant
- water
- crop grows under day progression rules
- harvest
- ship
- receive gold

## Non-Goals

- sprinklers beyond preserving current rules
- advanced economy/shop behaviors
- visual-polish beyond not breaking the current visual floor
- fishing/mining/social/crafting breadth

## Required Behavior

- If the current code is already at parity, treat this as verification + targeted hardening only.
- If a real gap exists, make the smallest coherent correction that restores the loop.
- Do not redesign farming.

## Validation

Run:

```bash
cargo check
cargo test --test headless test_full_crop_lifecycle
cargo test --test headless test_shipping_bin_sells_on_day_end
cargo test --test headless test_gold_clamps_to_zero
cargo test --test headless test_season_validation_blocks_wrong_season_crop
```

## When Done

Write:

- `status/workers/tranche-1-farm-loop.md`

Must include:

- files changed
- whether this was verification-only or fix work
- validation results
- remaining risk
