# Economy Domain Audit Report

**Audited:** `src/economy/` (11 files, ~1050 lines)  
**Date:** 2026-03-02  
**Auditor:** Code Scout

---

## 1. Public Functions by Module

### `gold.rs`

| Function | Signature | Purpose |
|---|---|---|
| `apply_gold_changes` | `System` | Reads `GoldChangeEvent` queue; adds positive amounts to `PlayerState.gold`; subtracts negative amounts (clamped to 0). Updates `EconomyStats` counters. Runs in both `Playing` and `Shop` states. |
| `format_gold` | `(u32) -> String` | Formats a gold amount with comma-separators and a trailing `g` (e.g. `1,234g`). Used by UI. |

**Resource defined:** `EconomyStats { total_gold_earned, total_gold_spent, total_items_shipped, total_transactions }`

---

### `shipping.rs`

| Function | Signature | Purpose |
|---|---|---|
| `place_in_shipping_bin` | `System` | Reads `ShipItemEvent`; validates item exists in registry and player has enough in inventory; removes from `Inventory`; merges into `ShippingBin`. Plays SFX. |
| `process_shipping_bin_on_day_end` | `System` | Reads `DayEndEvent`; iterates the bin, looks up `sell_price` from `ItemRegistry`, accumulates total; fires `GoldChangeEvent` for the total; clears the bin; sends toast and SFX. This is the primary income path. |
| `calculate_bin_value` | `(&ShippingBin, &ItemRegistry) -> u32` | Pure helper; returns the current estimated sell value of bin contents. Falls back to 1g for unregistered items. |
| `update_shipping_bin_preview` | `System` | Re-computes `ShippingBinPreview.{pending_value, item_count}` whenever `ShippingBin` or `ItemRegistry` changes. Called every frame for HUD display. |

**Resources defined:** `ShippingBinPreview { pending_value, item_count }`, `ShipItemEvent`

---

### `shop.rs`

| Function | Signature | Purpose |
|---|---|---|
| `on_enter_shop` | `System` | Watches `MapTransitionEvent`; when the player enters `GeneralStore`, `AnimalShop`, or `Blacksmith` maps, transitions to `GameState::Shop` and populates `ActiveShop` with season-filtered `ActiveListing`s. |
| `refresh_shop_affordability` | `System` | Re-checks `can_afford` on every `ActiveListing` each frame against current `PlayerState.gold`. |
| `on_exit_shop` | `System` | Returns to `GameState::Playing` and clears `ActiveShop` when the player presses UI-cancel (Escape). |

**Resource defined:** `ActiveShop { shop_id, listings: Vec<ActiveListing> }`  
**Helper (private):** `build_listings` — filters shop inventory by season, enriches with item data from registry.

---

### `blacksmith.rs`

| Function | Signature | Purpose |
|---|---|---|
| `handle_upgrade_request` | `System` | Validates `ToolUpgradeRequestEvent`; checks: correct shop, tool not already upgrading, tool not already max tier, sufficient gold, required bars in inventory. On success: deducts gold and bars, pushes `PendingUpgrade` with 2-day timer. |
| `tick_upgrade_queue` | `System` | Reads `DayEndEvent`; decrements all pending upgrade timers; applies completed upgrades to `PlayerState.tools`; fires `ToolUpgradeCompleteEvent` + toast + SFX. |
| `drain_upgrade_complete` | `System` | Silently drains `ToolUpgradeCompleteEvent` to suppress Bevy "event not read" warnings. |

**Resources defined:** `ToolUpgradeQueue { pending: Vec<PendingUpgrade> }`  
**Events defined:** `ToolUpgradeRequestEvent`, `ToolUpgradeCompleteEvent`  
**Public type alias:** `UpgradeEntry` — a 9-tuple for blacksmith UI display (tool, current/target tier, cost, bar id/qty, can_afford, has_bars, is_upgrading). No function returns this type.

**`ToolUpgradeQueue::is_upgrading(tool) -> bool`** — public method to query in-progress upgrades.

---

### `buildings.rs`

| Function | Signature | Purpose |
|---|---|---|
| `upgrade_cost` | `(BuildingKind, BuildingTier) -> (u32, Vec<(&str, u8)>)` | **Public.** Single source of truth for all building upgrade costs. Returns `(gold, materials)`. Returns `(0, [])` for invalid combos. |
| `handle_building_upgrade_request` | `System` | Reads `BuildingUpgradeEvent`; validates no upgrade in progress, enough gold, enough materials; deducts gold and materials; starts 2-day timer in `BuildingLevels`. |
| `tick_building_upgrade` | `System` | Reads `DayEndEvent`; decrements construction timer; on completion applies the upgrade to `HouseState`, `AnimalState`, or `BuildingLevels` and fires toast + SFX. |

