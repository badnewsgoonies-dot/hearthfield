# Domain Spec: Farming

## Scope
`src/farming/` — `mod.rs`, `soil.rs`, `crops.rs`, `harvest.rs`, `render.rs`, `events_handler.rs`, `sprinkler.rs`, `sprinklers.rs`

## Responsibility
Soil tilling/watering, crop planting and growth stages, harvest logic, withering/death, sprinkler auto-watering, and farm rendering.

## Shared Contract Types (import from `crate::shared`)
- `SoilState` (Untilled, Tilled, Watered)
- `SoilTile` (Component — state, grid_x, grid_y)
- `CropTile` (Component — crop_id, current_stage, days_in_stage, watered_today, days_without_water, dead)
- `CropDef` (id, name, seed_id, harvest_id, seasons, growth_days, regrows, regrow_days, sell_price, sprite_stages)
- `CropRegistry` (Resource — crops HashMap)
- `FarmState` (Resource — soil, crops, objects HashMaps)
- `FarmObject` (enum — Tree, Rock, Stump, Bush, Sprinkler, Scarecrow, Fence, Path, ShippingBin)
- `Inventory`, `ItemCategory`
- `SprinklerKind` (Basic, Quality, Iridium), `SprinklerState`, `PlacedSprinkler`, `PlaceSprinklerEvent`
- `ItemQuality` (Normal, Silver, Gold, Iridium)
- Events: `ToolUseEvent`, `DayEndEvent`, `CropHarvestedEvent`, `ItemPickupEvent`, `SeasonChangeEvent`
- Constants: `TILE_SIZE` (16.0), `Z_FARM_OVERLAY` (10.0), `Z_GROUND` (0.0)
- Functions: `grid_to_world_center()`, `world_to_grid()`, `watering_can_area()`

## Quantitative Targets
- 15 crops across 4 seasons:
  - Spring: turnip (4d), potato (6d), cauliflower (12d), strawberry (8d, regrows)
  - Summer: melon (12d), tomato (11d, regrows), blueberry (13d, regrows), corn (14d)
  - Fall: eggplant (13d), pumpkin (13d), cranberry (7d, regrows), yam (10d)
  - Any: wheat (4d), coffee (10d, regrows), ancient fruit (28d, regrows)
- Farm map: 32×24 tiles
- Crop stages: 4-6 per crop, each with sprite
- Withering: 2 days without water → wilted, 3 → dead
- Season transition: non-seasonal crops die
- Sprinkler ranges: Basic 1 (4-adj), Quality 1 (8-adj+diag), Iridium 2 (24 tiles)

## Constants & Formulas
- Wither threshold: `days_without_water >= 2` → wilted, `>= 3` → dead
- Growth: increment `days_in_stage` daily if watered; advance stage when `days_in_stage >= growth_days[current_stage]`
- Regrow: after harvest, reset to stage `growth_days.len() - 2`, set `days_in_stage = 0`
- Quality chance: based on farming level (if applicable, else always Normal)
- Sprinkler auto-water fires at start of each day before growth tick

## Key Systems
1. `handle_tool_use` — listen to `ToolUseEvent`, till soil (Hoe), water (WateringCan), break objects (Axe/Pickaxe)
2. `plant_crop` — when player uses seed on tilled soil, create `CropTile`
3. `daily_growth` — on `DayEndEvent`, advance all crops, check wither
4. `sprinkler_water` — on day start, auto-water tiles in sprinkler ranges
5. `harvest_crop` — on interact with mature crop, add to inventory, fire `CropHarvestedEvent`
6. `season_transition` — on `SeasonChangeEvent`, kill out-of-season crops
7. `render_farm` — spawn/update sprites for soil and crop tiles

## Does NOT Handle
- Crop data definitions (data domain loads from tables)
- Selling harvested crops (economy/shipping domain)
- Crafting sprinklers (crafting domain)
- Farm map generation (world domain generates base map)
- HUD display of farming stats (ui domain)
