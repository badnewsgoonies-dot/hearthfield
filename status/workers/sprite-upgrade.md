# Worker Report: Sprite Upgrade

## Files Modified (with approximate line counts changed)

| File | Changes |
|------|---------|
| `src/animals/spawning.rs` | Replaced 5 colored-rectangle animal blocks (Sheep, Goat, Pig, Duck, Rabbit) with atlas sprite spawning; updated animation frame counts for all 7 atlas animals |
| `src/farming/mod.rs` | Added `crop_atlases: HashMap<String, (Handle<Image>, Handle<TextureAtlasLayout>)>` field to `FarmingAtlases`; added per-crop sheet loading in `load_farming_atlases` for 9 crops |
| `src/farming/render.rs` | Updated `sync_crop_sprites` to prefer per-crop atlas over plants.png for both existing-entity updates and newly-spawned entities |
| `src/data/crops.rs` | Updated `sprite_stages` to sequential indices (0,1,2,...) for 8 crops with per-crop sheets: turnip, cauliflower, strawberry, tomato, corn, pumpkin, wheat, coffee |

## What Was Implemented

### Deliverable 1: Terrain Atlas (already complete)
The terrain atlas was already wired in a prior session. `TerrainAtlases` has `terrain_image`/`terrain_layout` fields, `ensure_atlases_loaded` loads `modern_farm_terrain.png` (32x23 grid), and `tile_atlas_info` maps all TileKinds (Grass, Dirt, Water, Sand, TilledSoil, WateredSoil) to the modern farm terrain sheet. Verified correct and untouched.

### Deliverable 2: Buildings (SKIPPED per instructions)
Buildings skipped as directed -- terrain + animals + crops are the priority.

### Deliverable 3: Animal Sprites
- **Sheep, Goat, Pig**: Now use real 32x32 atlas sprites from their respective sprite sheets instead of colored rectangles
- **Duck, Rabbit**: Now use real 16x16 atlas sprites instead of colored rectangles
- **Chicken, Cow**: Already had atlas sprites; updated frame counts from (4,3) to (24,36) to match actual sheet column counts
- **Horse, Cat, Dog**: Kept as colored rectangles (no sprite sheets available)
- Animation frame counts updated to match actual sprite sheet columns: Chicken=24, Cow=36, Sheep=24, Goat=24, Pig=24, Duck=48, Rabbit=48
- Animation period set to 0.15s for 24/36-col animals, 0.12s for 48-col animals (Duck/Rabbit)

### Deliverable 4: Crop Growth Sprites
- Added `crop_atlases` HashMap to `FarmingAtlases` for per-crop sprite sheets
- Loading 9 per-crop sheets on startup: turnip (7x3), cauliflower (7x2), strawberry (7x2), tomato (7x4), corn (7x4), pumpkin (7x4), wheat (7x2), coffee (7x4), watermelon (7x4)
- `sync_crop_sprites` now checks `crop_atlases` first; uses sequential column index (current_stage) for per-crop sheets, falls back to plants.png + crop_atlas_index for crops without sheets
- Updated sprite_stages in data/crops.rs to sequential [0,1,2,...] for 8 crops with per-crop sheets
- Crops without per-crop sheets (potato, melon, blueberry, eggplant, cranberry, yam, ancient_fruit) keep their existing plants.png indices unchanged

## Shared Type Imports Used
- `AnimalKind` (from shared, used in spawning match arms)
- `CropTile`, `CropDef`, `CropRegistry`, `FarmState`, `SoilState`, `SoilTile` (from shared, used in render.rs)
- `Season` (from shared, used in crops.rs)
- No types were redefined locally

## Validation Results
- `cargo check` -- PASS (clean compile)
- `cargo test --test headless` -- PASS (128 passed, 0 failed, 2 ignored)
- `cargo clippy -- -D warnings` -- PASS (zero warnings)

## Tempting Alternatives Rejected
- **Directional animal animation**: Could use multiple rows for facing directions, but the current system doesn't track animal facing. First-row cycling is safe and correct.
- **Delete old atlas PNGs**: Old grass.png, tilled_dirt.png, water.png kept as fallbacks per spec. plants.png kept for crops without per-crop sheets.
- **Add watermelon CropDef**: crop_watermelon.png is loaded but no CropDef exists for watermelon in the game. Adding it would be scope creep.

## Known Risks for Integration
- Animal animation speed may need tuning -- 48-frame cycles at 0.12s = 5.76s per full cycle, which might look slow for idle wander animation. Consider using a subset of frames (e.g., first 4-6 frames) for walk cycles.
- Per-crop atlas indices assume growth stages map 1:1 to column indices 0..N. If a crop has more growth_days entries than columns in its sheet, the index will exceed the atlas bounds. Current crops are safe (checked: all have growth_days.len() <= 7 columns).
- Terrain tile indices (5, 6, 67, 101, 291, 419, 417, 19, 161) were set in a prior session by visual inspection. May need fine-tuning if visual appearance doesn't match expectations.
