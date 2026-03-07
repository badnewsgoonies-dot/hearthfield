# Worker Completion Report: ANIMALS — Sprite System (C3)

## Files Modified
- `src/animals/mod.rs` (210 lines)
- `src/animals/spawning.rs` (489 lines)

## What Was Implemented

### 1. AnimalSpriteData simplified
Removed `sheep_image`, `sheep_layout`, `cat_image`, `cat_layout`, `dog_image`, `dog_layout` fields — no dedicated sprite assets exist for those kinds.

### 2. load_animal_sprites cleaned up
Removed character_spritesheet.png atlas loading for sheep/cat/dog (was producing tinted human sprites).

### 3. AnimalVisual struct simplified
Removed unused `color` and `width` fields (only `height` is consumed for name-tag placement). All 10 kinds have correct heights matching their sprite sizes.

### 4. Explicit per-kind sprite creation — no `_ =>` fallback
All 10 AnimalKind variants now have explicit `Sprite` construction in `handle_animal_purchase`:
- **Chicken**: atlas 16×16 (sprite_data.loaded) else yellow rect
- **Cow**: atlas 32×32 (sprite_data.loaded) else gray rect
- **Sheep**: 20×16 white (#f8f8f8)
- **Cat**: 12×12 orange (#e68c33)
- **Dog**: 14×14 brown (#9961 2e)
- **Duck**: 10×10 yellow (#f2e01a)
- **Rabbit**: 8×10 light gray (#cccccc)
- **Pig**: 22×18 pink (#f2b3ba)
- **Goat**: 18×18 white-gray (#d9d9c7)
- **Horse**: 24×20 dark brown (#592010)

### 5. AnimalAnimTimer on all 10 kinds
All animals get `AnimalAnimTimer` at spawn:
- Chicken: frame_count=4, period=0.2s
- Cow: frame_count=3, period=0.25s
- All others: frame_count=2, period=0.3s

### 6. animate_animal_sprites: timer.reset() when idle
When an animal stops moving, `anim.timer.reset()` is called so `elapsed_secs()` returns 0 — ensuring the bob offset is 0 at rest.

### 7. bob_non_atlas_animals (PostUpdate)
New system registered in PostUpdate `.after(crate::world::ysort::sync_position_and_ysort)`. For non-atlas animals that are moving, applies `sin(elapsed/period * 2π)` ±1px vertical bob to `transform.translation.y`. Atlas animals (chicken, cow) are skipped via `if sprite.texture_atlas.is_some() { continue; }`.

## Quantitative Targets
| Target | Result |
|--------|--------|
| All 10 AnimalKind variants explicit (no `_ =>`) | ✅ 10/10 |
| All 10 have AnimalAnimTimer at spawn | ✅ 10/10 |
| Non-atlas animals animate when moving | ✅ bob via PostUpdate |
| Zero clippy warnings | ✅ 0 warnings |

## Validation Results
```
cargo check          → Finished (0 errors, 0 warnings)
cargo test --test headless → 88 passed, 0 failed
cargo clippy -- -D warnings → Finished (0 errors, 0 warnings)
shasum -a 256 -c .contract.sha256 → src/shared/mod.rs: OK
```

## Known Risks
- Bob system runs in PostUpdate after ysort sync. If WorldPlugin is ever removed or ysort scheduling changes, the `.after(sync_position_and_ysort)` ordering constraint would need updating.
- Pre-existing `src/world/mod.rs` type_complexity clippy issue appeared transiently in working tree (from another wave worker); reverts cleanly with clamp-scope.sh.
