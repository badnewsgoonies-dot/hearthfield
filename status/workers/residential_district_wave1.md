# Residential District Wave 1

## Scope
- Added a new outdoor `TownWest` district west of `Town`.
- Rewired `Town.west` to `TownWest` and moved `TownHouseWest` / `TownHouseEast` access into `TownWest`.
- Reassigned starter housed residents:
  - `TownHouseWest`: `doc`, `mira`
  - `TownHouseEast`: `old_tom`, `sam`
- Added focused regression coverage for the new route, moved house access, housed schedules, and the preserved `Farm -> MineEntrance` west path.

## Files Changed
- `src/shared/mod.rs`
- `.contract.sha256`
- `src/world/maps.rs`
- `src/world/map_data.rs`
- `src/world/mod.rs`
- `src/player/interaction.rs`
- `src/data/npcs.rs`
- `src/npcs/schedules.rs`
- `src/ui/map_screen.rs`
- `src/ui/hud.rs`
- `src/ui/audio.rs`
- `tests/headless.rs`
- `assets/maps/town.ron`
- `assets/maps/town_house_west.ron`
- `assets/maps/town_house_east.ron`
- `assets/maps/town_west.ron`

## Verification
- `cargo check`
- `cargo test --test headless test_town_west_map_registered_and_reachable_from_town -- --exact`
- `cargo test --test headless test_town_houses_are_accessed_from_town_west_not_town -- --exact`
- `cargo test --test headless test_residential_starter_residents_have_housed_home_entries -- --exact`
- `cargo test --test headless test_farm_west_still_routes_to_mine_entrance -- --exact`

## Residual Risk
- The new district uses existing town ambience/music and the existing house interiors; if a later slice wants custom atmosphere or distinct outdoor props/NPC blockers, that can layer on without changing the route contract added here.
