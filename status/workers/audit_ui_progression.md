# Gameplay Action Audit — UI / Progression

Audited: 2026-03-03  
Auditor: Copilot (claude-sonnet-4.6)  
Scope: `src/ui/`, `src/world/`, `src/save/`, `src/economy/evaluation.rs`, `src/economy/achievements.rs`, `src/shared/mod.rs`, `src/input/mod.rs`, `src/player/`

---

## UI / INPUT

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 1 | Open inventory (E key) | ✅ YES | `KeyBindings.open_inventory = KeyCode::KeyE` (shared/mod.rs:1619); `gameplay_state_transitions` → `GameState::Inventory` (ui/menu_input.rs:35) | — |
| 2 | Close inventory (E or Esc) | ✅ YES | Toggle-close via E: menu_input.rs:52-55; Esc via `action.cancel` → Playing: menu_input.rs:63-70 | — |
| 3 | Open crafting (C key) | ✅ YES | `KeyBindings.open_crafting = KeyCode::KeyC` (shared/mod.rs:1620); `trigger_crafting_key` fires `OpenCraftingEvent` from anywhere (crafting/bench.rs:252-279); no bench-proximity required | Exception: blocked if holding a chest on Farm (bench.rs:265-276) |
| 4 | Close crafting (C or Esc) | ✅ YES | Toggle-close via C: menu_input.rs:56-59; Esc via cancel → Playing: menu_input.rs:67 | — |
| 5 | Open pause menu (Esc from gameplay) | ✅ YES | `KeyBindings.pause = KeyCode::Escape` (shared/mod.rs:1623); `gameplay_state_transitions` → `GameState::Paused` (menu_input.rs:30-33) | — |
| 6 | Save game from pause menu | ✅ YES | Pause menu option index 1 "Save Game" fires `SaveRequestEvent { slot }` (ui/pause_menu.rs:176-179); `handle_save_complete_in_pause_menu` shows result (pause_menu.rs:189-207) | — |
| 7 | Load game from pause menu | ❌ NO | `PAUSE_OPTIONS = ["Resume", "Save Game", "Quit to Menu"]` (pause_menu.rs:24) — no Load option | Load is only available from the main menu (main_menu.rs:221-313) |
| 8 | Start new game | ✅ YES | Main menu fires `NewGameEvent` (main_menu.rs:255); save system resets all state (save/mod.rs:763-789) | — |
| 9 | Navigate menu items (arrow keys + Enter) | ✅ YES | `merge_keyboard_to_menu_action` maps `ui_up/down/left/right/confirm` → `MenuAction` (menu_input.rs:10-18); Menu InputContext reads arrows + WASD (input/mod.rs:109-120) | — |
| 10 | Switch tools (1-5 number keys) | ✅ YES | Digit1–Digit9 set `input.tool_slot` (input/mod.rs:69-87); `hotbar_input_handler` sets `inventory.selected_slot` (menu_input.rs:78-87); HUD updates to reflect selection (hud.rs:645-711) | Keys 6-9 also work; question says 1-5 but code supports 1-9 |
| 11 | Use tool (F key or Space) | ⚠️ PARTIAL | Space/LMB is `tool_use` (shared/mod.rs:1617, input/mod.rs:62-64); **F is `interact`, not tool-use**. Space = swing tool. F = interact with object. Both work but do different things. | The "F key" does NOT use a tool; it triggers interactions. Tool use is Space only. |
| 12 | Interact with objects (F key) | ✅ YES | F key mapped to `interact` (shared/mod.rs:1616); `dispatch_world_interaction` processes it (player/interact_dispatch.rs:16-119); sets `InteractionClaimed` | — |
| 13 | Cancel/back in menus (Esc) | ✅ YES | `KeyBindings.ui_cancel = KeyCode::Escape` (shared/mod.rs:1627); `menu_cancel_transitions` returns to Playing (or Cutscene if in Dialogue during cutscene) (menu_input.rs:44-73) | — |

---

## NAVIGATION

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 14 | Walk in 4 directions (WASD or arrow keys) | ✅ YES | Both key sets map to `move_axis` (input/mod.rs:35-44); `player_movement` applies velocity (player/movement.rs:7-57) | — |
| 15 | Map transitions work (walk to edge/door) | ✅ YES | `MapTransition` zones defined per-map (world/maps.rs:133+); `player_interaction.rs:291-305` fires `MapTransitionEvent` when player enters zone | — |
| 16 | Screen fade on map transition | ✅ YES | `trigger_fade_on_transition` listens for `MapTransitionEvent`, sets `ScreenFade { target_alpha: 1.0 }` (ui/transitions.rs:55-68); `update_fade` animates overlay (transitions.rs:70-102) | — |
| 17 | Correct spawn position on new map | ✅ YES | `MapTransition.to_pos` becomes destination; `world/mod.rs:530-531` uses it; save system stores `save_grid_x/y` (save/mod.rs:732-733) | — |
| 18 | Cannot walk through solid tiles (walls, rocks, water) | ✅ YES | `is_blocked` checks `collision_map.solid_tiles` (movement.rs:135); also blocks farm objects (Tree, Rock, Stump, Fence) (movement.rs:144-154) | — |
| 19 | Cannot walk off map edges (bounded) | ✅ YES | `is_blocked` checks `collision_map.bounds` (min_x, max_x, min_y, max_y) (movement.rs:139-143); returns `true` if out-of-bounds | — |

