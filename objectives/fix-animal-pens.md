# Worker: FIX-ANIMAL-PENS (Spawn Bounds)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/animals/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/animals/spawning.rs (read the FULL file — pen_bounds_for is here)
2. src/world/maps.rs (search for "Farm" to find farm map dimensions: 32x24 tiles)
3. src/shared/mod.rs — search for TILE_SIZE (= 16.0), AnimalKind

## Bug: Animal Pens Outside Farm Map Bounds

### Root Cause
`pen_bounds_for()` in `src/animals/spawning.rs` returns pen bounds in negative world coordinates:
- Coop animals (Chicken, Duck, Rabbit): (-96, -192) to (96, -96)
- Barn animals (Cow, Sheep, Goat, Pig): (-192, -192) to (-32, -64)
- Companions (Horse, Cat, Dog): (-256, -256) to (256, 256)

The farm map is 32×24 tiles = 512×384 world units, spanning [0, 512] × [0, 384].
All animal pens are partially or fully outside the farm map bounds.

The barn is at tiles (3-7, 16-18) and the coop should be nearby.

### Fix Required
Update `pen_bounds_for()` to use coordinates WITHIN the farm map:

```rust
fn pen_bounds_for(kind: &AnimalKind) -> (Vec2, Vec2) {
    match kind {
        // Coop animals: area east of barn, tiles (8-12, 16-18)
        AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit => {
            (Vec2::new(8.0 * TILE_SIZE, 16.0 * TILE_SIZE),
             Vec2::new(12.0 * TILE_SIZE, 18.0 * TILE_SIZE))
        }
        // Barn animals: inside/around barn, tiles (3-7, 16-18)
        AnimalKind::Cow | AnimalKind::Sheep | AnimalKind::Goat | AnimalKind::Pig => {
            (Vec2::new(3.0 * TILE_SIZE, 16.0 * TILE_SIZE),
             Vec2::new(7.0 * TILE_SIZE, 18.0 * TILE_SIZE))
        }
        // Companions: follow player around farm, tiles (10-20, 8-16)
        AnimalKind::Horse | AnimalKind::Cat | AnimalKind::Dog => {
            (Vec2::new(10.0 * TILE_SIZE, 8.0 * TILE_SIZE),
             Vec2::new(20.0 * TILE_SIZE, 16.0 * TILE_SIZE))
        }
    }
}
```

These coordinates place:
- Coop: east of barn at (128, 256) to (192, 288)
- Barn animals: at barn area (48, 256) to (112, 288)
- Companions: central farm area (160, 128) to (320, 256)

All within the valid farm bounds [0, 512] × [0, 384].

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-animal-pens.md
