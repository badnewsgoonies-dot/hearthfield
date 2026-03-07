# Worker: ANIMALS — Sprite System for Remaining Animal Types (C3)

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/animals/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/animals/mod.rs — current AnimalSpriteData resource, load_animal_sprites(), animate_animal_sprites()
2. src/animals/spawning.rs — current spawn logic, see how chicken/cow use atlas sprites vs sheep/cat/dog using character_spritesheet with tints vs others using colored rectangles
3. src/shared/mod.rs — AnimalKind enum, WanderAi, AnimalAnimTimer

## Context
Currently:
- Chicken: sprites/chicken.png (64x32, 4×2 grid of 16×16 frames) ✓
- Cow: sprites/cow.png (96x64, 3×2 grid of 32×32 frames) ✓
- Sheep/Cat/Dog: Reuse character_spritesheet.png (192x192, 4×4 grid of 48×48) with color tints — looks like tinted humans
- Duck/Rabbit/Pig/Goat/Horse: Plain colored rectangles via fallback `_ =>` branch

No dedicated sprite assets exist for sheep/cat/dog/duck/rabbit/pig/goat/horse. We cannot load assets that don't exist.

## Deliverables

### 1. Give sheep/cat/dog their own distinct visual identity
Instead of reusing the 48×48 human character spritesheet, create **procedural colored sprites** using Bevy's built-in Sprite with custom_size and distinct colors/shapes:
- Sheep: 20×16 white sprite (wider = woolly)
- Cat: 12×12 orange sprite (small)
- Dog: 14×14 brown sprite (medium)

These are still simple but at least they're the right size and won't look like tinted humans.

### 2. Give duck/rabbit/pig/goat/horse distinct colored rectangles with appropriate sizes
Replace the generic `vis.color`/`vis.width`/`vis.height` fallback with explicit per-kind sizing:
- Duck: 10×10 yellow
- Rabbit: 8×10 light gray
- Pig: 22×18 pink
- Goat: 18×18 white-gray
- Horse: 24×20 dark brown

### 3. Ensure all animal kinds get AnimalAnimTimer
Currently only chicken and cow get AnimalAnimTimer for walk animation. Add it for all AnimalKind variants so animate_animal_sprites works. For non-atlas sprites (no walk frames), the animation system should gracefully handle entities without TextureAtlas — either skip them or use a simple bob/sway.

### 4. Add a simple "bob" animation for non-atlas animals
For animals without atlas frames (sheep, cat, dog, duck, rabbit, pig, goat, horse), implement a simple vertical bob animation (±1 pixel oscillation) when they're moving, to show they're alive and walking. Use the existing AnimalAnimTimer tick. This should modify Transform translation.y, not sprite index.

## Quantitative targets
- All 10 AnimalKind variants must have explicit sprite creation (no generic `_ =>` fallback)
- All 10 must have AnimalAnimTimer inserted at spawn
- Non-atlas animals must visually animate when moving (bob)
- Zero clippy warnings

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/animals_sprites_wave8.md
