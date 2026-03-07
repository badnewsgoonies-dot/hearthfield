# Worker: ADD-PILOT-HEADLESS-TESTS (Coverage for Untested Domains)

## Scope
You may only modify files under: dlc/pilot/tests/ AND dlc/pilot/src/

## Required reading
1. dlc/pilot/tests/headless.rs (FULL file — understand existing 64 test patterns)
2. dlc/pilot/src/save/mod.rs (lines 1-50 — SaveFile struct, understand save/load flow)
3. dlc/pilot/src/airports/mod.rs (understand AirportPlugin systems)
4. dlc/pilot/src/shared/mod.rs (search for key types used in tests)

## Task: Add Tests for Untested Domains

### Priority 1: Save/Load (4 tests)
Add to headless.rs:

```rust
#[test]
fn test_save_roundtrip() {
    // Create a game state, serialize to SaveFile, deserialize back
    // Verify all fields match
}

#[test]
fn test_save_default_loads_cleanly() {
    // SaveFile::default() should deserialize without errors
}

#[test]
fn test_save_preserves_gold() {
    // Set gold to 500, save, load, verify gold is still 500
}

#[test]
fn test_save_preserves_fleet() {
    // Add aircraft to fleet, save, load, verify aircraft still present
}
```

### Priority 2: Airport Systems (4 tests)
```rust
#[test]
fn test_airport_zone_definitions() {
    // Each airport should have valid zones
}

#[test]
fn test_airport_npc_count() {
    // Each airport should have at least 1 NPC
}

#[test]
fn test_home_base_exists() {
    // HomeBase airport must exist in airport data
}

#[test]
fn test_airport_services_available() {
    // Each airport should have fuel and maintenance services
}
```

### Priority 3: Data Validation (4 tests)
```rust
#[test]
fn test_all_missions_have_valid_routes() {
    // Each mission references valid airport IDs
}

#[test]
fn test_all_items_have_valid_data() {
    // ItemRegistry should have no empty names/descriptions
}

#[test]
fn test_crew_members_have_schedules() {
    // Each crew member should have valid schedule data
}

#[test]
fn test_shop_items_exist_in_registry() {
    // Shop items must reference valid ItemRegistry entries
}
```

Follow the EXACT test patterns from existing tests (create App, add plugins, insert resources, run systems, assert results). Use the same imports and helper functions.

### Validation
```
cd dlc/pilot && cargo test --test headless && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/add-pilot-tests.md