**Resource defined:** `BuildingLevels { coop_tier, barn_tier, silo_built, upgrade_in_progress }`

---

### `stats.rs`

| Function | Signature | Purpose |
|---|---|---|
| `track_crop_harvests` | `System` | Reads `CropHarvestedEvent`; updates `HarvestStats.crops[crop_id] = (count, revenue_gold)`. Applies quality multiplier from `ItemQuality::sell_multiplier()`. |
| `track_animal_products` | `System` | Reads `AnimalProductEvent`; categorises products as egg/milk/wool/other; accumulates `AnimalProductStats.total_revenue`. |

**Resources defined:** `HarvestStats`, `AnimalProductStats`

---

### `evaluation.rs`

| Function | Signature | Purpose |
|---|---|---|
| `check_evaluation_trigger` | `System` | Every frame: if Year ≥ 3, Spring, Day 1, and not yet evaluated, fires `EvaluationTriggerEvent`. |
| `handle_evaluation` | `System` | Reads `EvaluationTriggerEvent`; scores player across 8 categories (21 points max); maps to 1–4 candles; updates `EvaluationScore`; sends result toast. Handles re-evaluation comparisons. |

Scoring categories: Earnings (4 pts), Friends (2 pts), Spouse (2 pts), Skills (4 pts), Farm (3 pts), Collection (1 pt), Community (1 pt), Extras (1 pt).

---

### `achievements.rs`

| Function | Signature | Purpose |
|---|---|---|
| `check_achievements` | `System` | Every frame: iterates all 30 `ACHIEVEMENTS`; for unlocked ones skips; evaluates conditions via `evaluate_condition()`; pushes newly met ones to `Achievements.unlocked` and fires `AchievementUnlockedEvent`. |
| `notify_achievement_unlocked` | `System` | Reads `AchievementUnlockedEvent`; sends toast. |
| `track_achievement_progress` | `System` | Reads `ToolUseEvent` (Pickaxe → `rocks_broken`, Hoe → `crops_planted`) and `CropHarvestedEvent` (Gold/Iridium quality → `gold_crops`). Stores counters in `Achievements.progress`. |

---

### `play_stats.rs`

| Function | Signature | Purpose |
|---|---|---|
| `track_crops_harvested` | `System` | `CropHarvestedEvent` → `PlayStats.crops_harvested++` |
| `track_fish_caught` | `System` | `ItemPickupEvent` where item is in `FishRegistry` → `PlayStats.fish_caught++` |
| `track_day_end` | `System` | `DayEndEvent` → `PlayStats.days_played++` and adds bin contents to `PlayStats.items_shipped` |
| `track_gifts_given` | `System` | `GiftGivenEvent` → `PlayStats.gifts_given++` |
| `track_animals_petted` | `System` | `AnimalProductEvent` → `PlayStats.animals_petted++` (proxy) |
| `track_gold_earned` | `System` | Positive `GoldChangeEvent` → `PlayStats.total_gold_earned++` |
| `track_recipes_cooked` | `System` | `EatFoodEvent` → `PlayStats.recipes_cooked++` (proxy) |

---

### `tool_upgrades.rs`

This module is a **stub placeholder**. The file contains only a doc comment explaining that the actual tool upgrade helpers were moved to `crate::shared`. No code is present.

---

## 2. Gold Flow

### Sources (Gold IN)

| Source | Mechanism | Amount |
|---|---|---|
| **Shipping bin** | `process_shipping_bin_on_day_end` fires `GoldChangeEvent(+total)` on `DayEndEvent` | `sell_price * qty` per item in bin |
| **Shop sell-back** | `shop_screen.rs` directly does `player.gold += price` (bypasses event) | Item's `sell_price` from registry |
| **Other domains** | Any domain may fire `GoldChangeEvent { amount > 0, reason }` | Domain-defined |

### Sinks (Gold OUT)

| Sink | Mechanism | Cost |
|---|---|---|
| **Shop purchase** | `shop_screen.rs` directly does `player.gold -= listing.price` (bypasses event) | `listing.price` (from `ShopData`) |
| **Tool upgrade** | `blacksmith.rs` directly does `player_state.gold -= gold_cost` AND fires `GoldChangeEvent(-)` | 2000 / 5000 / 10000 / 25000g per tier |
| **Building upgrade** | `buildings.rs` directly does `player_state.gold -= gold_cost` AND fires `GoldChangeEvent(-)` | 100–50,000g depending on building/tier |

