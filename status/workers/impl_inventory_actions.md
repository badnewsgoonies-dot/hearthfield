# Inventory Actions Implementation

**File modified:** `src/ui/inventory_screen.rs` only

## Changes

### 1. Item Use (Enter/Return)
Extended `inventory_navigation` to handle `action.activate` (Enter key, mapped in `shared/mod.rs`):
- **Food** (`ItemCategory::Food`): sends `EatFoodEvent { item_id, stamina_restore, buff: None }` using `ItemDef.energy_restore`
- **Tool** (`ItemCategory::Tool`): sets `player_state.equipped_tool` via `tool_kind_from_item_id()` mapping; falls back to toast if item_id is unrecognized
- **Other**: sends `ToastEvent { message: "Cannot use this item.", duration_secs: 2.0 }`

No `EquipToolEvent` exists in `shared/mod.rs`; tool equipping is done by directly setting `PlayerState.equipped_tool`.

Note: No items with `ItemCategory::Tool` exist in `data/items.rs` — tools live in `PlayerState.tools` separately — so the tool branch is correct-but-inert for current data.

### 2. Item Description (hover info)
- Added `InventoryDescText` marker component
- Spawned a description `Text` node at the bottom of the inventory panel (panel height increased 320→360 px)
- `update_inventory_slots` now:
  - Accepts `ui_state: Option<Res<InventoryUiState>>` and `desc_query: Query<&mut Text, With<InventoryDescText>>`
  - Adds `Without<InventoryDescText>` to item-name and quantity queries to keep all three `&mut Text` queries disjoint
  - Writes `ItemDef.description` of the hovered slot to the description node each frame

### 3. Hint text updated
`"WASD/Arrows: Move | Esc: Close"` → `"WASD/Arrows: Move | Enter: Use | Esc: Close"`

## No changes to `src/ui/mod.rs`
All new behaviour is woven into the three already-registered systems
(`update_inventory_slots`, `update_inventory_cursor`, `inventory_navigation`).
