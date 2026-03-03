# Gameplay Systems Audit — End-to-End Trace
**Date:** 2026-03-03  
**Auditor:** Copilot (automated trace)  
**Scope:** Animals (1–6), Cooking (7–10), Crafting (11–14), Storage (15–18)

---

## Summary Table

| # | Action | Status | Chain Traced | Issues Found |
|---|--------|--------|--------------|--------------|
| 1 | BUY ANIMAL | ✅ YES | `ShopTransactionEvent` → `handle_animal_purchase` (spawning.rs) → building check → cap check → `commands.spawn(Animal + WanderAi + name tag)` | None |
| 2 | FEED ANIMAL | ❌ NO | `ItemRemovedEvent{item_id:"hay"}` → `handle_feed_trough_interact` → `animal.fed_today = true` | **Event never fired.** No code in player domain emits `ItemRemovedEvent` for hay near the trough. `item_use.rs` has no hay branch. `player/tools.rs` has no hay branch. The consumer exists; the producer does not. |
| 3 | PET ANIMAL | ✅ YES | `PlayerInput.tool_use` → `handle_animal_interact` (proximity ≤32px, no product_ready) → `animal.petted_today = true`, `happiness += 5` → floating text + `PlaySfxEvent` | None |
| 4 | COLLECT PRODUCT | ✅ YES | `PlayerInput.tool_use` → `handle_product_collection` (priority over petting) → reads `PendingProductQuality` → clears `product_ready` → `ItemPickupEvent` + quality toast + floating text | None |
| 5 | ANIMAL DAY-END | ✅ YES | `DayEndEvent` → `handle_day_end_for_animals` → unfed days tracked, happiness ±(5/12/+5), flags reset, baby aging at 5 days, `product_ready` + `PendingProductQuality` set per species cadence | None |
| 6 | HOUSING CAP | ✅ YES | `ShopTransactionEvent` → `handle_animal_purchase` → `coop_level * 4` / `barn_level * 4` vs current count → `ToastEvent("Your coop/barn is full!")` + early return | None |
| 7 | ACCESS KITCHEN | ⚠️ PARTIAL | Player in `PlayerHouse` → F key → `dispatch_world_interaction` finds `KitchenStove` → `OpenCraftingEvent{cooking_mode:true}` → `handle_open_crafting` → `GameState::Crafting` | `KitchenStove` is spawned for **all** `PlayerHouse` entries unconditionally — `has_kitchen` flag (set on Big/Deluxe house upgrade) is never checked. Cooking is available from day 1 regardless of house upgrade. |
| 8 | SELECT RECIPE | ✅ YES | `handle_open_crafting` filters `UnlockedRecipes` by `is_cooking == cooking_mode`, sorts alphabetically → `CraftingUiState.available_recipes` → crafting_screen.rs renders list | None |
| 9 | COOK | ⚠️ PARTIAL | `CraftItemEvent` → `handle_cook_item` → unlocked check → ingredient check (wildcard `any_fish` resolved) → `consume_non_wildcard_ingredients` + fish wildcard consumed → `inventory.try_add` → `ItemPickupEvent` | **Stamina restored at cook time, not eat time.** `cooking.rs` reads `item_def.energy_restore` and writes `player_state.stamina` immediately when the food is crafted — before the player eats it. This double-dips if the player later eats the item via `dispatch_item_use` → `EatFoodEvent`. Buffs are **not** applied by cooking; only the stamina restore fires early. |
| 10 | EAT FOOD | ✅ YES | R key → `dispatch_item_use` → `def.edible` → `EatFoodEvent{stamina_restore, buff: food_buff_for_item()}` → `handle_eat_food` → removes 1, restores stamina (capped), applies/replaces buff in `ActiveBuffs`, toast + SFX | None |
| 11 | OPEN CRAFTING | ✅ YES | C key → `PlayerInput.open_crafting` → `trigger_crafting_key` → `OpenCraftingEvent{cooking_mode:false}` → `handle_open_crafting` → `GameState::Crafting` | None |
| 12 | CRAFT ITEM | ✅ YES | `CraftItemEvent` → `handle_craft_item` → unlocked check → all ingredients present → `consume_ingredients` → `inventory.try_add` (refund on full) → `ItemPickupEvent` + SFX | None |
| 13 | USE MACHINE | ✅ YES | R key → `dispatch_item_use` → `PlaceMachineEvent` → `handle_place_machine` spawns `ProcessingMachine`; F key near machine → `InsertMachineInputEvent` / `CollectMachineOutputEvent` → `handle_insert_machine_input` / `handle_collect_machine_output`; `tick_processing_machines` advances timers each frame | None |
| 14 | RECIPE UNLOCK | ✅ YES | Three paths all feed `UnlockRecipeEvent` → `handle_unlock_recipe`: (a) `initialize_unlocked_recipes` at startup for defaults; (b) `check_milestone_recipe_unlocks` via `ItemPickupEvent`; (c) `check_friendship_recipe_unlocks` on `Relationships.is_changed()` | None |
| 15 | OPEN CHEST | ✅ YES | F key → `interact_with_chest` checks `interaction_claimed`, proximity (≤2 tiles) → `ChestInteraction.entity = Some(e)` → `update_chest_ui_lifecycle` spawns split-panel UI | None |
| 16 | DEPOSIT | ✅ YES | Enter on inventory panel → `handle_chest_input` → `transfer_from_inventory_to_chest` → stacks onto matching slots, then first empty slot; returns remainder to inventory if chest full | None |
| 17 | WITHDRAW | ✅ YES | Tab to chest panel → Enter → `handle_chest_input` → `transfer_from_chest_to_inventory` → `inventory.try_add`; excess stays in chest if inventory full | None |
| 18 | CLOSE CHEST | ✅ YES | Esc → `PlayerInput.ui_cancel` → `close_chest_on_escape` → `ChestInteraction.entity = None` → `update_chest_ui_lifecycle` despawns UI, removes `ChestUiState` | None |

