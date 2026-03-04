# Worker: FIX-MAIN-SAVE-GAPS (Missing Resources in Main Game Save)

## Scope
You may modify files under: src/economy/, src/calendar/, src/npcs/, src/crafting/, src/save/

## Required reading
1. src/save/mod.rs (FULL file — FullSaveFile, ExtendedResources, ExtendedResourcesMut, write_save, handle_load_request, handle_new_game)
2. src/economy/blacksmith.rs (search for ToolUpgradeQueue struct)
3. src/economy/shipping.rs (search for ShippingBinQuality struct)
4. src/calendar/festivals.rs (search for FestivalState struct)
5. src/npcs/mod.rs (search for FarmVisitTracker struct)
6. src/crafting/machines.rs (search for ProcessingMachineRegistry struct)

## Bug: 5 Game State Resources Missing from Save File

### Root Cause
These resources are initialized but not included in FullSaveFile:
1. **ToolUpgradeQueue** — pending tool upgrades lost on save/load
2. **ShippingBinQuality** — quality tracking lost
3. **FestivalState** — festival progress lost
4. **FarmVisitTracker** — NPC farm visit state lost (daily repeats)
5. **ProcessingMachineRegistry** — placed processing machines lost

### Fix Pattern (for each resource)
1. Check if it has `Serialize, Deserialize` derives — add if missing
2. Add `#[serde(default)] pub field_name: Type` to FullSaveFile
3. Add to ExtendedResources: `pub field_name: Res<'w, Type>`
4. Add to ExtendedResourcesMut: `pub field_name: ResMut<'w, Type>`
5. Add to write_save: `field_name: ext.field_name.clone()`
6. Add to handle_load_request: `*ext.field_name = file.field_name`
7. Add to handle_new_game: `*ext.field_name = Default::default()`

NOTE: If ExtendedResources/ExtendedResourcesMut already have 16 fields (Bevy's SystemParam limit), you may need to create a second bundle (ExtendedResources2/ExtendedResourcesMut2).

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-main-save-gaps.md
