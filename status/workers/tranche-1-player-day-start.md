verification-only

files changed
- `tests/headless.rs`
- `status/workers/tranche-1-player-day-start.md`

validation results
- `cargo check`: PASS
- `cargo test --test headless test_headless_boot_smoke_transitions_and_ticks`: PASS
- `cargo test --test headless test_save_roundtrip_preserves_current_map`: PASS
- `cargo test --test headless test_starter_items_include_hoe`: PASS
- `cargo test --test headless test_sleep_rollover_advances_day_before_cutscene_state_change`: PASS

exact reason for any code/test change
- Added one narrowly scoped headless regression test for the bed sleep/day rollover seam because the required gates and existing test manifest did not directly prove that `DayEndEvent` handling completes before the queued overnight cutscene changes state.
- No gameplay code changed. Code inspection already showed the tranche surface wiring in scope:
  `src/player/spawn.rs` starts a new game in `PlayerHouse`,
  `src/player/interaction.rs` plus `src/world/map_data.rs` preserve house exit/re-entry coordinates,
  and the new test closes the remaining proof gap for sleep rollover ordering.

remaining risk
- This rerun did not add UI automation for selecting `Start Game` from the rendered main menu; that part remains verified by the existing boot smoke, code inspection of `src/ui/main_menu.rs`, and the unchanged tranche wiring rather than a direct end-to-end input test.
