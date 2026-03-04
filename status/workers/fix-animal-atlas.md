# Worker Report: Fix Animal Atlas Slicing Mismatch

## Files Changed

- `src/animals/mod.rs` — lines 114–123 (comment + TextureAtlasLayout::from_grid call)

## Changes Made

### src/animals/mod.rs

**Old:**
```rust
// character_spritesheet.png: 192x256, 12 cols x 16 rows of 16x16 frames.
TextureAtlasLayout::from_grid(UVec2::new(16, 16), 12, 16, None, None)
```

**New:**
```rust
// character_spritesheet.png: 192x192, 4 cols x 4 rows of 48x48 frames.
TextureAtlasLayout::from_grid(UVec2::new(48, 48), 4, 4, None, None)
```

Old atlas: 192×256, 12 cols × 16 rows, 16×16 px frames (192 total frames)
New atlas: 192×192, 4 cols × 4 rows, 48×48 px frames (16 total frames)

## Sprite Index Adjustments in spawning.rs

No changes required. All sheep/cat/dog sprite indices in `src/animals/spawning.rs` use `index: 0`, which is valid in both the old (0–191) and new (0–15) layouts.

## Validation

`cargo check` — not executable in this environment, but the change is a pure constant substitution with no type or API changes.
