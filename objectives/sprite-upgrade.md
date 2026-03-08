# Worker: SPRITE UPGRADE — Modern Farm Terrain, Buildings, Animals, Crops

## Priority Order
1. **TERRAIN** (grass, water, dirt, sand — every pixel on screen)
2. **BUILDINGS** (houses, barns — major visual landmarks)
3. **ANIMALS** (replace colored rectangles with real sprites)
4. **CROPS** (upgrade growth stage sprites)

## Context
Modern Farm v1.2 sprite pack has been extracted. New PNGs are already in assets/.
The code must be updated to use the new sprite dimensions and atlas layouts.

**Key discovery**: `assets/tilesets/modern_farm_autotiles.png` (192×896, Godot-format
autotiles) has existed in the repo since before this session but was NEVER wired into
code. It contains proper edge-blending transitions for grass↔dirt, grass↔water,
grass↔sand, water↔sand, and tilled soil variants.

## Scope (hard allowlist)
You may modify files under:
- `src/world/` (terrain atlases, tile mapping, building sprites)
- `src/animals/` (animal sprite loading, spawning, animation)
- `src/farming/` (crop atlas loading, rendering)
- `src/data/crops.rs` (crop sprite_stages indices)

Do NOT edit `src/shared/mod.rs` (the checksummed contract).
Do NOT run `cargo fmt` on `src/shared/mod.rs`.

## Required reading (in this order)
1. `src/shared/mod.rs` — TileKind enum, AnimalKind enum, CropDef, MapId, Season
2. `src/world/mod.rs` — TerrainAtlases struct (line 180), ensure_atlases_loaded (line 197), tile_atlas_info (line 318)
3. `src/world/objects.rs` — ObjectAtlases struct (line 91), load function, house_walls/house_roof usage
4. `src/animals/mod.rs` — AnimalSpriteData (line 82), load_animal_sprites (line 91), animate_animal_sprites
5. `src/animals/spawning.rs` — spawn_animals (lines 274-375: sprite per AnimalKind)
6. `src/farming/render.rs` — FarmingAtlases, sync_crop_sprites, crop_atlas_index
7. `src/data/crops.rs` — CropDef definitions with sprite_stages

---

## DELIVERABLE 1: Terrain Atlas Upgrade (HIGHEST PRIORITY)

### New terrain asset: `assets/tilesets/modern_farm_terrain.png`
- Dimensions: 512×368 = 32 cols × 23 rows of 16×16 tiles
- Contains: grass variants, dirt, sand, water, shoreline transitions, soil types

### Autotile asset (already exists): `assets/tilesets/modern_farm_autotiles.png`
- Dimensions: 192×896 = 12 cols × 56 rows of 16×16 tiles
- Godot 3×3 minimal autotile format
- Contains edge-blending for all terrain transitions

### What to change in `src/world/mod.rs`:

**TerrainAtlases struct**: Add new fields for the Modern Farm terrain sheet:
```
pub terrain_image: Handle<Image>,
pub terrain_layout: Handle<TextureAtlasLayout>,
```

**ensure_atlases_loaded**: Load `tilesets/modern_farm_terrain.png` as 32 cols × 23 rows.
Keep the old atlas loads as fallbacks — don't remove them yet.

**tile_atlas_info**: Update the TileKind → atlas index mapping to use the new terrain
sheet. The new terrain PNG (512×368, 32 cols × 23 rows) has these approximate regions:

Row 0-2: Grass variants (light green, dark green) with dirt transitions
Row 3-5: Water tiles with shoreline autotile edges
Row 6-8: Sand/beach tiles
Row 9-11: Tilled soil variants (dry, wet, hoed)
Row 12-14: Dirt path variants
Row 15+: Additional terrain (cliff edges, etc.)

**IMPORTANT**: You MUST visually inspect the terrain PNG to find correct indices.
Read the file `assets/tilesets/modern_farm_terrain.png` to see the layout.
The rows above are APPROXIMATE. Find the actual center-fill tile for each TileKind:
- Grass: find a uniform grass center tile (no edges/transitions)
- Dirt: find a uniform dirt tile
- Water: find a uniform water center tile
- Sand: find a uniform sand tile
- TilledSoil: find tilled/hoed soil
- WateredSoil: find darker wet soil

Keep the seasonal grass variant system — use 4 different grass tiles for visual variety.
Keep the path autotile bitmask system (it works well already).

### Preferred approach
- Load the new terrain sheet alongside the old ones
- Update tile_atlas_info to prefer the new sheet for Grass, Dirt, Water, Sand, Stone
- Keep old sheets as fallback until confirmed working

### Tempting alternative: Replace the old PNGs entirely
### Why not: Breaking change if indices are wrong. Safer to add the new sheet and switch mappings.

---

## DELIVERABLE 2: Building Sprite Upgrade

### New asset: `assets/tilesets/modern_farm_buildings.png` (not yet copied — see below)
The props_and_buildings sheet is huge (512×2240). For buildings, the worker should
identify the farmhouse/barn sections and update ObjectAtlases to reference them.

**However**: The building system uses hardcoded tile indices in `objects.rs` (lines 1479-1565)
for house_walls and house_roof. Changing the atlas images requires carefully remapping
ALL those indices.

