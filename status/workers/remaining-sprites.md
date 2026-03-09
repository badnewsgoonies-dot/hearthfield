# Worker Report: Wire Remaining Sprites

## Files Modified
- `src/animals/mod.rs` — added `egg_nest_image`, `egg_nest_layout`, `milk_grass_image`, `milk_grass_layout` fields to `AnimalSpriteData`; added loading code in `load_animal_sprites`
- `src/animals/products.rs` — added `Res<AnimalSpriteData>` parameter to `update_product_indicators`; replaced plain colored-square product indicators with atlas sprites for chicken/duck (egg) and cow/goat (milk)

## Assets Copied
- `assets/sprites/_source_limezu/egg_and_nest.png` -> `assets/sprites/egg_and_nest.png` (64x16, 4 frames of 16x16)
- `assets/sprites/_source_limezu/milk_and_grass.png` -> `assets/sprites/milk_and_grass.png` (64x16, 4 frames of 16x16)

## What Was Wired
- **egg_and_nest.png**: Used as product-ready indicator sprite for Chicken and Duck animals (frame 0 = egg). Replaces the previous 6x6 yellow/green colored squares with a 10x10 atlas sprite showing an actual egg graphic.
- **milk_and_grass.png**: Used as product-ready indicator sprite for Cow and Goat animals (frame 0 = milk bucket). Replaces the previous 6x6 white/cream colored squares with a 10x10 atlas sprite showing an actual milk bucket graphic.

## What Was Skipped (with reasons)

### door_anim.png (224x32, 7 frames of 32x32)
**Skipped.** Current doors use `tilesets/doors.png` (16x64, 1 col x 4 rows of 16x16 tiles) as static sprites in the building overlay system. The animated door sprite uses 32x32 frames (2x larger), and animating doors would require a new proximity-based animation system and a component to track door state. Too invasive for a sprite-wiring task; the current static doors work correctly.

### Individual tree sprites (tree_oak_green.png, tree_oak_brown.png, tree_birch_green.png, tree_pine_blue.png)
**Skipped.** Trees are already well-served by `tree_sprites.png` (128x96 atlas, 32x48 cells, 4 seasonal variants x 2 tree types) plus seasonal tinting from `seasonal.rs` (variant_a/variant_b hash-based color variety). The individual tree files have inconsistent dimensions (80x96, 48x80, 64x96) that don't match each other or the atlas cell size, making them unsuitable as drop-in replacements.

### modern_farm_fences.png (512x272)
**Skipped.** Fences currently use `tilesets/fences.png` (64x64, 4x4 grid of 16x16 tiles) with a 4-bit cardinal-neighbor autotile system (`fence_autotile_index`). The modern fences tileset is 512x272 — a much larger tileset with a completely different layout that would require rewriting the autotile index mapping. The current fences render correctly.

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (128 passed, 0 failed, 2 ignored)
- `cargo clippy -- -D warnings` — PASS (0 warnings)

## Shared Type Imports Used
- `AnimalKind`, `AnimalProductEvent`, `ItemPickupEvent`, `PlaySfxEvent`, `ToastEvent`, `PlayerInput`, `InputBlocks`, `Player`, `LogicalPosition`, `ItemQuality`, `Z_EFFECTS` (all from `crate::shared::*`, unchanged)
