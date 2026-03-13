# Foreman Lane: Tranche 3 Shipping and Gold

## Scope

You may modify only:

- `src/economy/gold.rs`
- `src/economy/shipping.rs`
- `src/save/mod.rs`
- `tests/headless.rs`
- `status/workers/tranche-3-shipping-and-gold.md`

Do not edit anything else.
If you touch unrelated files, the run is a failure.

## Required Reading

1. `AGENTS.md`
2. `.memory/STATE.md`
3. `docs/HEARTHFIELD_BASE_GAME_RECONSTRUCTION_SPEC.md`
4. `docs/HEARTHFIELD_REPLICATION_TARGET_SPEC.md`
5. `status/research/runtime_surface_manifest.csv`
6. `status/research/test_baseline_manifest.csv`
7. `status/launch/tranche-2-report.md`

## Mission

This is a verification-first tranche.

Target surface:

- shipping bin sale flow remains correct
- gold change application remains correct
- gold still clamps to zero on spend
- multi-day shipping accumulation remains correct
- save/load still preserves shipping/economy state for this surface

## Hard Rules

- Do not broaden into shop entry/UI, upgrades, achievements, or evaluation. Those belong to other tranche-3 slices.
- Do not add a regression unless one of the named validations fails or you find a concrete uncovered seam inside scope.
- If the current code already satisfies the tranche, make no gameplay-code changes and write only the status report.

## Validation Order

Run exactly these first:

```bash
cargo check
cargo test --test headless test_shipping_bin_sells_on_day_end
cargo test --test headless test_gold_clamps_to_zero
cargo test --test headless test_multi_day_shipping_accumulation
cargo test --test headless test_play_stats_tracks_gold_earned
cargo test --test headless test_save_roundtrip_shipping_log
```

Only if those expose a real shipping/gold gap, or if code inspection proves a concrete missing direct regression for this surface, may you patch code or add one narrowly scoped test.

## When Done

Write:

- `status/workers/tranche-3-shipping-and-gold.md`

It must say one of:

- `verification-only`
- `minimal-fix`

And include:

- files changed
- validation results
- exact reason for any code/test change
- remaining risk
