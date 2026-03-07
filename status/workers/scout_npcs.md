# NPC System Audit Report

**Files reviewed:** `src/npcs/mod.rs`, `src/npcs/dialogue.rs`, `src/npcs/gifts.rs`,
`src/npcs/quests.rs`, `src/npcs/schedules.rs`, `src/npcs/romance.rs`, `src/data/npcs.rs`

---

## 1. NPC LIST

Heart-dialogue lines = sum across all 4 tiers (tier-0, tier-3, tier-6, tier-9).
Default dialogue = fallback lines shown before any tier is reached.
Gift prefs = number of explicit `prefs.insert(...)` entries (unlisted items → `Neutral`).

| NPC | Role | Default Lines | Heart-Dialogue Lines (per tier) | Gift Prefs | Marriageable |
|---|---|---|---|---|---|
| Margaret | Baker | 3 | 19 (5/5/5/4) | 16 | No |
| Marco | Inn cook | 3 | 19 (5/5/5/4) | 16 | No |
| Lily | Florist | 3 | 19 (5/5/5/4) | 16 | **Yes** |
| Old Tom | Fisherman | 3 | 19 (5/5/5/4) | 16 | No |
| Elena | Blacksmith | 3 | 19 (5/5/5/4) | 16 | **Yes** |
| Mira | Trader | 3 | 19 (5/5/5/4) | 16 | No |
| Doc | Doctor | 3 | 19 (5/5/5/4) | 16 | No |
| Mayor Rex | Mayor | 3 | 12 (3/3/3/3) | 16 | No |
| Sam | Musician | 3 | 12 (3/3/3/3) | 16 | No |
| Nora | Farmer | 3 | 12 (3/3/3/3) | 16 | No |

**Notes:**
- All 10 NPCs have weather comments (Rainy / Stormy / Snowy) and seasonal dialogue variants.
- Each NPC also has birthday dialogue (hard-coded in `dialogue.rs`, not in `data/npcs.rs`).
- The dialogue system falls back through lower tiers when a tier is unavailable.

---

## 2. QUEST LIST

Quests are procedurally generated each day. Templates determine possible objectives.

### Quest Types & Objective Pools

| Type | Template Count | ID Examples | Reward Range |
|---|---|---|---|
| **Deliver** | 26 | `wood`, `egg`, `pizza`, `gold_bar`, `ancient_fruit` | 100–1000 g |
| **Harvest** | 16 | `turnip`→`pumpkin`, `blueberry`, `wheat`, `coffee` | 100–1000 g |
| **Catch** | 16 | `bass`→`anglerfish`, `sardine`, `sturgeon` | 100–1000 g |
| **Mine** | 12 | `copper_ore`→`diamond`, `quartz`, `gold_bar` | 100–1000 g |
| **Talk** | 8 | Visit any NPC and give a gift | 100–1000 g |
| **Slay** | 3 | `green_slime`, `bat`, `rock_crab` | 100–1000 g |

Reward also includes friendship points (20–70) with the quest giver.

### Completability

| Objective | Achievable? | Notes |
|---|---|---|
| Deliver | ✅ | All item IDs exist in `ItemRegistry` |
| Harvest | ✅ | All crop IDs match `CropRegistry` |
| Catch | ✅ | All fish IDs match `FishRegistry` |
| Mine | ✅ | All mine item IDs exist; `gold_bar` comes from smelting, not direct drop |
| Talk | ✅* | Tracked via `GiftGivenEvent`; requires **giving a gift** to complete |
| Slay `green_slime` | ✅ | `MineEnemy::GreenSlime` fires `MonsterSlainEvent` with `"green_slime"` |
| Slay `bat` | ✅ | `MineEnemy::Bat` → `"bat"` |
| Slay `rock_crab` | ✅ | `MineEnemy::RockCrab` → `"rock_crab"` |

---

## 3. RELATIONSHIP (HEARTS)

### Point → Heart Conversion

```
100 friendship points = 1 heart
Max: 10 hearts = 1000 points (MAX_FRIENDSHIP)
Negative friendship is clamped to 0.
```

### Heart → Relationship Stage (via `update_relationship_stages`)

| Hearts | Stage |
|---|---|
| 0–1 | Stranger |
| 2–3 | Acquaintance |
| 4–5 | Friend |
| 6–10 | CloseFriend |
| — | Dating *(set only by bouquet)* |
| — | Engaged *(set only by proposal)* |
| — | Married *(set only by wedding)* |

