# Visual Polish Worker Report

## Files Created
- `src/world/grass_decor.rs` (180 lines) — new grass decoration system

## Files Modified
- `src/world/mod.rs` — added module declaration, import, resource init, and system registration for grass decorations; fixed pre-existing clippy issues (orphaned doc comment, too_many_arguments)
- `src/world/lighting.rs` — improved day/night lighting keyframes
- `src/world/seasonal.rs` — enhanced falling leaf variety (colors, sizes, rotation)
- `src/world/weather_fx.rs` — improved snow particle visuals

## Improvements Implemented

### 1. Grass Decoration Sprites (grass_decor.rs)
- Spawns small 12x12 decorative sprites from `grass_biome.png` on grass tiles
- Uses positional hashing for deterministic, stable placement
- ~15-20% of grass tiles get decorations (varies by season)
- Decorations are slightly offset from tile center using a secondary hash for natural look
- Z layer at 5.0 (between Z_GROUND=0 and Z_FARM_OVERLAY=10)
- Tagged with MapTile component so they despawn with map transitions

### 2. Seasonal Grass Decorations (integrated into grass_decor.rs)
- Spring: ~18% coverage, favors flowers and small plants (indices 0,2,3,4,5,6)
- Summer: ~15% coverage, dry tufts and weeds (indices 0,1,5,7,8)
- Fall: ~10% coverage, sparse grass tufts only (indices 0,1,5)
- Winter: ~8% coverage, very sparse (indices 0,5)
- Season-appropriate tint applied to decorations
- Decorations respawn automatically on season change

### 3. Improved Day/Night Lighting Keyframes
- Sunrise (6:00): warmer golden tint — (1.0, 0.85, 0.55) instead of (1.0, 0.9, 0.7)
- New golden hour keyframe at 17:00: warm amber (1.0, 0.92, 0.70) at intensity 0.08
- Sunset (18:00): more dramatic — (1.0, 0.75, 0.45) at intensity 0.18
- Night intensity reduced to 0.45 (from 0.5) — less oppressive darkness
- Midnight/late night also at 0.45

### 4. Improved Falling Leaf Variety
- 4 leaf colors instead of 2: vivid orange, deep red, golden yellow, brown
- Variable leaf sizes: 2-6px instead of fixed 4px
- Initial random rotation on spawn
- Continuous gentle rotation animation tied to oscillation frequency
- Spawn rate increased: 1 per 45 frames (from 60) for denser autumn feel

### 5. Snow Particle Improvements
- Larger snowflakes: 5x5 instead of 4x4
- Varied brightness (0.92-1.0) with slight blue tint for depth
- Varied alpha (0.6-0.85) for visual variety
- Slower drift: speed 20-45 (from 30-60), drift freq 0.8-2.5 (from 1.0-3.0)
- Overall more peaceful, gentle snowfall feel

## Shared Type Imports Used
- `Calendar`, `Season`, `Weather`, `MapId`, `TileKind`
- `PlayerState`, `GameState`, `UpdatePhase`
- `TILE_SIZE`, `SCREEN_WIDTH`, `SCREEN_HEIGHT`
- `Z_GROUND`, `Z_SEASONAL`, `Z_WEATHER`
- `grid_to_world_center`, `world_to_grid`

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (128 passed, 0 failed, 2 ignored)
- `cargo clippy -- -D warnings` — PASS (0 warnings)

## Pre-existing Issues Fixed
- Fixed orphaned `///` doc comment in mod.rs (converted to `//` comment)
- Added `#[allow(clippy::too_many_arguments)]` to `neighbor_is` in mod.rs

## Known Risks
- Grass decorations add entities proportional to grass tile count (~10-18% of grass tiles). For a 40x30 map with 50% grass, that's ~60-108 extra entities — very lightweight.
- No performance concern: decorations are static sprites with no per-frame update cost.
