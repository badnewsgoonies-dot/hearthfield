# Hearthfield — A Farming & Life Sim

## Vision

A cozy Harvest Moon / Stardew Valley-inspired farming simulator built in Rust with Bevy.
Top-down 16x16 pixel art. Seasonal calendar, crop farming, animal husbandry, foraging,
fishing, town relationships, festivals, and a gentle narrative about restoring a neglected farm.

## Art Style

- 16x16 pixel art, pastel palette (Sprout Lands aesthetic)
- 2x or 3x upscale rendering (nearest-neighbor)
- Four distinct seasonal palettes: spring (pink/green), summer (gold/green), autumn (orange/brown), winter (blue/white)
- Day/night tint overlay (warm daylight → amber dusk → blue night)

## Core Loop

```
Wake up → Tend farm (water, harvest, plant) → Explore town →
Talk to NPCs / Shop / Fish / Mine → Return home → Sleep →
Day advances → Season changes every 28 days → Year cycles
```

## Asset Sources (download before build)

1. **Sprout Lands - Asset Pack** (Cup Nooble) — terrain, crops, animals, character
   https://cupnooble.itch.io/sprout-lands-asset-pack
2. **Sprout Lands - UI Pack** (Cup Nooble) — buttons, icons, frames
   https://cupnooble.itch.io/sprout-lands-ui-pack
3. **LPC Farming Tilesets** (Daniel Eddeland) — supplementary crops, props
   https://opengameart.org/content/lpc-farming-tilesets-magic-animations-and-ui-elements

Asset directory: assets/ with subdirectories per category.
All sprites loaded as texture atlases with Bevy's asset system.

-----

## 12 Domains

### 1. Calendar & Time (`calendar`)

- 4 seasons × 28 days = 112-day year
- Time of day: 6:00 AM to 2:00 AM (20 hours, ~1 real minute = 10 game minutes)
- Time pauses in menus, cutscenes, and dialogue
- Season transitions trigger visual changes (tileset swap, palette tint)
- Year counter for long-term progression
- Festival days: 1 per season (Spring Dance, Summer Luau, Fall Harvest, Winter Star)

### 2. Player (`player`)

- 4-directional movement with walk/run animations
- Tool use animations: hoe, watering can, axe, pickaxe, fishing rod, scythe
- Stamina/energy system: 100 base, tool use costs 2-8, eating restores
- Inventory: 36 slots (hotbar 12 + backpack 24)
- Currently equipped tool (cycle with Q/E or number keys)
- Collision with terrain, objects, NPCs

### 3. Farm & Crops (`farming`)

- Tilled soil tiles (hoe on dirt → tilled)
- Watering (watering can on tilled → watered, visual darkening)
- Planting (seed on tilled → crop stage 0)
- Crop growth: 4-6 stages over N days, requires daily water
- Withering: 2 days without water → wilted, 1 more → dead
- Harvest: interact with mature crop → item added to inventory
- Seasonal crops: each crop only grows in 1-2 seasons
- 15 crops initially:
  - Spring: turnip (4d), potato (6d), cauliflower (12d), strawberry (8d, regrows)
  - Summer: melon (12d), tomato (11d, regrows), blueberry (13d, regrows), corn (14d)
  - Fall: eggplant (13d), pumpkin (13d), cranberry (7d, regrows), yam (10d)
  - Any: wheat (4d), coffee (10d, regrows), ancient fruit (28d, regrows)
- Sprinkler system: craft to auto-water adjacent tiles
- Scarecrow: prevents crow events in radius

### 4. Animals (`animals`)

- Chicken: buy from shop, lay eggs daily if fed and happy
- Cow: buy from shop, milk daily if fed and happy
- Sheep: buy from shop, shear wool every 3 days
- Cat/Dog: pet companion, no produce, happiness bonus
- Happiness: 0-255, affected by feeding, petting, being outside
- Barn and coop buildings (purchase/upgrade)
- Feed trough: place hay to feed animals
- Animal aging: baby → adult (7 days)

### 5. World & Maps (`world`)

- Tilemap-based: 16x16 tiles, maps defined as 2D arrays
- Multiple maps: Farm (64x64), Town (48x48), Beach (32x32), Forest (40x40), Mine entrance (24x24)
- Map transitions via edge-walking or door interaction
- Collision layer: solid, walkable, water, tilled
- Seasonal tileset variants (grass color, tree leaves, snow)
- Object layer: trees, rocks, stumps, bushes (breakable with tools)
- Forageable items spawn per season (berries, mushrooms, flowers)