### Gold tracking

- `EconomyStats` accumulates `total_gold_earned` / `total_gold_spent` / `total_transactions` only via the `apply_gold_changes` system watching `GoldChangeEvent`.
- `PlayStats.total_gold_earned` accumulates from all positive `GoldChangeEvents` via `track_gold_earned`.

---

## 3. Shipping Bin — End-of-Day Processing

1. **Deposit**: Player interaction fires `ShipItemEvent { item_id, quantity }`.  
   `place_in_shipping_bin` removes items from `Inventory` and merges them into `ShippingBin`.

2. **Preview**: `update_shipping_bin_preview` keeps `ShippingBinPreview.{pending_value, item_count}` up to date each frame (change-detected).

3. **Day-end sale**: On `DayEndEvent`, `process_shipping_bin_on_day_end`:
   - Iterates `ShippingBin.items`
   - Looks up `sell_price` from `ItemRegistry` (unknown items default to **1g**)
   - Sends one `GoldChangeEvent` for the total
   - Sends a toast ("Shipping: earned Xg from Y items")
   - Clears `ShippingBin.items`

4. **Stats side-channel**: `track_day_end` (play_stats) also reads `DayEndEvent` and snapshot-adds the bin's quantity total to `PlayStats.items_shipped` before the bin is cleared (both systems fan off the same event, so ordering is safe).

**No quality multiplier** is applied during shipping. Only the base `sell_price` is used, regardless of crop/item quality. Fish, artisan goods, and crops are all priced at flat `sell_price` from the registry.

---

## 4. Tool Upgrade System (Blacksmith)

### Flow

1. Player enters the Blacksmith map → `on_enter_shop` sets `GameState::Shop`.
2. UI sends `ToolUpgradeRequestEvent { tool }`.
3. `handle_upgrade_request` validates:
   - `ActiveShop.shop_id == Blacksmith`
   - Tool not already in `ToolUpgradeQueue`
   - Tool has a next tier (not Iridium)
   - `PlayerState.gold >= target_tier.upgrade_cost()`
   - Inventory has required bars (`blacksmith.rs::required_bars_for_tier`)
4. On success: gold and bars deducted; `PendingUpgrade { tool, target_tier, days_remaining: 2 }` pushed to queue.
5. Each `DayEndEvent`: `tick_upgrade_queue` decrements timers; at 0, applies `player_state.tools.insert(tool, new_tier)` and fires `ToolUpgradeCompleteEvent` + toast.

### Tier progression (all tools)

| Tier | Gold Cost | Bars Required |
|---|---|---|
| Basic → Copper | 2,000g | 5 × copper_bar |
| Copper → Iron | 5,000g | 5 × iron_bar |
| Iron → Gold | 10,000g | 5 × gold_bar |
| Gold → Iridium | 25,000g | 5 × iridium_bar |

*(Costs from `ToolTier::upgrade_cost()` in `shared/mod.rs`, queried via `target_tier.upgrade_cost()`)*

### Effects of upgraded tools
Defined in `shared/mod.rs`:
- **Watering can**: area coverage increases (1×1 → 1×3 → 3×3 → etc.)
- **All tools**: stamina cost multiplier decreases (1.0 → 0.85 → 0.7 → 0.55 → 0.4)

---

## 5. Building Upgrade System (Coop/Barn/House/Silo)

### Buildings and tiers

| Building | Tiers | Animal Capacity |
|---|---|---|
| Coop | None → Basic → Big → Deluxe | 0 → 4 → 8 → 12 |
| Barn | None → Basic → Big → Deluxe | 0 → 4 → 8 → 12 |
| House | Basic → Big → Deluxe | N/A (unlocks kitchen, nursery) |
| Silo | None → Basic only | N/A (enables hay storage) |

### Costs (from `buildings::upgrade_cost`)

| Building | To Tier | Gold | Materials |
|---|---|---|---|
| House | Big | 10,000g | 200 wood |
| House | Deluxe | 50,000g | 100 hardwood |
| Coop | Basic | 4,000g | 150 wood + 50 stone |
| Coop | Big | 10,000g | 200 wood + 100 stone |
| Coop | Deluxe | 20,000g | 250 wood + 150 stone |
| Barn | Basic | 6,000g | 200 wood + 75 stone |
| Barn | Big | 12,000g | 250 wood + 125 stone |
| Barn | Deluxe | 25,000g | 250 wood + 200 stone |
| Silo | Basic | 100g | 50 stone + 5 copper_bar |

### Flow