---

## Detailed Issue Reports

### Issue A — FEED ANIMAL (Action 2): Missing event producer ❌

**Severity:** Critical (feature completely broken)

**Chain gap:**  
`feeding.rs::handle_feed_trough_interact` listens for `ItemRemovedEvent { item_id: "hay" }`, but no code path produces this event for the hay-at-trough interaction:

- `player/item_use.rs` (`dispatch_item_use`, R key): handles edible items, sprinklers, machines, bouquets — **no hay branch**
- `player/tools.rs`: handles tool swing actions — **no hay branch**
- `player/interaction.rs`: **no hay branch**

`ItemRemovedEvent` for "hay" is only emitted in: `blacksmith.rs` (tool upgrades), `farming/crops.rs` (planting), `fishing/cast.rs` (bait), `npcs/gifts.rs` (gifting). None of these relate to feeding animals.

**Fix required:** Add a hay-use branch in `dispatch_item_use` (or a dedicated system) that:
1. Checks player is near the feed trough
2. Removes 1 hay from inventory via `inventory.try_remove("hay", 1)`
3. Emits `ItemRemovedEvent { item_id: "hay".into(), quantity: 1 }`

---

### Issue B — ACCESS KITCHEN (Action 7): `has_kitchen` gate missing ⚠️

**Severity:** Moderate (balance issue — upgrade incentive bypassed)

`world/objects.rs` always spawns the `KitchenStove` interactable when loading `PlayerHouse`, without checking `player_state.has_kitchen` (set to `true` only on Big/Deluxe house upgrades in `economy/buildings.rs:183,187`).

**Fix required:** Wrap the `KitchenStove` spawn in:
```rust
if player_state.has_kitchen {
    // spawn KitchenStove ...
}
```

---

### Issue C — COOK (Action 9): Stamina restored at cook time, not eat time ⚠️

**Severity:** Minor (gameplay balance / double-dip risk)

`crafting/cooking.rs::handle_cook_item` reads `item_def.energy_restore` and adds it directly to `player_state.stamina` after a successful cook. The player then has the cooked food item in their inventory. If they later eat it via R key → `dispatch_item_use` → `EatFoodEvent` → `handle_eat_food`, stamina is restored **a second time**.

**Fix required:** Remove the inline stamina-restore block from `handle_cook_item` (~lines 108–119 in cooking.rs). Stamina recovery should only happen via the `EatFoodEvent` path when the food is consumed.

---

## Stats

| Status | Count |
|--------|-------|
| ✅ YES | 14 |
| ⚠️ PARTIAL | 2 |
| ❌ NO | 1 |
| **Total** | **18** |