### Preferred approach for buildings
- Copy `3_Props_and_Buildings_16x16.png` to `assets/tilesets/modern_farm_buildings.png`
- Read the image to identify tile indices for: farmhouse walls, barn walls, roofs
- Update ObjectAtlases to load the new sheet
- Update house_walls and house_roof index references in spawn_building_sprites

### If building remapping is too complex
- Skip buildings, focus on terrain + animals + crops
- Buildings can be a follow-up task

---

## DELIVERABLE 3: Animal Sprites

### AnimalSpriteData (src/animals/mod.rs)
Currently holds only chicken and cow. Add all 7 animals:

| Animal | File | Pixel Size | Tile Size | Grid (cols×rows) |
|--------|------|-----------|-----------|-----------------|
| Chicken | sprites/chicken.png | 384×64 | 16×16 | 24×4 |
| Cow | sprites/cow.png | 1152×192 | 32×32 | 36×6 |
| Sheep | sprites/sheep.png | 768×128 | 32×32 | 24×4 |
| Goat | sprites/goat.png | 768×192 | 32×32 | 24×6 |
| Pig | sprites/pig.png | 768×128 | 32×32 | 24×4 |
| Duck | sprites/duck.png | 829×128 | 16×16 | 48×8 |
| Rabbit | sprites/rabbit.png | 768×128 | 16×16 | 48×8 |

Note: Duck is 829px wide which doesn't divide cleanly by 16. Use 48 cols (768px worth)
and ignore the rightmost partial column.

### load_animal_sprites
Add image/layout loading for sheep, goat, pig, duck, rabbit using dimensions above.
Update the chicken layout from `(4, 2)` to `(24, 4)`.
Update the cow layout from `(3, 2)` to `(36, 6)`.

### spawn_animals (src/animals/spawning.rs)
Replace ALL colored-rectangle blocks (Sheep, Goat, Pig, Duck, Rabbit) with
Sprite::from_atlas_image calls matching the Chicken/Cow pattern.

Custom sizes:
- Chicken: 16×16, Duck: 16×16, Rabbit: 16×16
- Cow: 32×32, Sheep: 32×32, Goat: 32×32, Pig: 32×32

Keep Horse and Cat as colored rectangles (no sprites in pack).

### AnimalAnimTimer.total_frames
When spawning, set total_frames to the NUMBER OF COLUMNS (first row only):
- Chicken: 24, Duck: 48, Rabbit: 48
- Cow: 36, Sheep: 24, Goat: 24, Pig: 24

The animation system cycles frame 0..total_frames. Using only the first row
gives a basic walk cycle without needing directional logic.

**Tempting alternative**: Use all rows for directional animation.
**Why not**: The current system doesn't track animal facing. First-row cycling is safe.

---

## DELIVERABLE 4: Crop Growth Sprites

### Per-crop sprite approach
The new crop sprites are individual per-crop sheets (all 112px wide = 7 cols of 16×16).
Heights vary (32-96px = 2-6 rows depending on crop height).

### Crops to upgrade (exist in both game AND farm pack):
| Crop ID | File | Dimensions | Grid |
|---------|------|-----------|------|
| turnip | crop_turnip.png | 112×48 | 7×3 |
| cauliflower | crop_cauliflower.png | 112×32 | 7×2 |
| strawberry | crop_strawberry.png | 112×32 | 7×2 |
| tomato | crop_tomato.png | 112×64 | 7×4 |
| corn | crop_corn.png | 112×64 | 7×4 |
| pumpkin | crop_pumpkin.png | 112×64 | 7×4 |
| wheat | crop_wheat.png | 112×32 | 7×2 |
| coffee | crop_coffee.png | 112×64 | 7×4 |
| watermelon | crop_watermelon.png | 112×64 | 7×4 |

### FarmingAtlases changes
Add a `crop_atlases: HashMap<String, (Handle<Image>, Handle<TextureAtlasLayout>)>`
field. Load each matching crop's sheet on startup keyed by crop_id.

### sync_crop_sprites changes
When rendering a crop, look up its atlas by crop_id in the HashMap.
If found, use the per-crop atlas. If not found (potato, melon, blueberry,
eggplant, cranberry, yam, ancient_fruit), fall back to the old plants.png atlas.

### sprite_stages in data/crops.rs
For crops with new atlases, update sprite_stages to sequential indices:
`vec![0, 1, 2, 3]` for 4-stage crops, `vec![0, 1, 2, 3, 4]` for 5-stage, etc.
The growth stage maps directly to the column index in the per-crop sheet.

### Keep plants.png as fallback
Rename nothing. Crops without Modern Farm sprites keep their existing indices.

---

## Failure patterns to avoid
- Do NOT redefine types from src/shared/mod.rs locally
- Do NOT edit files outside the allowed scope
- Do NOT change the AnimalKind or TileKind enums (they're in the contract)
- Do NOT remove the colored-rectangle fallback for Horse and Cat
- Do NOT try to implement directional animation for animals
- Do NOT run cargo fmt on shared/mod.rs
- Do NOT delete or rename the old atlas PNGs — keep as fallbacks
- Do NOT change map definitions or MapDef structures

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/sprite-upgrade.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Tempting alternatives rejected and why
- Known risks for integration