1. UI/player sends `BuildingUpgradeEvent { building, to_tier }`.
2. `handle_building_upgrade_request` validates: no upgrade in progress, sufficient gold, sufficient materials.
3. Deducts gold and materials; sets `BuildingLevels.upgrade_in_progress = Some((building, tier, 2))`.
4. Each `DayEndEvent`: `tick_building_upgrade` decrements timer; at 0, applies upgrade:
   - **House**: updates `HouseState.tier` and unlocks kitchen/nursery flags.
   - **Coop/Barn**: sets `AnimalState.has_coop/barn = true`, sets `coop_level/barn_level` (1/2/3), updates `BuildingLevels.coop_tier/barn_tier`.
   - **Silo**: sets `BuildingLevels.silo_built = true`.
5. Sends completion toast + SFX.

**Only one building upgrade can be in progress at a time** (global lock via `upgrade_in_progress`).

---

## 6. Bugs and Gaps

### 🔴 CRITICAL: Double Gold Deduction on Tool Upgrades

**File:** `src/economy/blacksmith.rs`, line 166–170  
`handle_upgrade_request` does **both**:
```rust
player_state.gold -= gold_cost;          // direct mutation
gold_writer.send(GoldChangeEvent { amount: -(gold_cost as i32), ... }); // event
```
`apply_gold_changes` will subsequently process the event and deduct the same amount again. **The player pays twice for every tool upgrade.**

---

### 🔴 CRITICAL: Double Gold Deduction on Building Upgrades

**File:** `src/economy/buildings.rs`, line 127–131  
Identical pattern:
```rust
player_state.gold = player_state.gold.saturating_sub(gold_cost);
gold_writer.send(GoldChangeEvent { amount: -(gold_cost as i32), ... });
```
`apply_gold_changes` will deduct again. **The player pays twice for every building upgrade.**

---

### 🟠 HIGH: `ShippingLog.shipped_items` Is Never Populated

**File:** `src/economy/shipping.rs` / `src/economy/evaluation.rs:183`  
`process_shipping_bin_on_day_end` never writes to `ShippingLog.shipped_items`. The collection category in the year-end evaluation (`shipping_log.shipped_items.len() >= 30`) will always be 0 (HashMap is always empty). The "Collection" evaluation point is **permanently unachievable**.

---

### 🟠 HIGH: Shop Transactions Bypass `GoldChangeEvent`

**File:** `src/ui/shop_screen.rs`, lines 585 and 602  
Shop buys (`player.gold -= listing.price`) and sells (`player.gold += price`) directly mutate `PlayerState.gold` without firing `GoldChangeEvent`. Consequences:
- `EconomyStats.total_gold_spent` is never incremented for shop purchases.
- `EconomyStats.total_gold_earned` is never incremented for shop sell-backs.
- `PlayStats.total_gold_earned` (tracked via `track_gold_earned`) misses shop sell income.
- `total_transactions` counter is inaccurate.
- Achievements/evaluation criteria that depend on `total_gold_earned` are understated.

---

### 🟡 MEDIUM: Duplicate Cost Table — `ToolTier::upgrade_cost()` vs `required_bars_for_tier()`

**Files:** `src/shared/mod.rs:183–244`, `src/economy/blacksmith.rs:10–18`

`shared/mod.rs` defines three helpers: `upgrade_cost()`, `upgrade_cost_gold()`, `upgrade_bars_needed()`, `upgrade_bar_item()`.  
`blacksmith.rs` defines a private `required_bars_for_tier()` that duplicates the bar requirements.

Currently the values agree, but any future change to tier costs/materials must be made in **two places** (`shared/mod.rs` AND `blacksmith.rs`). The `upgrade_cost_gold()` method on `ToolTier` in shared is also dead code — `blacksmith.rs` uses `target_tier.upgrade_cost()` instead.

---

### 🟡 MEDIUM: `UpgradeEntry` Type Alias Is Unused

**File:** `src/economy/blacksmith.rs:239–249`  
`pub type UpgradeEntry = (ToolKind, ToolTier, ToolTier, u32, &'static str, u8, bool, bool, bool)` is declared public but no function in the economy domain returns or constructs this type. There is no `build_upgrade_entries()` function. The blacksmith UI must reconstruct this logic independently, risking divergence.

---

### 🟡 MEDIUM: `ToolUpgradeCompleteEvent` Is Immediately Drained

**File:** `src/economy/blacksmith.rs:70–72`, `src/economy/mod.rs:80`  
`drain_upgrade_complete` consumes all `ToolUpgradeCompleteEvent`s silently. The event exists as a public signal but no UI or other system subscribes to it. The player only receives a generic toast message. No "pickup at blacksmith" mechanic is implemented (the toast text says "Pick it up at the Blacksmith" but the item is applied immediately at day-end without a pickup step).

