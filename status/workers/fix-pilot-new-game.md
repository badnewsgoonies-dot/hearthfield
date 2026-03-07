# Worker Completion Report: FIX-PILOT-NEW-GAME

## Files Modified
- `dlc/pilot/src/player/spawn.rs` — added `HangarAssignments` import, expanded `setup_new_game` signature and body (+20 lines)
- `dlc/pilot/src/player/mod.rs` — registered `setup_new_game` in `OnEnter(GameState::Playing)` (+1 char change)

## What Was Implemented

### 1. Registered `setup_new_game` in `PlayerPlugin::build()`
Changed single-system registration to a tuple so both `spawn_player` and `setup_new_game` run on `OnEnter(GameState::Playing)`:
```rust
.add_systems(OnEnter(GameState::Playing), (spawn::spawn_player, spawn::setup_new_game))
```

### 2. Guard against re-initialization
Added early return if gold is already nonzero (handles load-game and zone transitions back to Playing):
```rust
if gold.amount > 0 {
    return; // Already initialized (loaded game or returning from flight)
}
```

### 3. Starter aircraft
Added `Fleet` and `HangarAssignments` parameters and inserted a starter `cessna_172` named "Old Faithful" (condition 65%, 30 fuel, 47 prior flights) into the fleet and assigned it to `AirportId::HomeBase`. Guard is `fleet.aircraft.is_empty()` to avoid duplicating on hypothetical re-entry.

### 4. Import added
`use crate::aircraft::fleet::HangarAssignments;` in `player/spawn.rs`.

## Quantitative Targets Hit
- Starter gold: 500 (STARTER_GOLD constant)
- Starter inventory items: 5 (pilot_manual x1, local_map x1, granola_bar x3, water_bottle x2)
- Starter aircraft: 1 (cessna_172 "Old Faithful")
- Welcome toast: fires on new game entry

## Shared Type Imports Used
- `Gold`, `Inventory`, `Fleet`, `OwnedAircraft`, `ToastEvent`, `AirportId` (all from `crate::shared::*`)
- `HangarAssignments` (from `crate::aircraft::fleet`)

## Validation Results
```
cargo check          — PASS
cargo test --lib     — PASS (4/4 tests)
cargo clippy -- -D warnings — PASS (0 warnings)
```

## Known Risks
- The guard `gold.amount > 0` means a player who starts with exactly 0 gold (after spending all their starter gold and saving) will re-trigger the setup on their next `OnEnter(Playing)`. This is unlikely in practice but could be addressed with a dedicated `NewGameInitialized` marker resource if needed.
