# Domain: General Store Interior

## Map: GeneralStore (12×12 grid)
Pierre-style general store. Sells seeds, basic supplies. Warm and rustic.

## Layout Requirements

### Walls
- Full Void perimeter on rows 0, 11 and columns 0, 11
- Door opening: x=5,6 y=11 (south wall, WoodFloor) — exit to Town

### Room Zones
1. **Shop Counter** (y=3, x=3-8, Stone tiles):
   - Counter runs east-west, separating shopkeeper area from customer area
   - Shopkeeper (NPC) stands behind counter at ~(5,2)
   - Counter furniture: idx 32 and 33 along the line

2. **Behind Counter / Shopkeeper Area** (y=1-2, x=1-10):
   - Shelves along north wall: idx 9, 10, 11 repeated at y=1
   - Extra stock crates: idx 36 at corners

3. **Customer Area** (y=4-10, x=1-10):
   - Open floor for walking
   - Display shelves on left wall: idx 9 at (1,5), idx 18 at (1,6), idx 9 at (1,8)
   - Display shelves on right wall: idx 10 at (10,5), idx 18 at (10,6), idx 10 at (10,8)
   - Small rug at entrance: idx 48 at (5,10) and (6,10)
   - Crate near door: idx 36 at (2,9) and (9,9)
   - Barrel: idx 27 at (1,9)
   - Plant decor: idx 22 at (10,4) and (1,4)

### Transition
```rust
MapTransition {
    from_map: MapId::GeneralStore,
    from_rect: (5, 11, 2, 1),
    to_map: MapId::Town,
    to_pos: (8, 5),
}
```

## Furniture Count Target: 15-18 pieces
## Output: generate_general_store() + general_store_furniture()
Remember: tiles[y * width + x], door at y=11 (south/bottom exit per existing code).