---

### 🟡 MEDIUM: `track_animals_petted` Uses Wrong Proxy

**File:** `src/economy/play_stats.rs:99–106`  
`PlayStats.animals_petted` is incremented via `AnimalProductEvent` (collecting a product), not an actual petting interaction. An animal can produce without being petted. Conversely, petting without collecting a product is not counted. Achievement "Pet Lover" (max happiness on a pet) does not depend on this counter, so the impact is limited to `PlayStats` accuracy.

---

### 🟡 MEDIUM: `track_recipes_cooked` Uses Wrong Proxy

**File:** `src/economy/play_stats.rs:131–138`  
`PlayStats.recipes_cooked` is incremented via `EatFoodEvent`, not a crafting/cooking event. Eating pre-existing or purchased food would incorrectly increment this counter. The "Chef" achievement checks `stats.recipes_cooked >= 20` so over-counting could falsely unlock it. The evaluation uses `unlocked_recipes.ids.len()` for the skills point, which is correct, but the achievement condition is flawed.

---

### 🟡 MEDIUM: Shipping Sell Price Ignores Item Quality

**File:** `src/economy/shipping.rs:110–113`  
All items in the shipping bin are sold at their flat `sell_price` from the registry. There is no quality multiplier applied at time of sale. `HarvestStats` does account for quality via `sell_multiplier()`, but the actual gold earned via the bin does not. Gold star / iridium crops are sold at the same price as normal crops.

---

### 🟢 LOW: Hardcoded `2` Day Timer for Upgrades

**Files:** `src/economy/blacksmith.rs:183`, `src/economy/buildings.rs:139`  
Both systems hardcode `days_remaining: 2` and `upgrade_in_progress = Some((..., 2))`. `shared/mod.rs` provides `ToolTier::upgrade_days() -> u8` (which always returns 2), but it is never called. The duration is not configurable per tier or building type.

---

### 🟢 LOW: Fallback Sell Price of 1g for Unknown Items

**File:** `src/economy/shipping.rs:113`, `:163`  
Items not found in `ItemRegistry` fall back to 1g each. This is silent — no warning is logged — so a misconfigured item ID (e.g., after a rename) would silently sell for 1g instead of failing loudly.

---

### 🟢 LOW: `evaluation.rs` has a No-Op Stub Function in Documentation

**File:** `src/economy/evaluation.rs:261–274`  
A function documented as "allow re-evaluation" is described in a doc comment but is a no-op stub (commented out). The re-evaluation mechanic works via `handle_evaluation`'s `was_evaluated` check, but the described "reset `evaluated` to false" mechanism for the shrine UI is not implemented.

---

### 🟢 LOW: `check_achievements` Runs Every Frame

**File:** `src/economy/achievements.rs:318`  
All 30 achievement conditions are evaluated every frame while in `GameState::Playing`. Most conditions are cheap, but `community_pillar`, `completionist`, and `early_riser` iterate over collections or the farm soil map each frame. Change-detection or event-driven triggering would be more efficient at scale.

---

## Summary Table

| # | Severity | Issue |
|---|---|---|
| 1 | 🔴 CRITICAL | Double gold deduction for tool upgrades |
| 2 | 🔴 CRITICAL | Double gold deduction for building upgrades |
| 3 | 🟠 HIGH | `ShippingLog.shipped_items` never populated — evaluation collection point always fails |
| 4 | 🟠 HIGH | Shop transactions bypass `GoldChangeEvent` — stats undercounted |
| 5 | 🟡 MEDIUM | Duplicate bar-cost tables between `shared` and `blacksmith` |
| 6 | 🟡 MEDIUM | `UpgradeEntry` public type alias unused — blacksmith UI must duplicate logic |
| 7 | 🟡 MEDIUM | `ToolUpgradeCompleteEvent` is immediately drained, no subscriber |
| 8 | 🟡 MEDIUM | `animals_petted` proxy counts product collection, not actual petting |
| 9 | 🟡 MEDIUM | `recipes_cooked` proxy counts eating, not cooking |
| 10 | 🟡 MEDIUM | Shipping bin ignores item quality for sell price |
| 11 | 🟢 LOW | Upgrade duration hardcoded to 2 days; `ToolTier::upgrade_days()` never used |
| 12 | 🟢 LOW | Unknown items sell silently for 1g with no warning |
| 13 | 🟢 LOW | Re-evaluation shrine mechanic is stub only |
| 14 | 🟢 LOW | Achievement check runs every frame (performance concern at scale) |
