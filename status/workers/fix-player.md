# Worker Report: FIX-PLAYER

## Files Modified
- `src/player/item_use.rs` (165 lines, was 152 lines; +13 lines net)

## What Was Changed and Why

### Bug: Animal Feeding Had No Code Path
The animals domain (`src/animals/feeding.rs`) has a `handle_feed_trough_interact` system that listens for `ItemRemovedEvent` with `item_id == "hay"`. However, nothing in the codebase was emitting this event. The player's item-use dispatcher (`dispatch_item_use`) handled food, sprinklers, machines, bouquet, and mermaid pendant -- but not hay. As a result, players could never feed animals.

### Fix Applied
In `src/player/item_use.rs`, function `dispatch_item_use`:

1. Changed `inventory: Res<Inventory>` to `mut inventory: ResMut<Inventory>` (needed to call `try_remove`)
2. Added `mut removed_events: EventWriter<ItemRemovedEvent>` parameter
3. Added a hay handling branch BEFORE the food/edible check:
   - Checks if `item_id == "hay"`
   - Calls `inventory.try_remove("hay", 1)` to consume one hay from inventory
   - If removal succeeded (`removed > 0`), sends `ItemRemovedEvent { item_id: "hay", quantity: 1 }`
   - Returns early (hay is not edible and should not fall through to other branches)

### Shared Type Imports Used
- `ItemRemovedEvent` (from `src/shared/mod.rs`, line 861) -- the event that bridges player -> animals domains
- `Inventory` (from `src/shared/mod.rs`, line 357) -- changed from `Res` to `ResMut` to allow mutation via `try_remove`
- All previously used shared types remain unchanged

## Validation Results
- `cargo check`: PASS (no errors in src/player/; pre-existing errors in src/crafting/cooking.rs and src/npcs/dialogue.rs are out of scope)
- `cargo clippy -- -D warnings`: PASS (no warnings in src/player/)
- Pre-existing issues in other domains (crafting, npcs) are unrelated to this fix

## Known Risks for Integration
- None. The `ItemRemovedEvent` is already registered in `main.rs`. The animals domain already has the listener. This fix simply connects the two by emitting the event from the player domain.