The system **only promotes, never demotes** (prevents flickering on temporary friendship loss).

### Gift Point Values

| Preference | Points | Birthday Multiplier |
|---|---|---|
| Loved | +80 | ×8 = +640 |
| Liked | +45 | ×8 = +360 |
| Neutral | +20 | ×8 = +160 |
| Disliked | −20 | ×1 (no bonus) |
| Hated | −40 | ×1 (no bonus) |

One gift per NPC per day; duplicate gifts are silently rejected with dialogue.

### Dialogue Tiers

| Tier Key | Heart Range |
|---|---|
| 0 | 0–2 hearts |
| 3 | 3–5 hearts |
| 6 | 6–8 hearts |
| 9 | 9–10 hearts |

---

## 4. MARRIAGE

Marriage is **fully implemented** in `src/npcs/romance.rs`.

### Progression

| Step | Requirement | Item Consumed |
|---|---|---|
| **Dating** | 8+ hearts, not already married | `bouquet` (1×) |
| **Engaged** | Dating + 10 hearts + house tier ≥ Big | `mermaid_pendant` (1×) |
| **Wedding** | 3 game-days after engagement | — (auto fires `WeddingEvent`) |
| **Married** | — | — |

Only **Lily** and **Elena** have `is_marriageable: true`.

### Post-Marriage Spouse Actions (daily at 8:00 AM)

| Action | Probability | Effect |
|---|---|---|
| WaterCrops(6) | 40% | Waters 6 random unwatered crop tiles |
| FeedAnimals | 25% | Sets `fed_today = true` for all animals |
| GiveBreakfast | 15% | Adds item to inventory (`fried_egg`, `pancakes`, `toast`, `porridge`, or `fruit_salad`) |
| RepairFence | 10% | Toast only; no mechanical effect (TODO) |
| StandOnPorch | 10% | Cosmetic; no effect |

**Spouse Happiness:** Tracked as `i8` from −100 to +100. +2/day if gifted, −3/day otherwise. Starts at 50. Warning toasts at <0 and <−50. No gameplay consequence beyond toast messages (no divorce, no penalty mechanic).

---

## 5. BUGS

| # | Severity | Location | Description |
|---|---|---|---|
| B1 | 🔴 Critical | `romance.rs:151` | `bouquet` is **not registered** in `ItemRegistry` (`src/data/items.rs`). `inventory.try_remove("bouquet", 1)` always returns 0 → dating is permanently blocked. |
| B2 | 🔴 Critical | `romance.rs:243` | `mermaid_pendant` is **not registered** in `ItemRegistry`. Proposal is permanently blocked. |
| B3 | 🟠 High | `romance.rs` (spouse breakfast) | `"toast"` and `"porridge"` are **not registered** in `ItemRegistry`. `inventory.try_add(item_id, 1, 99)` silently fails when spouse gives those items. |
| B4 | 🟡 Medium | `quests.rs:308–309` | `npc_names` is populated from `npc_registry.npcs.keys()` (NPC IDs like `"margaret"`), so quest titles display raw IDs: "Lumber Delivery for margaret". Should use `npc_def.name` for display. |
| B5 | 🟡 Medium | `quests.rs:464` | When `npc_names` has only 1 entry the Talk quest target falls back to `"child_lily"` — a non-existent NPC. The gift event will never match and that quest is uncompletable. |
| B6 | 🟡 Medium | `quests.rs` (Talk) | Talk completion is triggered by `GiftGivenEvent`, but quest descriptions say "visit and give a gift" while quest titles (e.g. "Neighbor Check-in") say nothing about gifting. Likely to confuse players. |
| B7 | 🟡 Medium | `romance.rs` (RepairFence) | `SpouseAction::RepairFence` applies no game-state change. Code comment acknowledges it as TODO. |
| B8 | 🟢 Low | `quests.rs` (MINE_TEMPLATES) | `"gold_bar"` in Mine quest templates is a smelted product, not a mine drop. Tracked via `ItemPickupEvent` so it does count, but description "from deep levels" is misleading. |
| B9 | 🟢 Low | `dialogue.rs` (`npc_season_comment`) | Many NPC/season combinations have no entry and return `None` silently (e.g., Nora in Winter, Mira in Spring). Only ~50% of the 40 possible combinations are covered. |
| B10 | 🟢 Low | `romance.rs` (spouse happiness) | `spouse_happiness` metric has no gameplay consequence (no divorce path, no benefit for high happiness). System does produce toast warnings but nothing acts on the value. |
