# Worker Report: FIX-MAIN-SAVE-GAPS

## Status: COMPLETE

## Files Modified
- `src/economy/blacksmith.rs` — Added `Serialize, Deserialize` to `ToolUpgradeQueue` and `PendingUpgrade`
- `src/economy/shipping.rs` — Added `Serialize, Deserialize` to `ShippingBinQuality`
- `src/calendar/festivals.rs` — Added `Serialize, Deserialize` to `FestivalKind` and `FestivalState`; `#[serde(skip)]` on `timer` (Bevy `Timer` not serializable)
- `src/npcs/schedules.rs` — Added `Serialize, Deserialize` to `FarmVisitTracker`
- `src/crafting/machines.rs` — Added `Serialize, Deserialize` to `MachineType`; created `SavedMachine` struct; made `machine_atlas_index` pub
- `src/crafting/mod.rs` — Exported `SavedMachine` and `machine_atlas_index`
- `src/save/mod.rs` — Full integration of 5 resources + placed machines

## What Was Implemented

### Resources added to save/load/new-game cycle:
1. **ToolUpgradeQueue** — via `ExtendedResources`/`ExtendedResourcesMut`
2. **ShippingBinQuality** — via `ExtendedResources`/`ExtendedResourcesMut` (removed duplicate `ResMut` param from `handle_new_game`)
3. **FestivalState** — via `ExtendedResources`/`ExtendedResourcesMut`; `timer` field skipped (runtime-only)
4. **FarmVisitTracker** — via `ExtendedResources`/`ExtendedResourcesMut`

### ProcessingMachineRegistry (chest pattern):
- `SavedMachine { grid_x, grid_y, machine_type, input_item, output_item, processing_time_remaining, is_ready }` added to `FullSaveFile.placed_machines`
- `handle_save_request` collects machines via `Query<(&ProcessingMachine, &GridPosition)>`
- `MachineLoadResources` SystemParam bundle added for load-time machine restoration
- `handle_load_request` despawns existing machine entities, spawns saved ones, rebuilds registry
- `handle_new_game` despawns machines and clears registry

### FullSaveFile new fields (all `#[serde(default)]`):
- `tool_upgrade_queue: ToolUpgradeQueue`
- `shipping_bin_quality: ShippingBinQuality`
- `festival_state: FestivalState`
- `farm_visit_tracker: FarmVisitTracker`
- `placed_machines: Vec<SavedMachine>`

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (88/88)
- `cargo clippy -- -D warnings` — PASS
- `shasum -a 256 -c .contract.sha256` — OK (src/shared/mod.rs unchanged)

## Known Risks
- `FestivalState.timer` is not serialized. After a load during an active festival, the timer resets to `None`. Festival state (active/score/items_collected) is preserved; the gameplay timer must restart.
- `ProcessingMachineRegistry` entity handles are not serialized (not possible with Bevy `Entity`). The registry is rebuilt from saved machine positions on load. This means runtime entity handles change across save/load, but all callers look up entities from the registry by position so they will get the new handles.
