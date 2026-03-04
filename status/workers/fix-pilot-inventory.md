# Worker Report: PILOT INVENTORY USE/EQUIP

## Files Modified
- `dlc/pilot/src/ui/inventory_screen.rs` (99 -> ~310 lines)

## What Was Implemented
1. **SelectedSlot resource** — `SelectedSlot(Option<usize>)` tracks current selection; inserted on spawn, removed on despawn.
2. **Visual highlight** — Selected slot gets a gold border (`BorderColor`); unselected slots have transparent borders.
3. **Click handler** (`handle_slot_click`) — Clicking a slot via Bevy `Interaction::Pressed` sets `SelectedSlot`, with hover/none color feedback.
4. **Keyboard navigation** (`handle_inventory_keyboard`) — Arrow keys navigate a 5-column grid, Enter/Space fires `UseItemEvent`, Esc transitions to `GameState::Playing`.
5. **Item detail panel** — Right-side panel showing item name, description, and quantity. Uses `DetailField` enum component to tag text entities (Name, Description, Quantity, ButtonLabel).
6. **Use/Equip button** (`handle_use_equip_button`) — Button hidden until an item is selected. Shows "Equip" for Tool/Part/Cosmetic categories, "Use" otherwise. Fires `UseItemEvent { slot }` on press, with hover color feedback following the `ShopBuyButton` pattern.
7. **Selection visual updater** (`update_selection_visuals`) — Runs when `SelectedSlot` changes; updates border highlights, detail panel text, and button visibility/label in one system.

## Shared Type Imports Used
- `GameState` (for Esc -> Playing transition)
- `Inventory`, `InventorySlot` (reading slot data)
- `ItemRegistry`, `ItemDef`, `ItemCategory` (looking up item info, determining equippable)
- `UiFontHandle` (font resource)

## New Types Defined (local to file)
- `SelectedSlot` (Resource)
- `InventorySlotButton(usize)` (Component)
- `DetailField` enum (Component) — Name, Description, Quantity, ButtonLabel
- `UseEquipButton` (Component)
- `UseItemEvent { slot: usize }` (Event)

## Public Systems Exported (for wiring in ui/mod.rs)
- `spawn_inventory_screen` (OnEnter)
- `despawn_inventory_screen` (OnExit)
- `handle_slot_click` (Update)
- `update_selection_visuals` (Update)
- `handle_use_equip_button` (Update)
- `handle_inventory_keyboard` (Update)

## Validation Results
- `cargo check` — PASS (zero errors)
- `cargo test --test headless` — PASS (76/76 tests)
- `cargo clippy -- -D warnings` — PASS (zero warnings)

## Known Risks for Integration
- The other worker modifying `ui/mod.rs` needs to register the four new Update systems (`handle_slot_click`, `update_selection_visuals`, `handle_use_equip_button`, `handle_inventory_keyboard`) under `.run_if(in_state(GameState::Inventory))`.
- The other worker modifying `shared/mod.rs` needs to register `UseItemEvent` as an event via `app.add_event::<inventory_screen::UseItemEvent>()` or move the event definition to shared.
- No consumer for `UseItemEvent` exists yet; a gameplay system needs to handle it.
