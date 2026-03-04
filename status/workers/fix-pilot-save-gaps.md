# Worker Report: FIX-PILOT-SAVE-GAPS

## Files Modified

| File | Change |
|------|--------|
| `dlc/pilot/src/economy/gold.rs` | Added `use serde::{Deserialize, Serialize};`; added `Serialize, Deserialize` + `Clone` to `Transaction`, `TransactionLog`, `GoldMilestones` |
| `dlc/pilot/src/aircraft/fuel.rs` | Added `use serde::{Deserialize, Serialize};`; added `Serialize, Deserialize` + `Clone` to `FuelWarnings` |
| `dlc/pilot/src/save/mod.rs` | Added imports for `FuelWarnings`, `TransactionLog`, `GoldMilestones`; added 3 fields to `SaveFile`; added `SaveResources2` and `LoadResources2` `SystemParam` bundles; wired them into `save_gather` and `load_apply` |

## What Was Implemented

- **Audit**: Compared all `init_resource` calls against `SaveFile` struct fields.
- **NavigationState**: Determined runtime-only (reset on each `FlightStartEvent`, fully computed from `FlightState`). Not saved — correct.
- **MaintenanceTracker**: Already in `SaveFile` with full serde support. No change needed.
- **TransactionLog**: Was missing serde derives and absent from `SaveFile`. Fixed.
- **GoldMilestones**: Was missing serde derives and absent from `SaveFile`. Fixed.
- **FuelWarnings**: Was missing serde derives and absent from `SaveFile`. Fixed. Includes `total_fuel_burned` aggregate stat and warning threshold flags (reset on refuel).
- **SystemParam overflow handled**: Original `SaveResources`/`LoadResources` had 15 fields (at Bevy's 16-field limit). Created `SaveResources2`/`LoadResources2` with the 3 new resources; both system functions now take both param bundles.

## Quantitative Targets

- Resources audited: all `init_resource` calls across `dlc/pilot/src/`
- Save gaps closed: 3 (`TransactionLog`, `GoldMilestones`, `FuelWarnings`)
- Fields added to `SaveFile`: 3
- New `SystemParam` structs: 2 (`SaveResources2`, `LoadResources2`)

## Validation Results

```
cargo check          ✅ Finished dev profile — 0 errors
cargo test --lib     ✅ 4 passed, 0 failed
cargo clippy -D warnings  ✅ Finished dev profile — 0 warnings
```

## Known Risks

- `FuelWarnings.warned_25/15/05` are flight-session flags that get reset on refuel (`handle_refuel`). Saving them means they are persisted across sessions, but on next load the warnings won't re-trigger for the current fuel level until the threshold is crossed again (harmless — the aircraft will be on the ground when saved).
- `TransactionLog` can grow unbounded over long play sessions. No trimming logic added (out of scope).
