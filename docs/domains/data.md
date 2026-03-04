# Domain Spec: Data Tables

## Scope
`src/data/` — `mod.rs`, `crops.rs`, `fish.rs`, `items.rs`, `npcs.rs`, `recipes.rs`, `shops.rs`

## Responsibility
Static data definitions loaded at startup. Populates all registries: items, crops, fish, NPCs, recipes, shops. No game logic — pure data loading.

## Shared Contract Types (import from `crate::shared`)
- `ItemRegistry`, `ItemDef`, `ItemId`, `ItemCategory`
- `CropRegistry`, `CropDef`
- `FishRegistry`, `FishDef`, `FishLocation`, `Rarity`
- `NpcRegistry`, `NpcDef`, `NpcSchedule`, `ScheduleEntry`
- `RecipeRegistry`, `Recipe`
- `ShopData`, `ShopListing`, `ShopId`
- `Season`, `Weather`, `MapId`
- `GiftPreference`

## Quantitative Targets
- Items: 80+ unique items across all categories
  - Seeds: 15 (one per crop)
  - Crops: 15 harvest items
  - Animal products: 6 (egg, milk, wool, cheese, cloth, jam)
  - Fish: 20
  - Minerals: 10 (ores, bars, gems)
  - Crafting materials: 10 (wood, stone, fiber, clay, hardwood, etc.)
  - Cooked food: 15
  - Crafted items: 20
  - Special: bouquet, mermaid pendant, etc.
- Crops: 15 with full growth_days arrays and sprite_stages
- Fish: 20 with location, season, time, weather, rarity, difficulty
- NPCs: 10 with full gift preferences, schedules, dialogue
- Recipes: 35 (20 crafting + 15 cooking)
- Shop listings: 3 shops with seasonal availability

## Key Systems
1. `load_items` — populate `ItemRegistry` with all ItemDef entries
2. `load_crops` — populate `CropRegistry` with all CropDef entries
3. `load_fish` — populate `FishRegistry` with all FishDef entries
4. `load_npcs` — populate `NpcRegistry` with NPC definitions and schedules
5. `load_recipes` — populate `RecipeRegistry` with all Recipe entries
6. `load_shops` — populate `ShopData` with shop listings

## Does NOT Handle
- Game logic (all domains consume registries but data domain doesn't process events)
- Save/load (data is static, not saved — registries are reconstructed on load)
- UI display of data (ui domain reads registries)
- Runtime modification of data (registries are read-only after startup)
