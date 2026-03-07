# Worker: Fix Animal Atlas Slicing Mismatch

## Context
`assets/sprites/character_spritesheet.png` is 192×192 pixels (4 columns × 4 rows of 48×48 frames).
But `src/animals/mod.rs` lines 115-123 slice it as if it were 192×256 (12 cols × 16 rows of 16×16 frames).
This means the last 4 rows sample beyond the image bounds, producing broken frames for sheep/cat/dog.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/animals/`
All out-of-scope edits will be reverted.

## Required reading
1. src/animals/mod.rs — the `load_animal_sprites` function, lines ~114-130
2. src/shared/mod.rs — DO NOT MODIFY, import types from here

## Task

In `src/animals/mod.rs`, find the block starting with the comment:
```rust
// Sheep, cat, dog: reuse character_spritesheet.png with tint colors.
// character_spritesheet.png: 192x256, 12 cols x 16 rows of 16x16 frames.
```

Replace that comment and the TextureAtlasLayout::from_grid call with:
```rust
// Sheep, cat, dog: reuse character_spritesheet.png with tint colors.
// character_spritesheet.png: 192x192, 4 cols x 4 rows of 48x48 frames.
let sheet = asset_server.load("sprites/character_spritesheet.png");
let sheet_layout = layouts.add(TextureAtlasLayout::from_grid(
    UVec2::new(48, 48),
    4,
    4,
    None,
    None,
));
```

The rest of the code (sheep_image, sheep_layout, cat_image, etc.) stays the same.

## Also check
In `src/animals/spawning.rs`, check if any sprite index references assume the old 12×16 grid.
If you find references like `atlas.index = 0` for sheep, they should still work (frame 0 is frame 0).
But if you find indices > 15 (the old grid had 192 frames, new has 16), clamp them to valid range 0-15.

## Do NOT
- Modify src/shared/mod.rs
- Modify any files outside src/animals/
- Change the animal spawning logic or behavior
- Add new features

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write a brief report to status/workers/fix-animal-atlas.md listing:
- Lines changed
- Old vs new atlas dimensions
- Any sprite index adjustments made in spawning.rs