---

## PROGRESSION

| # | Action | Status | Evidence | Issue |
|---|--------|--------|----------|-------|
| 20 | Seasonal festivals trigger on correct dates | ✅ YES | `check_festival_day`: Spring 13 → Egg Festival, Summer 11 → Luau, Fall 16 → Harvest Festival, Winter 25 → Winter Star (calendar/festivals.rs:52-57, 79-114) | — |
| 21 | Festival has interactive activities | ✅ YES | Egg Hunt: collect eggs via F (festivals.rs:125-260); Luau: soup submission (festivals.rs:289-384); Harvest Festival: crop judging (festivals.rs:386-536); Winter Star: gift exchange (festivals.rs:559-680) | — |
| 22 | Year-end evaluation happens | ✅ YES | `check_evaluation_trigger` fires on Year ≥ 3, Spring 1, Day 1 (evaluation.rs:44-48); `handle_evaluation` scores and sends toast (evaluation.rs:50-261) | Evaluation doesn't fire until Year 3 — players in Year 1-2 won't see it |
| 23 | Evaluation scores farming/social/exploration | ✅ YES | 8 categories: earnings (4 tiers), friendships, skills (crops/fish/mine floor/recipes), house upgrades, animals, items shipped, collection, community quests, extras (evaluation.rs:77-210) | — |
| 24 | Achievements unlock based on milestones | ✅ YES | 30 achievements defined in `ACHIEVEMENTS`; `check_achievements` evaluates conditions each Playing frame and fires `AchievementUnlockedEvent` (achievements.rs:319-370) | — |
| 25 | Achievement notification appears | ✅ YES | `show_achievement_toast` converts `AchievementUnlockedEvent` → `ToastEvent` (achievements.rs:382-390); toast UI displays it | — |
| 26 | Tutorial hints show for new players | ✅ YES | 5 hints + 5 objectives defined; `check_tutorial_hints` fires `HintEvent` forwarded to toast (tutorial.rs:57-132); skipped once `tutorial_complete = true` | — |
| 27 | Weather changes daily (sunny/rain/storm/snow) | ✅ YES | `roll_weather` called on each day-end (calendar/mod.rs:350-354); 4 variants: `Sunny`, `Rainy`, `Stormy`, `Snowy` (shared/mod.rs:76-79); Snowy is Winter-only | — |
| 28 | Season changes after 28 days | ✅ YES | `DAYS_PER_SEASON = 28` (shared/mod.rs:953); `trigger_day_end` advances season when `day >= DAYS_PER_SEASON` (calendar/mod.rs:174-201) | — |
| 29 | Visual season changes (tile colors, tree sprites) | ✅ YES | `apply_seasonal_tint` recolors all `MapTile` and `WorldObject` sprites per season (world/seasonal.rs:121-169); 4 terrain tints + tree/bush variants; Fall adds falling leaf particles (seasonal.rs:190+) | Tile sprites are recolored, not replaced — season change is tint-based, not swap-based |
| 30 | Save preserves ALL game state | ✅ YES | `FullSaveFile` includes: Calendar, Inventory, FarmState, AnimalState, Relationships, PlayerState (with `deepest_floor_reached`), MineState, RelationshipStages, FishEncyclopedia, AnimalProductStats, Achievements, TutorialState, EvaluationScore, and more (save/mod.rs:288-338) | — |

---

## Summary

| Category | YES | PARTIAL | NO |
|----------|-----|---------|-----|
| UI/Input (1-13) | 10 | 1 | 1 |
| Navigation (14-19) | 6 | 0 | 0 |
| Progression (20-30) | 11 | 0 | 0 |
| **Total (30)** | **27** | **1** | **1** |

### Critical Findings

1. **#7 — No Load from Pause Menu** (`PARTIAL`/`NO`): The pause menu only offers Resume, Save Game, and Quit to Menu. Players cannot load a save slot mid-session; they must quit to main menu first.

2. **#11 — Tool vs. Interact Key confusion** (`PARTIAL`): The F key triggers interactions (talk to NPC, open chest, pick up item). The Space bar (or LMB) swings tools (hoe, watering can, axe, etc.). These are distinct actions. A player expecting "F to use tool" will find it doesn't work as a standard tool swing — it dispatches an interaction event instead.
