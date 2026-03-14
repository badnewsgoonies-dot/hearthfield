# FoMT Farm Layout Spec — Hearthfield Reconstruction

## Source: Harvest Moon: Friends of Mineral Town (GBA, 2003)

## FoMT Farm Layout (canonical)

The FoMT farm is roughly rectangular, wider than tall. Buildings are arranged around the perimeter with a large open field in the center. The layout from the GBA game:

```
NORTH EDGE (exit to mountain/forest path)
+------------------------------------------------------------------+
|                                                                    |
|  [BARN]          [HORSE STABLE]  [WOOD BIN]      [CHICKEN COOP]  |
|  (upper-left)    (north-center)  (near stable)   (upper-right)   |
|                                                                    |
|                                                                    |
|              LARGE OPEN FIELD (tillable)                           |
|              (center of farm, ~70% of area)                        |
|              scattered with rocks, stumps, weeds                   |
|              at game start                                         |
|                                                                    |
|                                                                    |
|                                                                    |
|  [PLAYER HOUSE]                              [SHIPPING BIN]       |
|  (lower-left)                                (near south exit)    |
|  door faces RIGHT                                                  |
|  [DOG HOUSE] next to house                   [MAILBOX]           |
|                                              (near house/path)    |
|                                                                    |
+------------------------------------------------------------------+
SOUTH EDGE (exit to Mineral Town / Poultry Farm / Yodel Ranch)
WEST EDGE (exit to Gotz's house / forest)
EAST EDGE (blocked by fence, no exit)
```

### Key spatial relationships:
- **Player House**: LOWER-LEFT corner, door faces east/right
- **Barn**: UPPER-LEFT corner, door faces south  
- **Chicken Coop**: UPPER-RIGHT area, door faces south
- **Horse Stable**: North-center, between barn and coop
- **Wood/Lumber Bin**: Near horse stable
- **Shipping Bin**: LOWER-RIGHT area, near south exit path
- **Mailbox**: Near player house, along the path
- **Dog House**: Adjacent to player house
- **Large tillable field**: CENTER of the farm, most of the area
- **Pond**: Not on the farm in FoMT (pond is at the mountain)
- **South exit**: Path leading to town (main exit)
- **West exit**: Path to Gotz's house / mountain area
- **North exit**: Not a direct exit in FoMT (fence)

### Critical differences from current Hearthfield:
1. **House position MOVES from top-center to bottom-left** (biggest change)
2. **Barn MOVES from bottom-left to upper-left**
3. **Chicken coop MOVES from bottom-left to upper-right**
4. **Shipping bin MOVES from upper-right to lower-right**
5. **Pond REMOVED from farm** (move to forest/mountain map)
6. **Field becomes center-dominant** instead of center-bottom
7. **South exit to town is the MAIN exit** (currently exists)
8. **West exit to forest/mine** (currently exists)
9. **House door faces east** not south

## Hearthfield Current Layout (32x24 grid, 16px tiles)

```
Current:
- Player House: grid (13,0) top-center, 6x3, door at (15,2) facing south
- Barn: grid (3,16) bottom-left, 5x3, door at (5,16)
- Chicken Coop: grid (9,17) bottom-left, 3x2, door at (10,17)
- Shipping bin: grid (14,6) right of house path
- Crafting bench: grid (12,6)
- Pond: grid (24,17) bottom-right, 5x4
- Central field (tillable): grid (6,6) to (25,15), 20x10
- Path south to town: grid (14,16) 3-wide going south
- Path east to forest: grid (26,9) going east
- Path west to mine: grid (0,9) going west
- North exit to mountain
```

## Target Layout (FoMT-style, mapped to 32x24 grid)

```
Target:
Row 0-4 (top/north):
  - Barn: grid (2,1), 5x3, door at (4,3) facing south
  - Horse area: grid (13,2), 3x2 (use fence objects)
  - Wood bin area: grid (17,2), 2x2
  - Chicken Coop: grid (24,1), 3x2, door at (25,2) facing south

Row 4-16 (center):
  - LARGE TILLABLE FIELD: grid (4,5) to (27,16), 24x12
  - (this is the heart of the farm — mostly Dirt tiles)
  - Path from house to field: grid (6,17) to (6,5), 2-wide going north

Row 17-23 (bottom/south):
  - Player House: grid (2,17), 6x3, door at (7,19) facing east/right
  - Dog house: grid (9,18), 1x1 object
  - Mailbox: grid (9,17), 1x1 object
  - Shipping Bin: grid (24,20), 2x2
  - Path south to town: grid (14,21) 3-wide going south

Exits:
  - South: town (keep existing)
  - West: forest/mine (keep existing, adjust y to row ~10)
  - North: mountain (keep existing)
  - East: forest (keep existing, adjust y to row ~10)
```

## Mapping FoMT structures to Hearthfield types

| FoMT Structure | Hearthfield Type | Notes |
|---|---|---|
| Player House | BuildingDef with BuildingImage::Farmhouse | Move to (2,17), door at (7,19) |
| Barn | BuildingDef with BuildingImage::Barn | Move to (2,1), door at (4,3) |
| Chicken Coop | BuildingDef with BuildingImage::ChickenHouse | Move to (24,1), door at (25,2) |
| Horse Stable | Fence objects in a rectangle | No BuildingImage for stable |
| Wood Bin | WorldObjectKind if available, else Log objects | Near stable |
| Shipping Bin | Keep existing spawn_shipping_bin, change grid pos | Move to (24,20) |
| Crafting Bench | Keep existing spawn_crafting_bench, change grid pos | Move near house |
| Mailbox | Object near house | grid (9,17) |
| Dog House | Object near house | grid (9,18) |
| Pond | REMOVE from farm | Was at (24,17) |

## Files that need changes

1. **src/world/maps.rs** — `generate_farm()`: tile layout (biggest change)
2. **src/world/objects.rs** — `farm_buildings()`: building positions and doors
3. **src/world/objects.rs** — `spawn_shipping_bin()`: new grid position
4. **src/world/objects.rs** — `spawn_crafting_bench()`: new grid position  
5. **src/world/map_data.rs** — `DoorDef` for Farm: house door position
6. **src/world/map_data.rs** — `EdgeDefs` for Farm: exit positions (may need adjustment)
7. **src/world/map_data.rs** — `MapTransition` entries: spawn positions after transition

## What NOT to change
- Do NOT modify src/shared/mod.rs (no new enums)
- Do NOT change interior maps (PlayerHouse, Barn interior, etc.)
- Do NOT change the town layout
- Do NOT change game mechanics (farming, animals, fishing)
- Do NOT change the save system
