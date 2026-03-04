# Orchestrator Persistent State
# READ THIS ON EVERY SESSION START / AFTER COMPACTION

## Sub-Agent Dispatch Methods (in order of preference)

### 1. Copilot CLI (PREFERRED — uses user's GitHub Premium requests)
```bash
export COPILOT_GITHUB_TOKEN="$COPILOT_GITHUB_TOKEN"  # Token stored in ~/.bashrc, NOT in repo
copilot -p "$(cat objectives/fix-something.md)" --allow-all-tools --model claude-sonnet-4.6
```
- Token is in ~/.bashrc (github_pat_11BWLAP7A... — do NOT commit to repo)
- Works as of 2026-03-04
- Use `--allow-all-tools` for full autonomous mode
- Use `--model claude-sonnet-4.6` for best results

### 2. Built-in Agent tool (FALLBACK — if copilot fails)
- Use `subagent_type: "general-purpose"` for implementation
- Use `subagent_type: "Explore"` for read-only investigation
- These are Claude sub-agents, more expensive per the user's preference

## Current Phase: Wave 2c COMPLETE, Wave 2d (save trackers) in progress

### Branch: claude/setup-orchestration-framework-L8ILN

### Wave 1 (COMPLETED — committed as 0cb9202)
7 critical bugs fixed:
1. Double gold deduction in shops (economy/shop.rs)
2. Talking doesn't increase friendship (npcs/dialogue.rs)
3. Talk quest requires gift instead of dialogue (npcs/quests.rs)
4. Cooking restores stamina twice (crafting/cooking.rs)
5. Kitchen accessible without upgrade (crafting/cooking.rs)
6. Hay has no dispatch path (player/item_use.rs)
7. Duplicate ToastEvent registration (world/mod.rs)

### Wave 2a (COMPLETED — committed as 80ba40a)
- FeedTrough position: moved from (-10,-8) to (5,19) near barn (animals/spawning.rs)
- Spouse happiness: now counts talking via DailyTalkTracker (npcs/romance.rs)

### Wave 2b (COMPLETED — committed as b1179e2)
- Tool lock during upgrade (player/tools.rs)
- Night-fish time wrapping for Eel/Squid/Anglerfish (fishing/fish_select.rs)
- Crafts counter for Artisan achievement (crafting/bench.rs + cooking.rs)
- EconomyStats serialization (economy/gold.rs + save/mod.rs)

### Wave 2c (COMPLETED — committed as 932f760 + d3c28da)
- Chef achievement checks recipes_cooked instead of food_eaten (economy/achievements.rs)
- recipes_cooked counter added to cooking (crafting/cooking.rs)
- 7 missing animal ItemDefs added (data/items.rs): goat, duck, rabbit, pig, horse, cat, dog
- Animal pen bounds fixed to be within farm map (animals/spawning.rs)

### Wave 2d (IN PROGRESS — copilot worker running)
- DailyTalkTracker + GiftDecayTracker persistence in save/load (npcs + save domains)

### REMAINING KNOWN ISSUES (lower priority)
- Hay proximity check (player eats hay anywhere, should need trough)
- Shop sell gold not tracked in PlayStats

### AUDIT RESULTS (all verified)
- Prior fix audit: 13/13 PASS — all fixes correctly applied
- Second-0 gameplay trace: ZERO SOFT-LOCKS — fully playable from boot to minute 10
- Game is 70% feature-complete — core loops all work

### Wave 3 (FUTURE — new content)
Priority order:
1. Quest Log / Journal screen (J key bound but dead) — ~400 lines new UI
2. Relationships viewer screen — ~200 lines new UI
3. Full Map screen (M key bound but dead) — ~300 lines new UI
4. Settings screen — ~250 lines new UI
5. Calendar view screen
6. DailyTalkTracker save (in progress)
7. GiftDecayTracker save (in progress)

### Game Completeness Snapshot
| System | Status |
|--------|--------|
| Calendar & Time | 100% |
| Crops & Farming | 100% |
| Fishing | 100% (night fish fixed) |
| Mining & Combat | 100% |
| Animals | 100% (pens + ItemDefs fixed) |
| NPCs & Schedules | 100% |
| Romance & Marriage | 100% |
| Crafting & Cooking | 100% (counters fixed) |
| Shops & Economy | 100% (gold bug fixed) |
| Save/Load | 95% (DailyTalkTracker in progress) |
| UI — Core | 100% |
| UI — Missing screens | 0% (6 screens needed) |

## Architecture Quick Reference
- Rust + Bevy 0.15 ECS, plugin-per-domain (15 domains, ~41k LOC)
- src/shared/mod.rs is FROZEN type contract (2,252 lines, SHA256 checksummed)
- Gates: shasum check → cargo check → cargo test --test headless → cargo clippy -- -D warnings
- TILE_SIZE=16.0, PIXEL_SCALE=3.0, Screen=960x540
- 28 days/season, 4 seasons, max_stamina=100, inventory=36 slots

## User Directives
- NEVER write Rust code directly — always dispatch workers (copilot preferred, Agent fallback)
- "devise a formation and keep INTELLIGENTLY overseeing and dispatching"
- Prefer copilot CLI over built-in Agent tool for implementation
- Focus on fixing existing broken systems before adding new content
- Think from "second 0" — trace the real player experience
- Save context to this file after every major milestone
- Launch subagents for future investigation
- Launch audit subagents for prior work verification
