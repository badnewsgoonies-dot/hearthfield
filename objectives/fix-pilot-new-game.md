# Worker: FIX-PILOT-NEW-GAME (Starter Resources + Aircraft)

## Scope
You may only modify files under: dlc/pilot/src/

## Required reading
1. dlc/pilot/src/player/spawn.rs (FULL file — setup_new_game function at lines 89-104)
2. dlc/pilot/src/player/mod.rs (FULL file — PlayerPlugin::build(), find where OnEnter(Playing) systems are)
3. dlc/pilot/src/aircraft/fleet.rs (search for Fleet, how to add starter aircraft)
4. dlc/pilot/src/shared/mod.rs (search for PilotState, Gold, Fleet, GameState)
5. dlc/pilot/src/data/aircraft.rs (find Cessna 172 or smallest aircraft definition)

## Bug: Player Starts With 0 Gold, Empty Inventory, No Aircraft

### Root Cause
`setup_new_game()` in `player/spawn.rs` (lines 89-104) is defined but NEVER registered as a system. It should run on `OnEnter(GameState::Playing)` but isn't added in `PlayerPlugin::build()`.

This means:
- Player starts with 0 gold (should be 500 STARTER_GOLD)
- Inventory is empty (should have pilot_manual, local_map, granola_bar x3, water_bottle x2)
- Welcome toast never shows
- No starter aircraft in fleet

### Fix Required

#### 1. Register setup_new_game in PlayerPlugin::build()
In `dlc/pilot/src/player/mod.rs`, add `setup_new_game` to the `OnEnter(GameState::Playing)` systems:
```rust
.add_systems(OnEnter(GameState::Playing), spawn::setup_new_game)
```

BUT — we need a guard so this only runs on NEW games, not on every transition to Playing. Check if there's a flag or if Gold is already > 0. If no guard exists, add one:

In setup_new_game, add an early return if gold is already nonzero:
```rust
if gold.amount > 0 {
    return; // Already initialized (loaded game or returning from flight)
}
```

#### 2. Add starter aircraft to setup_new_game
After the existing inventory additions, add a starter aircraft:
```rust
// Give starter aircraft — search for the cheapest/smallest aircraft ID in the registry
// Look at dlc/pilot/src/data/aircraft.rs to find the exact ID
fleet.add_aircraft("cessna_172"); // or whatever the ID is
```

Read the Fleet struct to understand how to add an aircraft. It might be `fleet.aircraft.push(...)` or `fleet.add(...)`.

#### 3. Verify the toast system works
The welcome toast should fire after setup_new_game runs. Verify ToastEvent is properly registered.

### Validation
```
cd dlc/pilot && cargo check && cargo test --lib && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-pilot-new-game.md
