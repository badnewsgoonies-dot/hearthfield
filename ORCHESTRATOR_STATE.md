# Orchestrator Persistent State
# READ THIS ON EVERY SESSION START / AFTER COMPACTION

## Sub-Agent Dispatch Methods (in order of preference)

### 1. Copilot CLI (PREFERRED — uses user's GitHub Premium requests)
```bash
export COPILOT_GITHUB_TOKEN="$COPILOT_GITHUB_TOKEN"  # Token stored in ~/.bashrc, NOT in repo
copilot -p "$(cat objectives/fix-something.md)" --allow-all-tools --model claude-sonnet-4.6
```
- Token is also in ~/.bashrc
- Works as of 2026-03-04
- Use `--allow-all-tools` for full autonomous mode
- Use `--model claude-sonnet-4.6` for best results

### 2. Built-in Agent tool (FALLBACK — if copilot fails)
- Use `subagent_type: "general-purpose"` for implementation
- Use `subagent_type: "Explore"` for read-only investigation
- These are Claude sub-agents, more expensive per the user's preference

## Current Phase: Wave 2 Bug Fixes

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

### Wave 2a (COMPLETED — needs commit)
- FeedTrough position: moved from (-10,-8) to (5,19) near barn (animals/spawning.rs)
- Spouse happiness: now counts talking via DailyTalkTracker (npcs/romance.rs)
- Gates: all passing (88 tests, cargo check, clippy clean, contract OK)

### Wave 2b (IN PROGRESS — dispatching)
Priority order:
1. Tool lock during upgrade (player/tools.rs) — objective at objectives/fix-player-tool-lock.md
2. Crafting counter for Artisan achievement — needs objective
3. Night-fish time wrapping bug (fishing) — needs objective
4. EconomyStats serialization (economy+save) — needs objective

### Wave 2c (QUEUED — lower priority)
- 7 missing animal types in ItemRegistry (data domain — goat, duck, rabbit, pig, horse, cat, dog)
- Hay proximity check (player eats hay anywhere, should need trough)
- Animal spawn bounds in negative space
- "Chef" achievement checks food_eaten not recipes_cooked
- Shop sell gold not tracked in PlayStats

### Wave 3 (FUTURE — new content)
- 6 missing UI screens: Quest Log, Relationships, Full Map, Settings, Calendar, Character/Stats
- These are NEW features, not bug fixes — do after all existing bugs are fixed

## Architecture Quick Reference
- Rust + Bevy 0.15 ECS, plugin-per-domain (15 domains)
- src/shared/mod.rs is FROZEN type contract (SHA256 checksummed)
- Gates: shasum check → cargo check → cargo test --test headless → cargo clippy -- -D warnings
- TILE_SIZE=16.0, PIXEL_SCALE=3.0, Screen=960x540
- 28 days/season, 4 seasons, max_stamina=100, inventory=36 slots
- All cross-domain communication via Events (35+ registered)

## User Directives
- "I thought you were going to orchestrate…" — NEVER write Rust code directly, always dispatch workers
- "devise a formation and keep INTELLIGENTLY overseeing and dispatching" — continuous improvement loop
- Prefer copilot CLI over built-in Agent tool
- Focus on fixing existing broken systems before adding new content
- Think from "second 0" — trace the real player experience
