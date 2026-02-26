# CLAUDE.md — Hearthfield Build Orchestrator

You are building Hearthfield, a Harvest Moon-style farming simulator in Rust with Bevy.

## What Already Exists

- `GAME_SPEC.md` — full game design (READ THIS FIRST)
- `src/shared/mod.rs` — **the type contract** (components, resources, events, states)
- `src/main.rs` — plugin registration and wiring (DO NOT MODIFY)
- `Cargo.toml` — dependencies

## Architecture

Every domain is a Bevy plugin. Domains communicate ONLY through:

1. **Shared Resources** (defined in `crate::shared`)
2. **Events** (defined in `crate::shared`)
3. **State transitions** (via `NextState<GameState>`)

**No domain imports from any other domain.** Only `use crate::shared::*;`

## Asset Setup

Before building, ensure assets are downloaded and placed:

```
assets/
  sprites/          (Sprout Lands sprite sheets)
  ui/               (Sprout Lands UI pack)
  tilesets/          (terrain tiles per season)
  audio/
    music/
    sfx/
```

If assets aren't present, use placeholder colored sprites (like Vale Village) and mark with TODO comments.

## Build Order

### Phase 1: Data Layer

Create `src/data/mod.rs` — loads all registries (items, crops, fish, recipes, NPCs, shops).
This populates ItemRegistry, CropRegistry, FishRegistry, RecipeRegistry, NpcRegistry, ShopData.
Build as DataPlugin that runs in OnEnter(GameState::Loading).

### Phase 2: Domain Plugins (PARALLEL — flat dispatch)

Dispatch 12 workers simultaneously. Each worker:

1. Reads GAME_SPEC.md for their domain
2. Reads src/shared/mod.rs for the type contract
3. Creates src/{domain}/mod.rs exporting a {Domain}Plugin
4. Implements real game logic, NOT stubs
5. Uses `use crate::shared::*;` for all cross-domain types
6. Registers systems with appropriate state guards

```bash
for domain in calendar player farming animals world npcs economy crafting fishing mining ui save; do
  codex exec --full-auto --skip-git-repo-check -c model_reasoning_effort=\"high\" \
    "You are building the ${domain} domain for Hearthfield, a Harvest Moon farming sim in Rust/Bevy.

Read these files first:
- $(pwd)/GAME_SPEC.md (full game design)
- $(pwd)/src/shared/mod.rs (type contract — ALL shared types)
- $(pwd)/src/main.rs (see how your plugin is registered)

Create: $(pwd)/src/${domain}/mod.rs (and any sub-modules you need in src/${domain}/)

Your plugin struct must be: pub struct ${Domain}Plugin;
Import shared types with: use crate::shared::*;
DO NOT modify any file outside src/${domain}/.
DO NOT import from any other domain (only crate::shared).

Write REAL implementations with game logic. Not stubs. Not TODOs.
Use Bevy ECS patterns: systems, queries, resources, events.
Guard gameplay systems with .run_if(in_state(GameState::Playing)) or appropriate state.
Read events from shared:: and write events to shared:: for cross-domain communication.

If sprites/assets aren't available, use Sprite::from_color() placeholders." &
  sleep 3
done
wait
```

### Phase 3: Data Population

After domains are built, create `src/data/` with actual game data:

- `src/data/items.rs` — all item definitions (seeds, crops, fish, minerals, food, etc.)
- `src/data/crops.rs` — crop growth definitions
- `src/data/fish.rs` — fish species
- `src/data/recipes.rs` — crafting and cooking recipes
- `src/data/npcs.rs` — NPC definitions, schedules, gift preferences
- `src/data/shops.rs` — shop inventories
- `src/data/maps.rs` — tilemap definitions for each area

### Phase 4: Compile & Fix

```bash
cargo check 2>&1 | head -100
```

Fix any compilation errors. Common issues:

- Missing mod declarations (add `mod {submodule};` in domain mod.rs)
- Import paths (should all be `use crate::shared::*;`)
- Bevy API mismatches (check Bevy 0.15 API)

### Phase 5: Integration Test

```bash
cargo run
```

Should show: window opens, loading state, transition to playing, camera renders.

## Domain Reference

| Domain   | Plugin Struct  | Primary Responsibility                       |
|----------|----------------|----------------------------------------------|
| calendar | CalendarPlugin | Time progression, day/season events          |
| player   | PlayerPlugin   | Movement, tool use, stamina, collision       |
| farming  | FarmingPlugin  | Soil tilling, crop growth, harvest           |
| animals  | AnimalPlugin   | Animal care, products, buildings             |
| world    | WorldPlugin    | Tilemap loading, rendering, transitions      |
| npcs     | NpcPlugin      | NPC spawning, schedules, dialogue, gifts     |
| economy  | EconomyPlugin  | Shops, shipping bin, gold transactions       |
| crafting | CraftingPlugin | Crafting bench, cooking, processing machines |
| fishing  | FishingPlugin  | Casting, minigame, catch resolution          |
| mining   | MiningPlugin   | Mine floors, rocks, monsters, loot           |
| ui       | UiPlugin       | HUD, inventory screen, menus, dialogue box   |
| save     | SavePlugin     | Save/load, autosave on sleep                 |

## Critical Integration Points

### DayEndEvent Flow (most important event in the game)

When player sleeps, CalendarPlugin sends DayEndEvent. Every domain listens:

- **farming**: advance crop growth, reset watered status, check for withering
- **animals**: check if fed, produce products, advance age
- **economy**: sell shipping bin contents, add gold
- **npcs**: reset gifted_today flags
- **calendar**: advance day, check season change, roll weather
- **save**: autosave

### Tool Use Flow

PlayerPlugin sends ToolUseEvent with target tile coordinates:

- **farming**: hoe → till soil, watering can → water soil
- **world**: axe → chop tree/stump, pickaxe → break rock
- **mining**: pickaxe → break mine rocks
- **fishing**: fishing rod → start fishing minigame

### Map Transition Flow

PlayerPlugin or WorldPlugin sends MapTransitionEvent:

- **world**: despawn current map, load new map
- **player**: reposition player at target coordinates
- **npcs**: spawn/despawn NPCs for new map
- **ui**: screen transition effect

## Rules

- ALL leaf workers use codex with reasoning=high
- DO NOT modify src/shared/mod.rs or src/main.rs after initial setup
- Each domain owns ONLY its own directory
- Test with `cargo check` after each phase
- Record: file count, LOC, compile errors per phase
