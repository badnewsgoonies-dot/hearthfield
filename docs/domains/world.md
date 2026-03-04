# Domain Spec: World & Maps

## Scope
`src/world/` — `mod.rs`, `maps.rs`, `objects.rs`, `chests.rs`, `lighting.rs`, `weather_fx.rs`, `seasonal.rs`, `ysort.rs`

## Responsibility
Map generation and tilemap rendering, map transitions, breakable objects, forage spawning, day/night lighting, weather visual effects, seasonal tileset changes, Y-sort depth ordering, and storage chests.

## Shared Contract Types (import from `crate::shared`)
- `MapId` (Farm, Town, Beach, Forest, MineEntrance, Mine, PlayerHouse, GeneralStore, AnimalShop, Blacksmith)
- `TileKind` (Grass, Dirt, TilledSoil, WateredSoil, Water, Sand, Stone, WoodFloor, Path, Bridge, Void)
- `MapTransition` (from_map, from_rect, to_map, to_pos)
- `GridPosition` (Component — x, y)
- `StorageChest` (Component — slots, capacity, grid_pos)
- `QualityStack`
- `DayNightTint` (Resource — intensity, tint)
- `YSorted` (Component marker)
- `LogicalPosition` (Component)
- `Calendar`, `Season`, `Weather`
- `PlayerState` (current_map)
- `Interactable`, `InteractionKind`
- Events: `MapTransitionEvent`, `DayEndEvent`, `SeasonChangeEvent`, `ItemPickupEvent`
- Constants: `TILE_SIZE` (16.0), `Z_GROUND` (0.0), `Z_FARM_OVERLAY` (10.0), `Z_ENTITY_BASE` (100.0), `Z_Y_SORT_SCALE` (0.01), `Z_EFFECTS` (200.0), `Z_SEASONAL` (300.0), `Z_WEATHER` (400.0)
- Functions: `grid_to_world_center()`, `world_to_grid()`

## Quantitative Targets
- Map sizes (tiles):
  - Farm: 32×24
  - Town: 28×22
  - Beach: 20×14
  - Forest: 22×18
  - MineEntrance: 14×12
  - PlayerHouse: 16×16
  - GeneralStore: 12×12
  - AnimalShop: 12×12
  - Blacksmith: 12×12
- Forageable items: 4-8 per outdoor map per season, respawn every 2-3 days
- Breakable objects: Trees (health 10, drop wood), Rocks (health 5, drop stone), Stumps (health 5)
- Chest capacity: 36 slots

## Constants & Formulas
- Day/night tint cycle:
  - 6:00-8:00: warm dawn (1.0, 0.9, 0.8) → (1.0, 1.0, 1.0)
  - 8:00-17:00: full daylight (1.0, 1.0, 1.0)
  - 17:00-20:00: amber dusk (1.0, 0.85, 0.6)
  - 20:00-2:00: blue night (0.4, 0.4, 0.7), intensity 0.5
- Y-sort: Z = `Z_ENTITY_BASE - world_y * Z_Y_SORT_SCALE`
- Forage spawn: random positions on Grass tiles, avoid occupied

## Key Systems
1. `generate_map` — create tilemap for each `MapId` with collision, object, and transition layers
2. `map_transition` — listen to `MapTransitionEvent`, despawn current map, spawn new
3. `y_sort_system` — PostUpdate: set Z for all `YSorted` entities based on Y position
4. `day_night_cycle` — update `DayNightTint` based on `Calendar.hour/minute`
5. `weather_effects` — spawn rain/snow/storm particles based on `Calendar.weather`
6. `seasonal_visuals` — on `SeasonChangeEvent`, swap tileset palettes
7. `forage_spawning` — on `DayEndEvent`, spawn seasonal forage items
8. `object_breaking` — handle tool impacts on trees/rocks, destroy and drop items
9. `chest_interaction` — open chest UI when player interacts with `StorageChest`

## Does NOT Handle
- Farm soil/crop rendering (farming domain)
- NPC placement on maps (npcs domain)
- Mine floor generation (mining domain)
- Shop interiors functionality (economy domain)
- Player spawning position per map (player domain)
