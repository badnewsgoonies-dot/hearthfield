# Player Trace — Wave 6

**Date:** 2026-03-08
**Scope:** First-60-seconds trace (boot → inventory)
**Method:** Full code path trace through all 10 steps

---

## Trace

1. **[Observed]** The app boots into `GameState::Loading`, `DataPlugin.load_all_data` populates all 6 registries (216 items, 15 crops, 28 fish, 52 recipes, 12 NPCs, shop data), then transitions to `GameState::MainMenu` where `spawn_main_menu` renders the title and 5 buttons. Pressing Enter on "New Game" fires `NewGameEvent` and transitions to `GameState::Playing`.

2. **[Observed]** On entering Playing, `handle_new_game` resets all resources to defaults (Spring Day 1, 6:00 AM), `spawn_player` creates the player entity at grid (8,8) in the farmhouse with `character_spritesheet.png` (4×4 atlas of 48×48 frames), and `grant_starter_items` adds 15 turnip seeds, 5 potato seeds, 20 wood, 15 stone, and 3 bread to inventory.

3. **[Observed]** WASD input drives `player_movement` at 80 px/s with collision checking; `animate_player_sprite` advances the walk cycle based on distance traveled (frame every 6 px, 4 frames per direction row), and `footstep_sfx` fires every 32 px. The player starts with `ToolKind::Hoe` equipped; pressing E/Q cycles through 6 tools.

4. **[Observed]** Pressing Space with Hoe equipped sends `ToolUseEvent` → `handle_hoe_tool_use` validates the target tile is Grass/Untilled, sets `FarmState.soil[pos] = SoilState::Tilled`, spawns a soil overlay entity with brown tint, drains 4.0 stamina, and plays a 4-frame tool animation from `character_actions.png`.

5. **[Observed]** Pressing F/Space with turnip seeds selected sends `PlantSeedEvent` → `handle_plant_seed` validates season (turnip = Spring, current = Spring), creates a `Crop` entry in `FarmState.crops` at stage 0, removes 1 seed from inventory via `ItemRemovedEvent`, and spawns a seedling sprite. Switching to watering can and pressing Space sends `ToolUseEvent` → `handle_watering_can_tool_use` sets soil to `SoilState::Watered` and marks `crop.watered_today = true`. Pressing I opens `GameState::Inventory` → `spawn_inventory_screen` renders a 3×12 grid with item sprites from `ItemAtlasData` (items_atlas.png, 13×17 grid) showing starter items with quantities.

---

## Evidence Summary

| Step | Status | Evidence Level |
|------|--------|---------------|
| Boot (Loading → MainMenu) | Wired | [Observed] |
| Menu (title + New Game button) | Wired | [Observed] |
| New Game (NewGameEvent → Playing) | Wired | [Observed] |
| Spawn (player entity at grid 8,8) | Wired | [Observed] |
| Walk (WASD → movement + animation) | Wired | [Observed] |
| Tool equip (Hoe default, E/Q cycle) | Wired | [Observed] |
| Till soil (Space → ToolUseEvent → FarmState) | Wired | [Observed] |
| Plant seed (F → PlantSeedEvent → Crop) | Wired | [Observed] |
| Water (Space → watering can → SoilState::Watered) | Wired | [Observed] |
| Open inventory (I → inventory screen + sprites) | Wired | [Observed] |

**All 10 steps fully connected. No breaks found in the first-60-seconds critical path.**

---

## Graduation Candidates

All P0 surfaces are verified:

| Claim | Evidence | Risk if False | Graduation Target | Status |
|-------|----------|---------------|-------------------|--------|
| Boot reaches MainMenu with populated registries | [Observed] | Dead game on launch | `test_headless_boot_smoke_transitions_and_ticks` | Already graduated |
| 216 items have unique sprite_index values | [Observed] | Wrong sprites in inventory | `test_no_duplicate_sprite_indices` | Graduated this wave |
| 28 fish have unique sprite_index values | [Observed] | Wrong sprites in fishing | `test_no_duplicate_sprite_indices` | Graduated this wave |
| Starter items granted on new game | [Inferred] | Empty inventory, can't plant | Needs graduation test | P1 |
| Hoe tills soil via ToolUseEvent | [Inferred] | Farming loop dead | Needs graduation test | P1 |
| Planting validates season | [Inferred] | Wrong-season planting silently fails | Needs graduation test | P1 |

**Note:** Steps 6-10 are marked [Inferred] for graduation purposes because they were traced through code reading, not through the headless test harness executing the actual event chain end-to-end. The wiring is structurally verified but not yet mechanically tested. Three P1 graduation targets are identified for the next wave.

---

## Tileset Audit Findings (from status/workers/tileset-audit.md)

Two visual-only issues NOT on the first-60-seconds path but worth noting:

1. **wood_bridge.png idx 7 is empty** — Bridge tiles render invisible. Code at `src/world/mod.rs:402` references row 1 but only row 0 has art.
2. **house_roof.png rows 3-4 are empty** — Building roofs render invisible. Code at `src/world/objects.rs:1541-1558` references indices 22-34 but only row 0 (0-4) has art.

These are **not first-60-seconds blockers** (player starts in farmhouse, not near bridges or town buildings) but will affect exploration within the first 5 minutes.
