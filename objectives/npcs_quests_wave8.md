# Worker: NPCS — Quest Content Expansion

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/npcs/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/npcs/quests.rs — existing quest system, DELIVER_TEMPLATES, post_daily_quests(), check_quest_progress(), complete_quest()
2. src/npcs/dialogue.rs — dialogue system, how NPC dialogue references quests
3. src/shared/mod.rs — QuestKind, QuestState, QuestLog, QuestDef

## Context
The quest system has a solid framework with delivery templates and quest generation. But it currently generates random quests from templates. The game needs **hand-crafted seasonal story quests** that give the player goals each season and tie into NPC relationships.

## Deliverables

### 1. Add 12 seasonal story quests (3 per season)
Add a new constant array `SEASONAL_QUESTS` with hand-crafted quests:

**Spring:**
- "Spring Cleanup" — Deliver 20 wood + 20 stone to Mayor Thomas (reward: 500g + 50 friendship)
- "First Harvest" — Harvest 5 parsnips (reward: 300g, unlocks seed discount)
- "Fishing Apprentice" — Catch 3 fish of any kind for Sam (reward: 400g + fishing bait x10)

**Summer:**
- "Summer Bounty" — Ship 10 melons or blueberries (reward: 800g)
- "Beach Cookout" — Deliver 3 cooked_fish + 1 salad to Marco (reward: 600g + 80 friendship)
- "Mining Expedition" — Reach mine floor 40 (reward: 1000g + gold_ore x5)

**Fall:**
- "Harvest Festival Prep" — Deliver 5 pumpkins + 5 corn to Margaret (reward: 700g + cake recipe)
- "Mushroom Hunt" — Forage 8 fall items from the forest (reward: 500g)
- "Animal Husbandry" — Have 3+ animals with happiness > 150 (reward: 600g)

**Winter:**
- "Winter Stockpile" — Have 50+ items in shipping bin total for the season (reward: 1200g)
- "Community Bundle" — Deliver 1 of each bar type: copper, iron, gold (reward: 1500g)
- "Frozen Lake Fishing" — Catch 5 winter fish (reward: 800g + iridium_ore x2)

### 2. Implement seasonal quest posting
Add a system or extend `post_daily_quests` to post the 3 seasonal quests on day 1 of each season (alongside daily random quests). Seasonal quests should:
- Have `expires_on_day: 28` (last day of season)
- Be tagged with a `seasonal: true` field or similar marker
- Not duplicate if already active in QuestLog

### 3. Wire NPC-specific quest givers
For quests that mention specific NPCs (Thomas, Sam, Marco, Margaret), set the `giver` field to that NPC's id so the quest appears associated with them in the UI.

## Quantitative targets
- 12 new seasonal quests (3 per season, verified by count)
- Each quest has: title, description, kind, target quantity, reward gold, giver NPC
- Seasonal quests post on day 1 of their season
- Seasonal quests expire on day 28
- Zero clippy warnings

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/npcs_quests_wave8.md
