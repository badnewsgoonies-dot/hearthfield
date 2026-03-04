# Worker: PILOT INVENTORY USE/EQUIP

## Context
The pilot DLC inventory screen at `dlc/pilot/src/ui/inventory_screen.rs` is display-only (99 lines). It renders item slots but has zero interaction handlers. You need to add:
1. Item selection (click or keyboard)
2. Use/Equip button that fires appropriate events
3. Item detail panel showing description

## Required reading
1. `dlc/pilot/src/ui/inventory_screen.rs` — current read-only display
2. `dlc/pilot/src/shared/mod.rs` — find `Inventory`, `ItemDef`, `ItemId`, any use/equip events
3. `dlc/pilot/src/ui/shop_screen.rs` — reference for button interaction pattern (ShopBuyButton, handle_shop_buy)
4. `dlc/pilot/src/ui/mod.rs` — see how Inventory state is wired

## Deliverables (all in `dlc/pilot/src/ui/inventory_screen.rs`)
1. `SelectedSlot(usize)` resource or local to track which slot is selected
2. Visual highlight on selected slot (border color change)
3. Click handler: clicking a slot selects it
4. Keyboard: arrow keys to navigate slots, Enter/Space to use, Esc to close
5. Item detail panel on the right showing: item name, description, quantity
6. "Use" button that fires a `UseItemEvent { slot: usize }` (define this event in the same file or use one from shared if it exists)
7. If the item is equipment/tool, show "Equip" instead of "Use"

## Pattern to follow
Look at how `shop_screen.rs` handles `ShopBuyButton` with `Interaction` component changes. Use the same Bevy UI interaction pattern.

## Validation
```bash
cd /home/user/hearthfield/dlc/pilot && cargo check 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo test --test headless 2>&1
cd /home/user/hearthfield/dlc/pilot && cargo clippy -- -D warnings 2>&1
```
Done = all three pass with zero errors/warnings.

## When done
Write completion report to `/home/user/hearthfield/status/workers/fix-pilot-inventory.md`
