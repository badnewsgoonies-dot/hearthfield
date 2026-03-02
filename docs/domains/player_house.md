# Domain: Player House Interior

## Map: PlayerHouse (16×16 grid)
The player's home. Should feel cozy and lived-in, like a Harvest Moon/Stardew starter house.

## Layout Requirements

### Walls (Void tiles on perimeter)
- Full Void perimeter on rows 0, 15 and columns 0, 15
- EXCEPT: door opening at x=7,8 y=0 (2 tiles wide, WoodFloor) — this is the exit

### Room Zones (all interior tiles are WoodFloor unless specified)
1. **Bedroom** (upper-right, roughly x=10-14, y=10-14):
   - Bed: 2 tiles wide using idx 2+3 at (12,13) 
   - Nightstand: idx 31 at (11,13)
   - Dresser: idx 21 at (14,11)
   - Rug: idx 48 or 49 under bed area

2. **Kitchen** (upper-left, roughly x=1-6, y=10-14):
   - Stone floor tiles for kitchen counter area (y=13-14, x=1-5)
   - Counter: idx 32+33 along north wall at (2,14) and (3,14)
   - Barrel (pantry): idx 27 at (1,12)
   - Table: idx 0+1 at (4,11) and (5,11)
   - Stool: idx 5 at (4,10) and (5,10)

3. **Living Room** (center, x=4-11, y=4-9):
   - Path tiles for rug area (x=6-9, y=5-8)
   - Fireplace: Stone tiles at (7,14) and (8,14) on north wall interior
   - Bookshelf: idx 9 (top) at (1,13), idx 18 (bottom) at (1,12)
   - Chair: idx 4 at (6,6) and (9,6) flanking rug
   - Lamp: idx 24 at (6,9) 

4. **Entry Area** (bottom, y=1-3):
   - Clear path from door (7,0)/(8,0) to interior
   - Small mat/rug: idx 48 at (7,1) and (8,1)
   - Barrel near door: idx 27 at (2,2)

### Transition
```rust
MapTransition {
    from_map: MapId::PlayerHouse,
    from_rect: (7, 0, 2, 1),  // door tiles at bottom
    to_map: MapId::Farm,
    to_pos: (31, 2),
}
```

### Bed Interaction
The bed at (12,13)/(13,13) is where the player sleeps. The existing sleep system uses the bed position. Keep bed at a consistent location.

## Furniture Count Target: 15-20 pieces
## Objects: none (interior, no trees/rocks)
## Forage Points: none

## Output
Write two functions in a single file:
1. `pub fn generate_player_house() -> MapDef`
2. `pub fn player_house_furniture() -> Vec<FurniturePlacement>`

Use the FurniturePlacement struct:
```rust
struct FurniturePlacement {
    x: i32,
    y: i32,
    index: usize,
    wide: bool,
}
```

Remember: tiles[y * width + x], y=0 is BOTTOM. Door is at y=0 (south wall).
