# Tranche 1 Report

Date: 2026-03-12
Status: complete

## Covered Surfaces

- `boot_main_menu`
- `player_spawn_house_exit_sleep`
- `farm_core_loop`

## Outcome

- Farm core loop closed as `verification-only`.
- Player/day-start closed as `verification-only` with one added regression for the bed sleep/day rollover seam.
- No gameplay-code changes were required to close tranche 1.

## Validation Highlights

- `cargo test --test headless test_full_crop_lifecycle`
- `cargo test --test headless test_shipping_bin_sells_on_day_end`
- `cargo test --test headless test_gold_clamps_to_zero`
- `cargo test --test headless test_season_validation_blocks_wrong_season_crop`
- `cargo test --test headless test_headless_boot_smoke_transitions_and_ticks`
- `cargo test --test headless test_save_roundtrip_preserves_current_map`
- `cargo test --test headless test_starter_items_include_hoe`
- `cargo test --test headless test_sleep_rollover_advances_day_before_cutscene_state_change`

All passed.

## What We Learned

- The tranche is distinguishing missing proof from missing implementation.
- The tranche is distinguishing bug-fixing from verification hardening.
- The playable spine is substantially present in the current base; the weakest points were proof gaps at specific seams, not broad missing systems.

## Residual Risk

- Main-menu `Start Game` remains verified by smoke coverage plus code inspection rather than rendered UI automation.
- Tranche 1 does not prove broader world graph, collision/lighting parity, or non-farm travel loops.

## Next Step

Launch tranche 2:

- `objectives/launch/tranche-2-world-graph-travel.md`
- `objectives/launch/tranche-2-collision-lighting.md`
