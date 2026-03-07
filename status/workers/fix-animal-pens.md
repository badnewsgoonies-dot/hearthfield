# Worker Report: FIX-ANIMAL-PENS (Spawn Bounds)

## Files Modified
- `src/animals/spawning.rs` — updated `pen_bounds_for()` (lines 76–84)

## What Was Implemented
Replaced hardcoded negative world coordinates in `pen_bounds_for()` with
tile-relative coordinates using `TILE_SIZE` (16.0), placing all pens within
the valid farm bounds [0, 512] × [0, 384]:

| Animals | Tiles | World coords |
|---------|-------|--------------|
| Coop (Chicken/Duck/Rabbit) | (8–12, 16–18) | (128, 256) → (192, 288) |
| Barn (Cow/Sheep/Goat/Pig) | (3–7, 16–18) | (48, 256) → (112, 288) |
| Companions (Horse/Cat/Dog) | (10–20, 8–16) | (160, 128) → (320, 256) |

## Shared Type Imports Used
- `TILE_SIZE` (already imported in file)
- `AnimalKind` (already imported in file)

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (88 passed, 0 failed)
- `cargo clippy -- -D warnings` — PASS

## Known Risks
None. Change is isolated to coordinate constants in a single function.
