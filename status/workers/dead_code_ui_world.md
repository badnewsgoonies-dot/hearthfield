# Dead Code Annotation Audit (UI + World)

Scope: audited all `#[allow(dead_code)]` annotations in the requested files using `grep -rn 'item_name' src/ --include='*.rs'`.

## Removed annotations (item is used outside its own definition/impl block)

- `src/ui/building_upgrade_menu.rs`: `BuildingRowText`
  - External uses found in same module outside struct definition (spawn/query usage).
- `src/ui/building_upgrade_menu.rs`: `BuildingRowCost`
  - External uses found in same module outside struct definition (spawn/query usage).
- `src/ui/crafting_screen.rs`: `CraftingRecipeIcon`
  - External use found in recipe row spawn code.
- `src/ui/dialogue_box.rs`: `DialogueUiState`
  - External uses found, including `src/ui/cutscene_runner.rs`.
- `src/ui/inventory_screen.rs`: `InventoryGridSlot`
  - External use found in inventory slot spawn code.
- `src/ui/tutorial.rs`: `is_crop_ready`
  - External uses found in tutorial tests.
- `src/world/maps.rs`: `MapDef`
  - External uses found (`src/world/mod.rs` import/field/signature usage).
- `src/world/mod.rs`: `TransitionZone`
  - External use found in transition zone spawn code.

## Kept annotations (item not used outside its own definition)

- `src/ui/hud.rs`: `HotbarSlotBackground`
- `src/ui/menu_kit.rs`: `BTN_PRESSED`
- `src/ui/menu_kit.rs`: `BTN_DISABLED`
- `src/world/maps.rs`: `MapDef::set_tile`
- `src/world/mod.rs`: `WorldMap::is_water`
- `src/world/mod.rs`: `WorldMap::get_tile`
- `src/world/mod.rs`: `WorldMap::transitions`
- `src/world/objects.rs`: `WorldObjectKind::is_solid`

## Notes

- No code was deleted.
- Only unnecessary `#[allow(dead_code)]` annotations were removed.
