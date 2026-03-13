verification-only

files changed
- `status/workers/tranche-2-world-graph-travel.md`

validation results
- `cargo check`: PASS
- `cargo test --test headless test_save_roundtrip_preserves_current_map`: PASS
- `cargo test --test headless test_snow_mountain_map_registered`: PASS
- `cargo test --test headless test_farm_to_snow_mountain_transition`: PASS
- `cargo test --test headless test_town_west_map_registered_and_reachable_from_town`: PASS
- `cargo test --test headless test_town_houses_are_accessed_from_town_west_not_town`: PASS

exact reason for any code/test change
- None. All required tranche validations passed in the prescribed order, so no gameplay-code or test changes were made.
- Code inspection did not reveal a concrete uncovered reachable travel seam that justified widening the tranche: `src/world/map_data.rs` still builds the shipped `MapRegistry`, `src/player/interaction.rs` still resolves doors before edges and preserves the fallback travel logic, and `src/save/mod.rs` still restores `current_map` by sending a `MapTransitionEvent` with the saved grid position.

remaining risk
- This tranche reran only the named travel/save validations. It did not rerun broader world-graph surfaces such as `Beach <-> CoralIsland` or `Forest <-> DeepForest`, so those paths remain covered by the existing manifest/test baseline rather than this targeted rerun.
