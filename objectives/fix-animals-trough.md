# Worker: FIX-ANIMALS (Feed Trough Position)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/animals/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/animals/spawning.rs (read the FULL file — the fix is here)
2. src/world/maps.rs (search for "Barn" to find barn position: x=3-7, y=16-18)
3. src/shared/mod.rs — search for FeedTrough, TILE_SIZE

## Bug: FeedTrough at Unreachable Position

### Root Cause
`setup_feed_trough` in `src/animals/spawning.rs:421-433` spawns the FeedTrough at grid (-10, -8) with world position (-160, -128). The Farm map is 32x24 (x: 0-31, y: 0-23). Grid (-10, -8) is completely outside map bounds — the player can never reach it.

The barn is at tiles (3, 16) to (7, 18). The trough should be at the barn entrance, around grid (5, 19) — just south of the barn on the connecting path.

### Fix Required
In `src/animals/spawning.rs`, in `setup_feed_trough`, change:
```rust
// FROM:
super::FeedTrough {
    grid_x: -10,
    grid_y: -8,
},
// ...
Transform::from_xyz(-160.0, -128.0, Z_ENTITY_BASE),
LogicalPosition(Vec2::new(-160.0, -128.0)),

// TO:
super::FeedTrough {
    grid_x: 5,
    grid_y: 19,
},
// ...
Transform::from_xyz(5.0 * TILE_SIZE, 19.0 * TILE_SIZE, Z_ENTITY_BASE),
LogicalPosition(Vec2::new(5.0 * TILE_SIZE, 19.0 * TILE_SIZE)),
```

Also update the comment from "Grid position (-10, -8)" to "Grid position (5, 19) — south of barn entrance".

TILE_SIZE is 16.0, so world position = (80.0, 304.0).

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-animals-trough.md
