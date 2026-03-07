# UI/UX Fixes — Implementation Notes

## 1. Shop Sell List Refresh After Buy (`src/ui/shop_screen.rs`)

**Location:** `shop_navigation`, inside the `is_buy_mode` + `activate` branch.

**Change:** After a successful buy (`overflow == 0`), added:
```rust
ui_state.sell_items = build_sell_list(&inventory, &item_registry);
```
This ensures the sell list immediately reflects newly purchased items, so switching to SELL mode shows up-to-date inventory quantities without requiring the player to reopen the shop.

## 2. Building Material Check Before Upgrade (`src/ui/building_upgrade_menu.rs`)

**Location:** `building_upgrade_navigation`, in the `activate` branch after the gold check.

**Changes:**
- Added `inventory: Res<Inventory>` parameter to the system.
- After the gold check, iterate over `entry.cost_materials` and call `inventory.has(mat, qty)` for each requirement.
- If any material is missing, build a descriptive message (e.g. `"Not enough materials! Need 200 wood, 100 stone."`) and return early without sending `BuildingUpgradeRequestEvent`.
- The `BuildingUpgradeEvent` is only sent when **both** gold **and** all materials are sufficient.

## Verification

`cargo check` passes for both modified files. Pre-existing errors in unrelated files (`evaluation.rs`, `quests.rs`, `shipping.rs`) are unchanged.
