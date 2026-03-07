# Worker Report: Data-Driven Maps

## Files Created
- `src/world/map_data.rs` (~510 lines) — MapData types, MapRegistry, loader, export logic
- `assets/maps/farm.ron` (945 lines)
- `assets/maps/town.ron` (792 lines)
- `assets/maps/beach.ron` (327 lines)
- `assets/maps/forest.ron` (613 lines)
- `assets/maps/mine_entrance.ron` (244 lines)
- `assets/maps/mine.ron` (602 lines)
- `assets/maps/player_house.ron` (282 lines)
- `assets/maps/general_store.ron` (170 lines)
- `assets/maps/animal_shop.ron` (170 lines)
- `assets/maps/blacksmith.ron` (170 lines)

## Files Modified
- `src/world/mod.rs` — Added `pub mod map_data;`, `MapRegistry` resource registration, passed registry through `load_map()`, `spawn_initial_map()`, and `handle_map_transition()`
- `src/player/interaction.rs` — Replaced hardcoded `map_bounds()` and `edge_transition()` with data-driven `map_bounds_from_registry()` and `edge_transition_from_registry()`; added `Res<MapRegistry>` parameter to `map_transition_check()` and `handle_map_transition()`; kept hardcoded fallbacks

## What Was Implemented
1. **MapData types**: MapData, ObjectDef, TransitionDef, DoorDef, EdgeTarget, EdgeDefs, BuildingDataDef — all RON-serializable
2. **MapRegistry resource**: HashMap<MapId, MapData> loaded at startup from RON files with hardcoded fallback
3. **10 RON files**: Exported from hardcoded generators with doors, edges, and buildings populated
4. **Data-driven transitions**: map_bounds() and edge_transition() now read from MapRegistry
5. **Hardcoded fallback preserved**: If RON file missing, falls back to generate_*() functions

## Quantitative Targets
- 10 RON files in assets/maps/ — YES (10/10)
- 0 hardcoded coordinates in edge_transition — YES (reads from MapRegistry, hardcoded kept as fallback only)
- map_bounds() reads from MapRegistry — YES
- Fallback to hardcoded generators preserved — YES

## Shared Type Imports Used
- MapId, TileKind, MapTransition, MapTransitionEvent (from src/shared/mod.rs)
- WorldObjectKind (already had Serialize/Deserialize)

## Validation Results
- `cargo check` — PASS (0 warnings)
- `cargo test --test headless` — PASS (128 passed, 0 failed)
- `cargo clippy -- -D warnings` — PASS
- `shasum -a 256 -c .contract.sha256` — PASS (src/shared/mod.rs: OK)

## Known Risks
- MapData.transitions (zone-based) are stored in RON for documentation but NOT used in game transition logic (doors + edges handle everything). This matches the original design where MapTransition data was "for documentation purposes" only.
- Pre-existing test failure in `animals::day_end::tests::outside_animals_get_bounded_happiness_bonus` is unrelated to this work.
