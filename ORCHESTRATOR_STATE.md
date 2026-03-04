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

## Current Phase: Wave 2b Bug Fixes (IN PROGRESS)

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

### Wave 2a (COMPLETED — committed as 80ba40a, pushed)
- FeedTrough position: moved from (-10,-8) to (5,19) near barn (animals/spawning.rs)
- Spouse happiness: now counts talking via DailyTalkTracker (npcs/romance.rs)

### Wave 2b (IN PROGRESS — 4 copilot workers running)
1. Tool lock during upgrade (player/tools.rs) — copilot worker running
2. Crafting counter for Artisan achievement (crafting/bench.rs + cooking.rs) — copilot worker running
3. Night-fish time wrapping (fishing/fish_select.rs) — DONE, copilot completed
4. EconomyStats serialization (economy/gold.rs + save/mod.rs) — copilot worker running

### Wave 2c (QUEUED — lower priority bugs)
- Chef achievement: checks food_eaten instead of recipes cooked — fix: change check in achievements.rs to use achievements.progress["crafts"] >= 20
- 7 missing animal types in ItemRegistry (data domain — goat, duck, rabbit, pig, horse, cat, dog)
- Hay proximity check (player eats hay anywhere, should need trough)
- Animal spawn bounds in negative space (animals outside all maps)
- Shop sell gold not tracked in PlayStats

### Wave 3 (FUTURE — new content, after all bugs fixed)
- 6 missing UI screens: Quest Log (J key), Relationships, Full Map (M key), Settings, Calendar, Character/Stats
- Quest Log and Map are wired to keys but dead — highest priority new screens

## Deep Audit Findings (from wave 2 investigation)

### Animal System (CRITICAL)
- 7 of 10 animal types missing from ItemRegistry (can't buy goat, duck, rabbit, pig, horse, cat, dog)
- FeedTrough was at (-10,-8) — FIXED in wave 2a to (5,19)
- Hay consumed anywhere without proximity/map check
- Animals spawn in negative world-space, outside all map bounds
- Animals not map-scoped (visible everywhere)

### Economy (HIGH)
- Tools NOT locked during upgrade — FIXING in wave 2b
- "Artisan" achievement impossible (crafts counter never incremented) — FIXING in wave 2b
- "Chef" achievement checks food_eaten not recipes cooked — QUEUED for 2c
- Shop sell gold not tracked in PlayStats

### Fishing (MEDIUM)
- Night-fish time wrapping bug (Eel, Squid, Anglerfish uncatchable) — FIXED in wave 2b
- Mining: fully functional

### Save/Load (MEDIUM-HIGH)
- EconomyStats not serialized — FIXING in wave 2b
- GiftDecayTracker not saved (friendship decay resets on load)
- Animal ECS components not persisted (cooldowns reset on load)

### UI (6 screens MISSING)
- Quest Log, Relationships, Full Map, Settings, Calendar, Character/Stats

## Architecture Quick Reference
- Rust + Bevy 0.15 ECS, plugin-per-domain (15 domains, ~41k LOC)
- src/shared/mod.rs is FROZEN type contract (2,252 lines, SHA256 checksummed)
- Gates: shasum check → cargo check → cargo test --test headless → cargo clippy -- -D warnings
- TILE_SIZE=16.0, PIXEL_SCALE=3.0, Screen=960x540
- 28 days/season, 4 seasons, max_stamina=100, inventory=36 slots
- All cross-domain communication via Events (35+ registered)
- Plugin-per-domain: src/{domain}/mod.rs exports {Domain}Plugin

## User Directives
- NEVER write Rust code directly — always dispatch workers (copilot preferred, Agent fallback)
- "devise a formation and keep INTELLIGENTLY overseeing and dispatching" — continuous improvement loop
- Prefer copilot CLI over built-in Agent tool for implementation
- Focus on fixing existing broken systems before adding new content
- Think from "second 0" — trace the real player experience
- Save context to this file after every major milestone (survives compaction)
- "Launch subagents to investigate future" — always have investigation agents running ahead of implementation
- "Launch audit subagents for prior work" — verify prior fixes are correct

## Objective Files Written
- objectives/fix-economy.md, fix-npcs.md, fix-crafting.md, fix-player.md, fix-world.md (wave 1)
- objectives/fix-animals-trough.md, fix-npcs-spouse.md, fix-player-tool-lock.md (wave 2a)
- objectives/fix-fishing-nighttime.md, fix-crafting-counter.md, fix-economy-save.md (wave 2b)

## Worker Reports
- status/workers/fix-economy.md, fix-crafting.md, fix-player.md, fix-world.md, fix-npcs.md (wave 1)
- status/workers/fix-animals-trough.md, fix-npcs-spouse.md (wave 2a)
- status/workers/fix-fishing-nighttime.md (wave 2b — just completed)
