# Worker Report: Animated Sprite Sheets for Machines & Sprinklers

## Files Modified
- `src/crafting/machines.rs` — Added `MachineAnimTimer` component, `MACHINE_ANIM_FRAMES` constant, `animate_processing_machines` system; modified `handle_place_machine` to spawn with animated atlas
- `src/crafting/mod.rs` — Exported `MachineAnimTimer`; registered `animate_processing_machines` in PostUpdate
- `src/farming/render.rs` — Added `SprinklerAnimTimer` component, `SPRINKLER_ANIM_FRAMES`/`SPRINKLER_ANIM_DURATION` constants, `trigger_sprinkler_animation` and `animate_sprinklers` systems; modified `sync_farm_objects_sprites` to spawn sprinklers with animated atlas
- `src/farming/mod.rs` — Added `sprinkler_anim_image`/`sprinkler_anim_layout` to `FarmingAtlases`; loaded sprinkler_anim.png atlas; registered `trigger_sprinkler_animation` (Update) and `animate_sprinklers` (PostUpdate)
- `src/world/objects.rs` — Added `machine_anim_image`/`machine_anim_layout` to `FurnitureAtlases`; loaded machine_anim.png atlas in `ensure_furniture_atlases_loaded`

## Files Created
- `assets/sprites/machine_anim.png` — Copied from `_source_limezu/`
- `assets/sprites/sprinkler_anim.png` — Copied from `_source_limezu/`

## Sprite Sheet Analysis
- **machine_anim.png** (1984x96): 64x48 tiles, 31 cols x 2 rows. Row 0 = processing animation (31 frames). Row 1 = faint overlay effects.
- **sprinkler_anim.png** (1344x192): 32x32 tiles, 42 cols x 6 rows. Row 0 = full watering cycle (42 frames, idle -> spray -> idle). Rows 1-5 = particle/effect overlays.

## Animation Behavior
### Processing Machines
- When `is_processing()` is true: cycles through 31 frames at ~10 fps
- When idle or ready for collection: snaps to frame 0
- Spawned with `MachineAnimTimer` component alongside `ProcessingMachine`

### Sprinklers
- Triggered by `MorningSprinklerEvent` (fires each morning at day start)
- Plays watering animation for 4 seconds at ~10 fps, then returns to frame 0 (idle)
- `trigger_sprinkler_animation` activates all sprinkler timers on the event
- `animate_sprinklers` ticks frame advancement and duration tracking

## Fallbacks
- If machine_anim atlas handle is default (not loaded), falls back to static `processing_machine.png`, then color placeholder
- If sprinkler_anim atlas handle is default, falls back to static `sprinkler.png`, then color placeholder
- Static sprite loading paths are fully preserved

## Shared Type Imports Used
- `GameState`, `FarmObject`, `FarmState`, `Calendar`, `TILE_SIZE`, `Z_ENTITY_BASE`, `LogicalPosition`, `YSorted`, `GridPosition`, `Interactable`, `InteractionKind`, `ToastEvent`, `PlaySfxEvent`, `ToolUseEvent`, `ToolKind`, `ItemRegistry`, `Inventory`, `DayEndEvent`, `ItemPickupEvent`, `SoilState`, `SoilTile`, `grid_to_world_center`, `Z_FARM_OVERLAY`, `ItemId`, `SprinklerState`, `PlacedSprinkler`, `SprinklerKind`, `PlaceSprinklerEvent`, `CropTile`, `CropRegistry`, `CropDef`, `MapId`, `PlayerState`, `Weather`, `Season`

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — 128 passed, 0 failed, 2 ignored
- `cargo clippy -- -D warnings` — PASS (0 warnings)

## Known Risks
- Animation frame counts (31 machine, 42 sprinkler) are hardcoded based on sprite sheet analysis; if sheets change, constants need updating
- Sprinkler animation uses the full 32x32 tile rendered at TILE_SIZE (16px game units), which scales down the water spray effects — may need custom_size adjustment if visual quality is unsatisfactory
- Row 1+ of both sheets (overlay/particle effects) are not used; could be layered as separate entities for richer visuals in a future pass
