# Interior Implementation Contract

## Coordinate System
- Origin (0,0) is BOTTOM-LEFT
- Y increases upward (north)
- Row-major storage: `tiles[y * width + x]`
- `grid_to_world_center(x, y)` converts grid→world coords

## TileKind Enum (src/shared/mod.rs)
```
Grass, Dirt, TilledSoil, WateredSoil, Water, Sand, Stone, WoodFloor, Path, Bridge, Void,
WallNorth, WallSouth, WallEast, WallWest  // NEW — rendered as dark wood, solid
```

## MapDef Struct
```rust
pub struct MapDef {
    pub id: MapId,
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileKind>,      // tiles[y * width + x]
    pub transitions: Vec<MapTransition>,
    pub objects: Vec<ObjectPlacement>,
    pub forage_points: Vec<(i32, i32)>,
}
```

## MapTransition Struct
```rust
pub struct MapTransition {
    pub from_map: MapId,
    pub from_rect: (i32, i32, i32, i32), // x, y, w, h trigger area
    pub to_map: MapId,
    pub to_pos: (i32, i32),
}
```

## FurniturePlacement Struct (src/world/objects.rs, crate-private)
```rust
struct FurniturePlacement {
    x: i32,
    y: i32,
    index: usize,    // atlas index in furniture.png (9 cols × 6 rows)
    wide: bool,      // if true, spans 2 tiles
}
```

## Furniture Atlas Index Map (furniture.png: 144×96, 9 cols × 6 rows)
Row 0 (idx 0-5): Beds and chairs
- 0: table-left, 1: table-right, 2: bed-top-left, 3: bed-top-right
- 4: chair, 5: small-stool

Row 1 (idx 9-14): Shelves and cabinets  
- 9: shelf-top, 10: cabinet-top, 11: cabinet-top-right
- 12: shelf-green-top, 13: cabinet-dark-top, 14: cabinet-dark-right

Row 2 (idx 18-25): Large furniture
- 18: bookshelf/furnace, 19: anvil/workbench, 20: large-cabinet
- 21: large-dresser, 22: small-plant, 23: potted-plant
- 24: lamp/lantern, 25: small-decor

Row 3 (idx 27-34): Containers and misc
- 27: barrel, 28: barrel-variant, 29: barrel-dark
- 30: rug-large, 31: small-box, 32: counter-section
- 33: counter-right, 34: small-item

Row 4 (idx 36-38): Crates
- 36: crate, 37: crate-variant, 38: crate-dark

Row 5 (idx 45-53): Rugs and mats
- 45: hay-bale, 46: hay-bale-variant, 47: hay-bale-dark
- 48-53: colorful rugs/mats (pink, blue, green variants)

## Wall Rendering Rules
- WallNorth/WallSouth/WallEast/WallWest tiles are SOLID (collision)
- They render using house_walls.png tileset (5 cols × 3 rows, 16×16 tiles)
  - Index 0-4: top wall variants
  - Index 5-9: middle wall variants  
  - Index 10-14: bottom wall variants
- Void tiles in interiors render as black (for corners/outside)

## Door Rendering
- doors.png: 1 col × 4 rows (16×16 tiles), indices 0-3
- Door tiles should be WoodFloor (walkable) with a door furniture sprite on top

## Interior Design Principles
1. Perimeter = wall tiles (WallNorth on top row, WallSouth on bottom, etc.)
2. Interior = WoodFloor (most rooms), Stone (blacksmith forge area)
3. Door opening = 2-tile-wide WoodFloor gap in south wall
4. Furniture placed against walls, not blocking walkways
5. Each room should have a clear function zone
6. All solid furniture tiles must also be in solid_tiles or the wall tiles handle it

## Output Format
Each worker produces a single .rs file containing exactly two functions:
```rust
// Function 1: map generator
pub fn generate_{name}() -> MapDef { ... }

// Function 2: furniture placements  
pub fn {name}_furniture() -> Vec<FurniturePlacement> { ... }
```

## Required Imports (copy these exactly)
```rust
use crate::shared::*;
```

## Validation
- All tiles within bounds: x in 0..width, y in 0..height
- Door transition rect must overlap walkable tiles
- No furniture placed on Void/wall tiles
- Furniture indices must be valid (0-53)
