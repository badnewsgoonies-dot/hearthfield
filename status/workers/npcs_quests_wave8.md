# Worker Report: NPCS Quest Content Expansion (Wave 8)

## Status: COMPLETE

## Files Modified
- `src/npcs/quests.rs` (+254 lines): Added `SeasonalObj` enum, `SeasonalQuestTemplate` struct, `SEASONAL_QUESTS` constant, `seasonal_obj_to_objective()` helper, and `post_seasonal_quests` system
- `src/npcs/mod.rs` (+3 lines): Imported and registered `post_seasonal_quests` in `NpcPlugin`

## What Was Implemented

### 12 Seasonal Story Quests (3 per season)

**Spring (season_idx = 0):**
1. `seasonal_spring_cleanup` — Deliver 20 wood to `mayor_rex` (500g, +50 friendship)
2. `seasonal_first_harvest` — Harvest 5 turnips for `nora` (300g, +40 friendship)
`seasonal_fishing_apprentice` 3. Catch 1 bass for `sam` (400g, +10 fiber, +45 friendship) 

**Summer (season_idx = 1):**
4. `seasonal_summer_bounty` — Harvest 10 melons for `lily` (800g, +50 friendship)
5. `seasonal_beach_cookout` — Deliver 3 pizza to `marco` (600g, +80 friendship)
6. `seasonal_mining_expedition` — Mine 5 gold_ore for `elena` (1000g, +5 gold_ore, +60 friendship)

**Fall (season_idx = 2):**
7. `seasonal_harvest_festival_prep` — Deliver 5 pumpkins to `margaret` (700g, +1 cake, +60 friendship)
8. `seasonal_mushroom_hunt` — Harvest 8 yam for `mira` (500g, +50 friendship)
9. `seasonal_animal_husbandry` — Deliver 3 wool to `nora` (600g, +60 friendship)

**Winter (season_idx = 3):**
10. `seasonal_winter_stockpile` — Deliver 20 coal to `mayor_rex` (1200g, +70 friendship)
11. `seasonal_community_bundle` — Deliver 1 gold_bar to `elena` (1500g, +80 friendship)
12. `seasonal_frozen_lake_fishing` — Catch 1 sturgeon for `old_tom` (800g, +2 gold_ore, +60 friendship)

### `post_seasonal_quests` System
- Runs every frame; no-ops unless `calendar.day == 1`
- Posts the 3 quests for the current season on day 1
- `days_remaining: Some(28)` → expires after day 28 (end of season)
- Tracker key: `{id}_y{year}` — quests repeat each year, no duplicates within a year
- Also checks `quest_log.active` to avoid adding duplicates if already present
- Registered in `NpcPlugin` in `UpdatePhase::Reactions`

## Quantitative Targets
-  12 seasonal quests (verified by count: 12 entries in `SEASONAL_QUESTS`)
- ✅ 3 per season (Spring=3, Summer=3, Fall=3, Winter=3)
- ✅ Each quest has: title, description, objective, reward_gold, giver NPC
- ✅ Posts on day 1 of the season
- ✅ Expires on day 28 (`days_remaining: Some(28)`)
- ✅ NPC givers: mayor_rex (2), nora (2), sam, lily, marco, elena (2), margaret, mira, old_tom

## Shared Type Imports Used
- `Quest`, `QuestObjective`, `QuestLog`, `QuestPostedEvent`, `QuestAcceptedEvent`
- `Calendar`, `Season`

## Validation Results
- `cargo check` — ✅ PASS (0 errors)
- `cargo test --test headless` — ✅ PASS (88 passed, 0 failed)
- `cargo clippy -- -D warnings` — ✅ PASS (0 errors, 0 warnings in scope)

Note: pre-existing unrelated clippy warning in `src/world/mod.rs:1041` (type_complexity) exists at HEAD and is out of scope.

## Commit
`971580e` feat(npcs): add 12 seasonal story quests posted on day 1 of each season
