# Gameplay Action Audit — Social & Economy
_Generated: 2026-03-03_

Legend: ✅ YES · 🟡 PARTIAL · ❌ NO

---

## SOCIAL ACTIONS

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 1 | **Talk to an NPC (dialogue appears)** | ✅ YES | `src/npcs/dialogue.rs:17-90` — `handle_npc_interaction()` checks `player_input.interact` (F key), finds closest NPC within 1.5 tiles, calls `build_dialogue_lines()`, transitions to `GameState::Dialogue` | — |
| 2 | **Give a gift to an NPC (item consumed, hearts change)** | ✅ YES | `src/npcs/gifts.rs:131-236` — R-key press, closest NPC found, once-per-day guard, item removed at line 213, `GiftGivenEvent` at line 230; `gifts.rs:55-61` applies friendship via `relationships.add_friendship()` | — |
| 3 | **Gift preferences (loved/liked/neutral/disliked/hated responses)** | ✅ YES | `src/data/npcs.rs` — all 10 NPCs have `gift_preferences: HashMap<ItemId, GiftPreference>` with all 5 tiers; `src/npcs/gifts.rs:104-113` maps Loved→+80, Liked→+45, Neutral→+20, Disliked→−20, Hated→−40; `src/npcs/dialogue.rs:461-531` renders NPC-specific reaction text | — |
| 4 | **Hearts increase from daily talking** | ❌ NO | `src/npcs/dialogue.rs:17-90` — `handle_npc_interaction()` enters Dialogue state but never calls `add_friendship()`. No friendship gain on talk anywhere in `src/npcs/` | Hearts only increase via gifts (#5) and quests (#10). Daily-talk hearts not implemented. |
| 5 | **Hearts increase from gifting** | ✅ YES | `src/npcs/gifts.rs:55-61` — friendship delta applied immediately; line 57 applies birthday ×8 multiplier; `src/shared/mod.rs:691-694` — `add_friendship()` clamps result to 0–1000 (0–10 hearts) | — |
| 6 | **Accept a quest from an NPC** | 🟡 PARTIAL | `src/npcs/quests.rs:291-540` — `post_daily_quests()` auto-generates and auto-accepts 2–3 quests daily, immediately inserting into `QuestLog.active`; `QuestAcceptedEvent` fired at line 536; no accept/reject UI | Player cannot choose to accept or decline — quests are silently auto-accepted. No interactive acceptance flow. |
| 7 | **Complete a delivery quest (bring item to NPC)** | ✅ YES | `src/npcs/quests.rs:612-622` — `track_quest_progress()` matches `QuestObjective::Deliver`, increments `delivered` on `ItemPickupEvent`; line 620 triggers completion; `quests.rs:689-741` awards gold + friendship | — |
| 8 | **Complete a slay quest (kill N monsters)** | ✅ YES | `src/npcs/quests.rs:781-810` — `track_monster_slain()` reads `MonsterSlainEvent`, matches `monster_kind`, increments counter; completes at target quantity; same completion handler at line 689 | — |
| 9 | **Complete a gather quest (collect N items)** | ✅ YES | `src/npcs/quests.rs:589-634` — `track_quest_progress()` handles `CropHarvestedEvent` for Harvest objectives (line 599) and `ItemPickupEvent` for Mine objectives (line 630); both feed the same completion path | — |
| 10 | **Quest rewards gold** | ✅ YES | `src/npcs/quests.rs:708-713` — `handle_quest_completed()` emits `GoldChangeEvent(event.reward_gold)`; `quests.rs:169-180` — `scaled_reward()` computes gold from tier + quantity variance | — |
| 11 | **Give bouquet to start dating (requires 8+ hearts)** | ✅ YES | `src/npcs/romance.rs:78-167` — `handle_bouquet()` checks marriageable NPC (line 100), hearts ≥ 8 (line 139), consumes bouquet item (line 151), sets `RelationshipStage::Dating` (line 160) | — |
| 12 | **Give mermaid pendant to propose (requires 10 hearts + dating)** | ✅ YES | `src/npcs/romance.rs:175-268` — `handle_proposal()` validates: Dating stage (line 204), hearts ≥ 10 (line 222), house tier ≥ Big (line 234), consumes pendant (line 243), schedules wedding in 3 days (line 257) | Proposal also requires Big house upgrade — not mentioned in spec. |
| 13 | **Marriage changes NPC schedule/dialogue** | 🟡 PARTIAL | `src/npcs/romance.rs:305-344` — `handle_wedding()` sets `RelationshipStage::Married`; `romance.rs:352-437` — `spouse_daily_action()` fires hardcoded 8 AM spouse actions (water crops, feed animals, give breakfast, etc.); `romance.rs:526-574` — spouse happiness tracked | No schedule changes (spouse doesn't follow player or move home). No unique post-marriage dialogue lines. Spouse actions are hardcoded to 8 AM only with no location/movement system. |

---

## ECONOMY ACTIONS

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 14 | **Buy item from general store** | ✅ YES | `src/economy/shop.rs:34-72` — `on_enter_shop()` detects map transition to shop; `shop.rs:135-167` — `build_listings()` creates `ActiveListing` entries; `shop.rs:76-87` — `refresh_shop_affordability()` checks gold per frame; `shop.rs:112-129` — `handle_shop_transaction_gold()` fires `GoldChangeEvent` on purchase | — |
| 15 | **Buy seeds from general store** | ✅ YES | Same flow as #14 — seeds are standard `ShopId::GeneralStore` listings in `src/data/shops.rs`, filtered by `season_available`; no separate code path needed | Seeds are seasonal — only available in correct season. |
| 16 | **Buy animal from animal shop** | ✅ YES | `src/economy/shop.rs:46-50` — `MapId::AnimalShop` routes to `ShopId::AnimalShop`; same `handle_shop_transaction_gold()` transaction flow; animal listings configured in shop data | — |
| 17 | **Sell item via shipping bin** | ✅ YES | `src/economy/shipping.rs:22-87` — `place_in_shipping_bin()` listens for `ShipItemEvent`, validates item exists in registry and player has quantity, adds to `ShippingBin`; `shipping.rs:91-155` — `process_shipping_bin_on_day_end()` sells all on `DayEndEvent` | Items are only sold at day-end, not immediately. |
| 18 | **Shipping bin calculates correct price** | 🟡 PARTIAL | `src/economy/shipping.rs:111-114` — reads `item_registry.get(&slot.item_id).sell_price`; line 121 — `sell_price.saturating_mul(slot.quantity)` per slot; `shipping.rs:116-120` — **quality multiplier is a TODO comment**, no quality field on `InventorySlot` | Quality multiplier not implemented. All items sell at base price regardless of quality. |
| 19 | **Upgrade tool at blacksmith (gold + bars consumed, 2-day wait)** | ✅ YES | `src/economy/blacksmith.rs:74-148` — `handle_upgrade_request()`: gold check (line 105), bars check (line 114), `GoldChangeEvent` (line 130-134), bars removed (line 136-138), `PendingUpgrade{days_remaining: 2}` added to queue (line 140-142) | — |
| 20 | **Pick up upgraded tool (auto-applied)** | ✅ YES | `src/economy/blacksmith.rs:162-185` — `tick_upgrade_queue()` runs on `DayEndEvent`, decrements timer, at 0 calls `player_state.tools.insert(tool, new_tier)` (line 174-175); fires `ToastEvent` notification (line 183) | Auto-applied — no manual pickup. Player is notified via toast. |
| 21 | **Upgrade coop/barn (gold + materials, 2-day wait)** | ✅ YES | `src/economy/buildings.rs:61-149` — `handle_building_upgrade_request()`: cost lookup (line 14-35), gold check (line 92), materials check (line 104-122), deductions (line 127-135), `upgrade_in_progress = Some((building, tier, 2))` (line 138); `buildings.rs:151-198` — `tick_building_upgrade()` completes on timer zero | — |
| 22 | **Craft an item at crafting station (C key or interact)** | ✅ YES | `src/crafting/bench.rs:250-270` — C key fires `OpenCraftingEvent{cooking_mode: false}`; `bench.rs:75-115` — transitions to `GameState::Crafting`; `bench.rs:119-199` — `handle_craft_item()` validates ingredients via `has_all_ingredients()`, consumes via `consume_ingredients()`, adds result via `inventory.try_add()` | — |
| 23 | **Cook food at kitchen** | ✅ YES | `src/crafting/cooking.rs:18-58` — `handle_cook_item()` runs in Crafting state with `cooking_mode: true`, filters recipes where `recipe.is_cooking == true`; `cooking.rs:60-65` — resolves `"any_fish"` wildcard; `cooking.rs:96-135` — consumes ingredients, fires `ItemPickupEvent`, applies immediate stamina restore | — |
| 24 | **Eat food (stamina/health restore)** | ✅ YES | `src/player/item_use.rs:46-56` — R key on edible item fires `EatFoodEvent`; `src/crafting/buffs.rs:173-202` — `handle_eat_food()` restores `player_state.stamina` by `event.stamina_restore`, removes item from inventory | Health restore not evidenced — only stamina. |
| 25 | **Food buffs apply (speed, luck, etc.)** | ✅ YES | `src/crafting/buffs.rs:17-148` — `food_buff_for_item()` maps item IDs → `BuffType` + magnitude + duration (e.g., pancakes → Speed 1.15× 90m, lucky_lunch → Luck 1.5× 180m); `buffs.rs:251-300` — `tick_buff_durations()` decrements per game-minute; `buffs.rs:304-370` — `apply_buff_effects()` applies Speed to `movement.speed`, etc. | — |
| 26 | **Gold display updates in real-time on HUD** | ✅ YES | `src/ui/hud.rs:103-334` — `spawn_hud()` creates `HudGoldText` marker; `hud.rs:584-594` — `update_gold_display()` queries marker, checks `player.is_changed()`, updates text to `"{} G".format(player.gold)` | — |

---

## Summary

| Category | YES | PARTIAL | NO |
|----------|-----|---------|-----|
| Social (1–13) | 9 | 2 | 1 |
| Economy (14–26) | 11 | 1 | 0 |
| **Total (26 actions)** | **20** | **3** | **1** |

### Issues Requiring Attention

| Priority | Issue | Actions Affected |
|----------|-------|-----------------|
| 🔴 Missing | Daily talking does not increase hearts | #4 |
| 🟡 Partial | Quest accept is auto-only — no player choice UI | #6 |
| 🟡 Partial | Married spouse has no schedule/location changes or post-marriage dialogue | #13 |
| 🟡 Partial | Shipping bin ignores item quality — no quality multiplier on sell price | #18 |
