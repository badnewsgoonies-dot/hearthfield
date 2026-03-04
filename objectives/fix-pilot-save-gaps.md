# Worker: FIX-PILOT-SAVE-GAPS (Missing Resources in Save File)

## Scope
You may only modify files under: dlc/pilot/src/

## Required reading
1. dlc/pilot/src/save/mod.rs (FULL file — FullSaveFile struct, write_save, handle_load, handle_new_game)
2. dlc/pilot/src/shared/mod.rs — search for NavigationState, FlightState, MaintenanceTracker, FuelWarnings, TransactionLog, GoldMilestones
3. dlc/pilot/src/economy/ — search for TransactionLog, GoldMilestones definitions
4. dlc/pilot/src/aircraft/ — search for MaintenanceTracker, FuelWarnings definitions

## Task: Audit and Fix Save File Gaps

### What to do
1. Read the FullSaveFile struct — list ALL fields currently saved
2. Search for ALL `init_resource` calls across dlc/pilot/src/ — list every Resource
3. For each Resource that tracks GAME STATE (not UI/runtime/asset handles):
   - Check if it has Serialize/Deserialize derives
   - Check if it's in FullSaveFile
   - If missing and important, add it

### Resources likely missing (investigate each):
- MaintenanceTracker (aircraft health data)
- TransactionLog (economy history)
- GoldMilestones (achievement tracking)
- FuelWarnings (aircraft fuel state)
- NavigationState (may be runtime-only — check)

### Pattern to follow for each missing resource:
1. Add serde derives if missing
2. Add `#[serde(default)]` field to FullSaveFile
3. Add to ExtendedResources (if that pattern exists) or write_save params
4. Add to handle_load_request restoration
5. Add to handle_new_game reset

### Validation
```
cd dlc/pilot && cargo check && cargo test --lib && cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-pilot-save-gaps.md
