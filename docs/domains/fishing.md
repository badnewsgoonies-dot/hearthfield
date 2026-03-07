# Domain Spec: Fishing

## Scope
`src/fishing/` — `mod.rs`, `cast.rs`, `bite.rs`, `fish_select.rs`, `minigame.rs`, `resolve.rs`, `render.rs`, `skill.rs`, `legendaries.rs`, `treasure.rs`

## Responsibility
Full fishing loop: cast → wait for bite → minigame → resolve (catch/fail), fish selection based on location/season/time/weather, fishing skill progression, legendary fish, and treasure chest catches.

## Shared Contract Types (import from `crate::shared`)
- `FishDef` (id, name, location, seasons, time_range, weather_required, rarity, difficulty, sell_price, sprite_index)
- `FishRegistry` (Resource — fish HashMap)
- `FishLocation` (River, Ocean, Pond, MinePool)
- `Rarity` (Common, Uncommon, Rare, Legendary)
- `Inventory`, `ItemId`
- `PlayerState`, `PlayerInput` (fishing_reel)
- `Calendar`, `Season`, `Weather`
- `MapId` (determines water location type)
- `GameState` (transition to `Fishing` state)
- `InputContext` (switch to `Fishing` during minigame)
- `InputBlocks`
- Events: `ToolUseEvent`, `ItemPickupEvent`, `PlaySfxEvent`, `ToastEvent`
- Constants: `TILE_SIZE` (16.0), `Z_EFFECTS` (200.0)

## Quantitative Targets
- 20 fish species:
  - River (Farm/Forest): Carp, Catfish, Bass, Trout, Salmon
  - Ocean (Beach): Sardine, Tuna, Red Snapper, Pufferfish, Octopus
  - Pond (Town): Bullhead, Bream, Sunfish, Crayfish
  - MinePool: Cave Fish, Ghost Fish, Stonefish
  - Legendary: Legend (rainy Forest), Crimsonfish (Summer Beach), Glacierfish (Winter Forest)
- Rarity distribution: 50% Common, 25% Uncommon, 15% Rare, 10% Legendary
- Minigame: bar tracks fish, hold reel to raise bar, release to lower
  - Bar size: starts 40px, increases with skill level (+3px per level)
  - Fish bounce pattern: difficulty determines speed and bounce range
  - Catch threshold: bar overlaps fish icon for 80% of timer
  - Timer: 10 seconds per attempt
- Fishing skill: level 1-10, gain XP per catch
  - XP per catch: Common 3, Uncommon 8, Rare 15, Legendary 25
  - Level thresholds: 10, 25, 50, 100, 200, 350, 550, 800, 1100, 1500
- Treasure: 10% chance per catch of treasure chest (random gems/items)

## Constants & Formulas
- Bite wait: `3.0 + random(0.0, 7.0)` seconds, -0.5 per fishing level
- Fish selection: filter by (location, season, time_range, weather_required), weighted by rarity
- Difficulty scaling: `fish.difficulty * (1.0 - skill_level * 0.03)`
- Catch success: bar_overlap_time / total_time >= 0.8

## Key Systems
1. `cast_system` — on tool_use with FishingRod near water, enter cast state
2. `bite_system` — random timer for fish bite, select fish from valid pool
3. `fish_select` — filter FishRegistry by current conditions, weighted random pick
4. `minigame_system` — fishing bar UI, physics of fish movement, bar tracking
5. `resolve_system` — determine catch/fail, add fish to inventory, award XP
6. `skill_system` — track fishing XP, level up, apply skill bonuses
7. `legendary_system` — track caught legendaries, prevent re-catch
8. `treasure_system` — random treasure chest on successful catch

## Does NOT Handle
- Fish data loading from tables (data domain)
- Fish selling (economy/shipping domain)
- Fishing rod upgrade (economy/blacksmith domain)
- Fishing buff from food (crafting domain)
- Fishing bar UI rendering beyond minigame (ui domain)