### 6. NPCs & Dialogue (`npcs`)

- 10 named NPCs with schedules (location varies by time/day/season)
- Dialogue trees: greeting → topics → goodbye
- Gift system: each NPC has loved/liked/neutral/disliked/hated items
- Friendship: 0-1000 points per NPC (10 hearts, 100 per heart)
- Gift response dialogue varies by preference level
- Birthday: +8x gift points on birthday
- 2 marriage candidates (expandable)
- NPC pathfinding: walk between schedule waypoints

### 7. Shops & Economy (`economy`)

- General store: seeds, basic tools, recipes
- Animal shop: chickens, cows, sheep, barn/coop upgrades
- Blacksmith: tool upgrades (copper → iron → gold → iridium)
- Shipping bin: place items, sold at end of day at base price
- Dynamic pricing: none (fixed prices for simplicity)
- Currency: gold (start with 500)
- Tool upgrade costs: 2000/5000/10000/25000 gold + bars

### 8. Crafting & Cooking (`crafting`)

- Crafting bench: combine materials into items (sprinklers, fences, paths, machines)
- Cooking: use kitchen (house upgrade) to make food from recipes
- Recipes: found as gifts, bought, or earned from friendship
- Food restores energy and can give temporary buffs
- 20 crafting recipes, 15 cooking recipes initially
- Processing machines: furnace (ore→bars), preserves jar (fruit→jam), cheese press, loom

### 9. Fishing (`fishing`)

- Fishing rod: cast into water tiles
- Minigame: timing-based (hold/release to keep bar on fish)
- Fish vary by: season, location (river/ocean/pond), time of day, weather
- 20 fish species with rarity tiers (common/uncommon/rare/legendary)
- Bait and tackle: improve catch rate
- Fish can be sold, cooked, or gifted

### 10. Mining (`mining`)

- Mine: 20 floors, descend via ladder (found by breaking rocks)
- Rocks drop: stone, copper ore, iron ore, gold ore (deeper = better)
- Gems: quartz, amethyst, emerald, ruby, diamond (rare)
- Combat: slimes and bats (simple bump-attack, 3 enemy types)
- Health: separate from stamina, restored by food
- Elevator: unlocks every 5 floors for quick return
- Mining consumes pickaxe stamina

### 11. UI System (`ui`)

- HUD: time/date display, gold, stamina bar, current tool
- Inventory grid with drag-and-drop
- Dialogue box: bottom-screen, portrait + text + advance prompt
- Shop interface: buy/sell with item icons and prices
- Crafting menu: recipe list with material requirements
- Pause menu: save, settings, quit
- Map screen: shows current area
- NPC friendship overview: heart meters

### 12. Save & Settings (`save`)

- Full state serialization: farm state, inventory, relationships, calendar, animals, mine progress
- 3 save slots
- Auto-save on sleep (end of day)
- Settings: volume (music/sfx), window size, keybinds

-----

## Technical Architecture

### Bevy Plugin Structure (mirrors Vale Village)

```
main.rs
  → CalendarPlugin
  → PlayerPlugin
  → FarmingPlugin
  → AnimalPlugin
  → WorldPlugin
  → NpcPlugin
  → EconomyPlugin
  → CraftingPlugin
  → FishingPlugin
  → MiningPlugin
  → UiPlugin
  → SavePlugin
```

### Integration Pattern (from Vale Village analysis)

- **Shared Resources**: `Calendar`, `PlayerState`, `Inventory`, `FarmState`, `Economy`, `Relationships`
- **Events**: `DayEndEvent`, `SeasonChangeEvent`, `ItemPickupEvent`, `DialogueStartEvent`, `ShopTransactionEvent`
- **States**: `GameState` (Loading, Playing, Paused, Dialogue, Shop, Fishing, Mining, Menu)
- **No direct cross-domain function calls** — everything through ECS Resources + Events + State transitions

### Asset Pipeline

- Texture atlases loaded in Loading state
- Sprite indices reference atlas positions
- Seasonal variants: swap atlas or apply palette shader
- Tile rendering: Bevy tilemap via `bevy_ecs_tilemap` or manual sprite batch

-----

## Build Strategy (Exp 09 validated)

- Flat dispatch: 1 orchestrator → 12 parallel codex workers
- Shared contract: `src/shared/components.rs` + `src/shared/events.rs` + `src/shared/resources.rs`
- Each worker owns `src/{domain}/` exclusively
- Integration: `main.rs` registers all plugins (Bevy handles wiring)
- Type contract written by orchestrator BEFORE worker dispatch
- Workers import from `crate::shared::*` only
