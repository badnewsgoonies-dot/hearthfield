# Economy Systems Audit

**Date:** 2026-03-03  
**Auditor:** gameplay-systems-auditor  
**Scope:** Shop, blacksmith, building upgrades, gold flow, evaluation

---

## Results Summary

| # | Action | Status | Notes |
|---|--------|--------|-------|
| 1 | Enter Shop | YES | |
| 2 | Buy Item | PARTIAL | Double gold deduction bug |
| 3 | Sell Item | PARTIAL | Double gold addition bug |
| 4 | Exit Shop | YES | |
| 5 | Request Upgrade | YES | |
| 6 | Receive Upgrade | YES | |
| 7 | Tool Locked During Upgrade | NO | Lock not enforced in tool_use() |
| 8 | Upgrade Coop | YES | |
| 9 | Upgrade Barn | YES | |
| 10 | Upgrade House | YES | |
| 11 | Day-End Gold | PARTIAL | `ToastEvent` missing `duration_secs` field (won't compile) |
| 12 | Gold Display | YES | |
| 13 | Evaluation | PARTIAL | No dedicated UI screen; result shown via toast only |

---

## Detailed Traces

### 1. ENTER SHOP — YES

```
Player steps on TransitionZone tile
  → world/mod.rs:check_transitions fires MapTransitionEvent { to_map: GeneralStore/AnimalShop/Blacksmith }
  → world/mod.rs:handle_map_transition despawns old map, loads new map tiles + objects
  → economy/shop.rs:on_enter_shop reads MapTransitionEvent
      → next_state.set(GameState::Shop)
      → ActiveShop populated with season-filtered listings
  → ui/mod.rs: on_enter(GameState::Shop) spawns ShopScreenRoot
```

All links intact. Interior map loads via WorldPlugin; shop state activates via EconomyPlugin.

---

### 2. BUY ITEM — PARTIAL

```
shop_screen.rs:shop_navigation [activate]:
  → gold check: player.gold >= listing.price ✓
  → inventory.try_add(item_id, 1, max_stack) ✓
  → player.gold -= listing.price  ← DIRECT MUTATION #1
  → ShopTransactionEvent { is_purchase: true, total_cost: price }

economy/shop.rs:handle_shop_transaction_gold:
  → reads ShopTransactionEvent
  → GoldChangeEvent { amount: -(total_cost) }  ← fires negative gold event

economy/gold.rs:apply_gold_changes:
  → reads GoldChangeEvent
  → player_state.gold -= cost  ← DIRECT MUTATION #2 (double deduction)
```

**Bug:** `shop_screen.rs` directly subtracts gold (`player.gold -= listing.price`, line 593), then fires `ShopTransactionEvent`. `handle_shop_transaction_gold` converts that to a `GoldChangeEvent`, which `apply_gold_changes` also applies to `player_state.gold`. Gold is deducted **twice** per purchase.

The comment in `economy/shop.rs` acknowledges this intent: *"The shop UI already mutates player.gold directly; this fires the event alongside that mutation purely for stats tracking."* But `apply_gold_changes` is not stats-only — it mutates gold.

**Fix:** Either (a) remove direct gold mutation from `shop_screen.rs` and let `GoldChangeEvent` be the sole mutator, or (b) add a separate `ShopStatsEvent` for stats tracking that does not go through `apply_gold_changes`.

---

### 3. SELL ITEM — PARTIAL

```
shop_screen.rs:shop_navigation [sell]:
  → inventory.try_remove(item_id, 1) ✓
  → player.gold += price  ← DIRECT MUTATION #1
  → ShopTransactionEvent { is_purchase: false, total_cost: price }

economy/shop.rs:handle_shop_transaction_gold:
  → GoldChangeEvent { amount: +total_cost }

economy/gold.rs:apply_gold_changes:
  → player_state.gold += gain  ← DIRECT MUTATION #2 (double addition)
```

Same double-mutation bug as buy. Gold is added twice per sale.

---

### 4. EXIT SHOP — YES

```
player_input.ui_cancel (Escape)
  → economy/shop.rs:on_exit_shop
      → next_state.set(GameState::Playing)
      → active_shop.shop_id = None; listings cleared
  → ui/mod.rs: on_exit(GameState::Shop) despawns ShopScreenRoot
```

Full chain intact.

---

### 5. REQUEST UPGRADE — YES

```
ShopUiState.upgrade_mode + player activates
  → shop_screen.rs:shop_navigation fires ToolUpgradeRequestEvent { tool }

economy/blacksmith.rs:handle_upgrade_request:
  → guard: active_shop.shop_id == Some(Blacksmith) ✓
  → checks: not already upgrading, has next tier, gold >= cost, inventory has bars ✓
  → GoldChangeEvent { amount: -gold_cost }  → apply_gold_changes deducts gold ✓
  → inventory.try_remove(bar_id, bar_qty) ✓
  → ItemRemovedEvent fired ✓
  → upgrade_queue.pending.push(PendingUpgrade { days_remaining: 2 }) ✓
  → PlaySfxEvent("blacksmith_forge") ✓
```

Full chain intact.

---

### 6. RECEIVE UPGRADE — YES

```
Player sleeps → CalendarPlugin fires DayEndEvent

economy/blacksmith.rs:tick_upgrade_queue:
  → reads DayEndEvent
  → for each pending: days_remaining.saturating_sub(1)
  → when days_remaining == 0:
      → upgrade_queue.pending.retain(|p| p.tool != tool) ✓
      → player_state.tools.insert(tool, new_tier)  ← tier increased ✓
      → ToolUpgradeCompleteEvent { tool, new_tier } ✓
      → PlaySfxEvent("upgrade_complete") ✓
      → ToastEvent("Your {:?} upgrade is complete — ready to use!") ✓
```

Full chain intact. Toast fires with 4.0s duration.

---

### 7. TOOL LOCKED DURING UPGRADE — NO

`ToolUpgradeQueue.is_upgrading()` exists and is used in two places:
- `blacksmith.rs:handle_upgrade_request` — prevents queueing the same tool twice ✓
- `shop_screen.rs:build_upgrade_entries` — renders `[IN PROGRESS]` status in UI ✓

However, `player/tools.rs:tool_use()` does **not** consult `ToolUpgradeQueue`. The system only checks `input_blocks.is_blocked()`, stamina, and cooldown. A tool that is currently at the blacksmith being upgraded can still be used by the player in the field.

**Fix:** Pass `ToolUpgradeQueue` into `tool_use()` and early-return if `upgrade_queue.is_upgrading(player_state.equipped_tool)`.

---

### 8. UPGRADE COOP — YES

```
ui/building_upgrade_menu.rs:building_upgrade_navigation [confirm]:
  → BuildingUpgradeEvent { building: Coop, to_tier }

economy/buildings.rs:handle_building_upgrade_request:
  → checks no upgrade in progress, gold, materials ✓
  → GoldChangeEvent (gold deducted) ✓
  → inventory.try_remove per material ✓
  → building_levels.upgrade_in_progress = Some((Coop, to_tier, 2)) ✓
  → ToastEvent("Upgrade started! Come back in 2 days.") ✓

economy/buildings.rs:tick_building_upgrade (on DayEndEvent, day 2):
  → animal_state.has_coop = true ✓
  → animal_state.coop_level = 1/2/3 per tier ✓
  → building_levels.coop_tier = target_tier ✓
  → building_levels.upgrade_in_progress = None ✓
  → ToastEvent("{:?} upgraded to {:?}!") ✓
```

Full chain intact.

---

### 9. UPGRADE BARN — YES

Identical flow to Coop. `tick_building_upgrade` sets `animal_state.has_barn`, `animal_state.barn_level`, and `building_levels.barn_tier`. Full chain intact.

---

### 10. UPGRADE HOUSE — YES

`tick_building_upgrade` handles `BuildingKind::House`:
- Big tier: `house_state.tier = HouseTier::Big`, `house_state.has_kitchen = true` ✓
- Deluxe tier: `house_state.tier = HouseTier::Deluxe`, `has_kitchen`, `has_nursery = true` ✓

Note: `BuildingLevels` resource does not track `house_tier` explicitly (only `coop_tier`, `barn_tier`, `silo_built`); house tier lives in `HouseState`. This is architecturally inconsistent but functionally correct.

---

### 11. DAY-END GOLD — PARTIAL

```
Player sleeps → DayEndEvent

economy/shipping.rs:process_shipping_bin_on_day_end:
  → iterates shipping_bin.items, calculates total_value ✓
  → GoldChangeEvent { amount: total_value }  → apply_gold_changes adds gold ✓
  → stats.total_items_shipped updated ✓
  → shipping_log.shipped_items updated ✓
  → PlaySfxEvent("day_end_coins") ✓
  → ToastEvent { message: "Shipping: earned Xg...", }  ← MISSING duration_secs FIELD
  → shipping_bin.items.clear() ✓
```

**Bug:** `ToastEvent` at `shipping.rs:151` is constructed without `duration_secs`, but `ToastEvent` in `shared/mod.rs:1079` defines it as a required field. This is a compile error that would prevent the game from building.

Compare with valid usages (e.g., `blacksmith.rs:tick_upgrade_queue`, `buildings.rs`) which all supply `duration_secs`.

**Fix:**
```rust
toast_writer.send(ToastEvent {
    message: format!("Shipping: earned {}g from {} items", total_value, items_shipped),
    duration_secs: 4.0,
});
```

---

### 12. GOLD DISPLAY — YES

```
Any gold mutation (shop buy/sell, shipping, blacksmith) → player_state.gold changes
  → PlayerState marked changed (Bevy change detection)

ui/hud.rs:update_gold_display:
  → runs every frame while HUD is active
  → player.is_changed() guard ✓
  → writes format!("{} G", player.gold) to HudGoldText entity ✓
```

Display updates correctly on every `PlayerState` mutation. Note: due to the double-gold-mutation bug in shop (items 2/3), the displayed balance will reflect the incorrect (twice-applied) delta.

---

### 13. EVALUATION — PARTIAL

```
calendar.year >= 3, season == Spring, day == 1
  → economy/evaluation.rs:check_evaluation_trigger fires EvaluationTriggerEvent

economy/evaluation.rs:handle_evaluation:
  → scores 8 categories, 21 total points ✓
  → points_to_candles() → 1–4 candles ✓
  → EvaluationScore resource updated (total_points, categories, candles_lit, evaluated) ✓
  → ToastEvent with score summary ✓
```

**Gap:** There is no dedicated evaluation UI screen. The candle result is communicated only via toast notification (4–6 seconds). `EvaluationScore` is populated in a resource and could be read by a UI, but no shrine cutscene, dedicated overlay, or persistent score screen is wired up. The `EvaluationTriggerEvent` → `handle_evaluation` logic itself is fully functional.

---

## Critical Issues

| Priority | Issue | Location |
|----------|-------|----------|
| 🔴 Compile Error | `ToastEvent` missing `duration_secs` in shipping bin | `economy/shipping.rs:151` |
| 🟠 Logic Bug | Double gold deduction on shop buy | `economy/shop.rs` + `ui/shop_screen.rs:593` |
| 🟠 Logic Bug | Double gold addition on shop sell | `economy/shop.rs` + `ui/shop_screen.rs:612` |
| 🟡 Missing Feature | Tool not locked while upgrading at blacksmith | `player/tools.rs:tool_use()` |
| 🟡 Missing Feature | No evaluation UI screen; candles only shown via toast | `ui/` — no evaluation screen |
