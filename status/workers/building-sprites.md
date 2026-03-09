# Worker Report: Building Sprite Upgrade

## Files Modified
- `src/world/objects.rs` — added composite building image support

## Files Created
- `assets/sprites/farmhouse.png` (copied from `_source_limezu/`)
- `assets/sprites/barn.png` (copied from `_source_limezu/`)
- `assets/sprites/chicken_house.png` (copied from `_source_limezu/`)
- `assets/sprites/well.png` (copied from `_source_limezu/`)

## What Was Implemented

### ObjectAtlases — new image handles
Added four `Handle<Image>` fields to `ObjectAtlases`:
- `farmhouse_image` (128x160)
- `barn_image` (128x160)
- `chicken_house_image` (48x48)
- `well_image` (48x32)

All loaded in `ensure_object_atlases_loaded`.

### BuildingImage enum
New enum (`Farmhouse`, `Barn`, `ChickenHouse`, `Well`) used as an optional
`composite` field on `BuildingDef`. Helper functions `resolve_building_image`
and `building_image_source_size` map variants to handles and source dimensions.

### spawn_building_sprites — composite path
Buildings with `composite: Some(...)` render as a single `Sprite::from_image`
scaled to match the tile footprint width, with height derived from aspect ratio.
The sprite is bottom-anchored at the building's front edge and extends upward
to cover walls and roof.

Farm buildings now use composites:
- Player house -> `Farmhouse`
- Chicken coop -> `ChickenHouse`
- Barn -> `Barn`

Town buildings retain the tile-by-tile wall + roof + door rendering (no composite).

### Well handle loaded but not spawned
The well image handle is loaded and ready. No spawn point exists in current
building definitions; the `Well` variant is `#[allow(dead_code)]` until a
spawn location is added.

## Validation Results
- `cargo check` — pass (0 errors, 0 warnings)
- `cargo test --test headless` — pass (128 passed, 2 ignored, 0 failed)
- `cargo clippy -- -D warnings` — pass (0 warnings)

## Shared Type Imports Used
From `src/shared/mod.rs`: `PlayerState`, `MapId`, `WorldObject` (via re-export),
`grid_to_world_center`, `TILE_SIZE`, `Z_GROUND`.

## Known Risks
- Composite sprite positioning uses bottom-anchor math; visual alignment may
  need tuning once rendered in-game with the camera system.
- The farmhouse/barn images (128x160) are significantly taller than the original
  tile-by-tile buildings; nearby objects or map edges may need adjustment.
