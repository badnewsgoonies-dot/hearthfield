# Worker Report: ADD-PILOT-HEADLESS-TESTS

## Files Modified
- `dlc/pilot/tests/headless.rs` — added 12 tests and 8 new imports (~165 lines added, total 1688 lines)

## What Was Implemented

### Priority 1: Save/Load (4 tests)
- `test_save_roundtrip` — serializes SaveFile with modified gold/calendar, deserializes, verifies all fields
- `test_save_default_loads_cleanly` — verifies `SaveFile::default()` roundtrips through serde_json without error
- `test_save_preserves_gold` — sets gold to 500, serializes, deserializes, asserts gold == 500
- `test_save_preserves_fleet` — adds OwnedAircraft to fleet, serializes, deserializes, asserts aircraft count and fields

### Priority 2: Airport Systems (4 tests)
- `test_airport_zone_definitions` — calls `generate_zone_map` for all 8 MapZone variants, verifies non-zero dimensions and tile array shape
- `test_airport_npc_count` — sends ZoneTransitionEvent to Terminal zone, runs `spawn_ambient_npcs`, asserts ≥1 AirportNpc entity spawned
- `test_home_base_exists` — verifies HomeBase has non-empty display_name/ICAO code and unlocks at Student rank
- `test_airport_services_available` — verifies all 10 airports offer WeatherBriefing and AirportMap services

### Priority 3: Data Validation (4 tests)
- `test_all_missions_have_valid_routes` — calls `get_mission_templates()`, asserts non-empty list, checks id/title/reward_gold for each
- `test_all_items_have_valid_data` — populates ItemRegistry, checks every item has non-empty name/description and matching id key
- `test_crew_members_have_schedules` — iterates CREW_IDS, calls `get_schedule()` for each, asserts weekday and weekend entries are non-empty
- `test_shop_items_exist_in_registry` — for all 10 airports calls `get_shop_inventory()`, verifies every item_id resolves in ItemRegistry

## New Imports Added
- `skywarden::save::SaveFile`
- `skywarden::airports::maps::generate_zone_map`
- `skywarden::airports::services::{services_at, AirportService}`
- `skywarden::airports::npcs::{AirportNpc, spawn_ambient_npcs}`
- `skywarden::data::items::populate_items`
- `skywarden::data::missions::get_mission_templates`
- `skywarden::data::shops::get_shop_inventory`
- `skywarden::crew::schedules::get_schedule`

## Validation Results
- `cargo test --test headless`: **76 passed, 0 failed** (was 64, now 76)
- `cargo clippy -- -D warnings`: **PASS** (no warnings)

## Known Risks
- None. All tests use pure functions or minimal ECS apps with MinimalPlugins.
