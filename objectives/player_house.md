You are implementing the Player House interior for a Harvest Moon-style farming game in Rust/Bevy.

## Your Task
Rewrite the `generate_player_house()` function and `player_house_furniture()` function to create a cozy, detailed interior.

## Step 1: Read these files for context
- src/world/maps.rs — find the existing `generate_player_house()` function (around line 659). Study its structure.
- src/world/objects.rs — find the existing `player_house_furniture()` function (around line 1415). Study its structure.
- docs/domains/player_house.md — your design spec
- docs/interior_contract.md — shared types and furniture atlas index map

## Step 2: Write your output to status/workers/player_house_impl.rs

The file must contain exactly these two functions (with all necessary type annotations):

```rust
// Replacement for generate_player_house() in src/world/maps.rs
// Map is 16x16. tiles[y * width + x]. y=0 is south/bottom, y=15 is north/top.
// Door exit is at y=15 (top row), x=7 and x=8.
// Transition: from_rect (7, 15, 2, 1) -> Farm at (31, 2)
```

### Map Design (16×16):
- Perimeter: Void walls (y=0, y=15, x=0, x=15) EXCEPT door at (7,15) and (8,15) = WoodFloor
- **Bedroom** upper-right (x=10-14, y=10-13): WoodFloor with bed, nightstand, dresser
- **Kitchen** upper-left (x=1-6, y=10-14): Mix of Stone (counters at y=13-14) and WoodFloor  
- **Living Room** center (x=3-12, y=4-9): WoodFloor with Path rug area (x=6-9, y=5-8)
- **Fireplace** at bottom center wall: Stone tiles at (6,1) to (9,2) 
- **Entry hall** (y=12-14 center): clear walkway from door down into rooms
- Player enters at y=14 (one tile below door row)

### Furniture placements (target 18+ pieces):
Use FurniturePlacement { x, y, index, wide } struct.
Furniture atlas indices (furniture.png, 9 cols × 6 rows):
- 0/1: table halves (left/right)
- 2/3: bed halves (left/right) 
- 4: chair
- 5: small stool
- 9/10/11: shelf/cabinet tops
- 18: bookshelf/furnace base
- 19: anvil/workbench
- 21: large dresser
- 22: small plant
- 24: lamp/lantern
- 27: barrel
- 30: large rug
- 31: small box
- 32/33: counter sections
- 36: crate
- 48-53: colorful rugs/mats

Place furniture to make it feel like a lived-in home:
- Bedroom: bed (2+3), nightstand (31), dresser (21), rug (48 or 49)
- Kitchen: counters (32, 33), barrel (27), table (0+1), stools (5)
- Living Room: chairs (4), lamp (24), bookshelf (9 top + 18 bottom), plant (22)
- Entry: welcome mat (48), coat barrel (27)
- Fireplace area: use Stone tiles in the map, no furniture needed

## Step 3: Write completion report to status/workers/player_house.md
Include: files created, furniture count, what was implemented.

## CRITICAL RULES
- Do NOT modify any existing source files. Only create new files.
- Do NOT edit src/world/maps.rs or src/world/objects.rs
- The output file must be valid Rust function bodies (will be copy-pasted into the real source)
- Match the exact function signatures from the existing code
- tiles array is Vec<TileKind> with row-major layout: tiles[y * width + x]
- Use TileKind::Void, TileKind::WoodFloor, TileKind::Stone, TileKind::Path, TileKind::Dirt
- Use MapId::PlayerHouse, MapId::Farm
- Make the room feel FULL — no empty expanses of bare floor
