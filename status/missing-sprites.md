# Missing Animal Sprites

**Date:** 2026-03-19  
**Checked by:** Copilot audit of `src/animals/spawning.rs` + `assets/sprites/`

---

## Summary

`horse.png` and `cat.png` are absent from `assets/sprites/`. Both animals render
as solid-color rectangles (fallback path). No entries exist for them in
`AnimalSpriteData` and `load_animal_sprites` does not load them.

---

## Current Fallback Rendering

| Animal | Fallback color | Size |
|--------|---------------|------|
| Horse  | `srgb(0.35, 0.20, 0.10)` (dark brown) | 24 × 20 px |
| Cat    | `srgb(0.90, 0.55, 0.20)` (orange)     | 12 × 12 px |

Code path: `src/animals/spawning.rs` lines 319–335 — plain `Sprite { color, custom_size, .. }`, no `TextureAtlas`.

---

## What Is Needed

### `assets/sprites/horse.png`

Recommended format (matching existing barn animals):

- **Layout:** 24 cols × 4 rows sprite sheet  
- **Frame size:** 48 × 48 px (matching cow; horse is large)  
- **Sheet size:** 1152 × 192 px  
- **Rows:** Down (0), Left (1), Right (2), Up (3) — 6 walk frames each  

### `assets/sprites/cat.png`

Recommended format (matching existing coop animals):

- **Layout:** 24 cols × 4 rows sprite sheet  
- **Frame size:** 16 × 16 px (matching chicken; cat is small)  
- **Sheet size:** 384 × 64 px  
- **Rows:** Down (0), Left (1), Right (2), Up (3) — 6 walk frames each  

---

## Code Changes Required After Sprites Are Added

1. **`src/animals/mod.rs` — `AnimalSpriteData`**: add fields  
   ```rust
   pub horse_image: Handle<Image>,
   pub horse_layout: Handle<TextureAtlasLayout>,
   pub cat_image: Handle<Image>,
   pub cat_layout: Handle<TextureAtlasLayout>,
   ```

2. **`src/animals/mod.rs` — `load_animal_sprites`**: load the new sheets  
   ```rust
   // horse.png: 1152×192, 24 cols × 4 rows of 48×48 frames
   sprite_data.horse_image = asset_server.load("sprites/horse.png");
   sprite_data.horse_layout = layouts.add(TextureAtlasLayout::from_grid(
       UVec2::new(48, 48), 24, 4, None, None,
   ));

   // cat.png: 384×64, 24 cols × 4 rows of 16×16 frames
   sprite_data.cat_image = asset_server.load("sprites/cat.png");
   sprite_data.cat_layout = layouts.add(TextureAtlasLayout::from_grid(
       UVec2::new(16, 16), 24, 4, None, None,
   ));
   ```

3. **`src/animals/spawning.rs` — `AnimalKind::Horse` branch** (~line 319): replace plain `Sprite` with `Sprite::from_atlas_image(sprite_data.horse_image.clone(), TextureAtlas { layout: sprite_data.horse_layout.clone(), index: 0 })` and set `custom_size = Some(Vec2::new(48.0, 48.0))`.

4. **`src/animals/spawning.rs` — `AnimalKind::Cat` branch** (~line 328): same pattern with `sprite_data.cat_image` / `sprite_data.cat_layout` and `custom_size = Some(Vec2::new(16.0, 16.0))`.

---

## Reference: Dog (working example)

Dog uses `dog.png` (1152 × 416, 24 × 13 frames of 48 × 32) and is fully wired
in `AnimalSpriteData`, `load_animal_sprites`, and the spawn match arm. Horse and
Cat should follow the same pattern once sprites are available.
