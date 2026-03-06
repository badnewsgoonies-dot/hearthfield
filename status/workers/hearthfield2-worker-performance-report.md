# Hearthfield 2.0 Worker — Performance Wave Report

## Status
Completed.

## Implemented Optimizations

### 1) Minimap redraw optimization (`src/ui/minimap.rs`)
- Added redraw gating based on:
  - map/base-layer changes,
  - farm crop-state changes while on `MapId::Farm`,
  - player tile movement,
  - player blink phase flips,
  - NPC tile-set movement changes.
- Added cached render state fields to `MinimapState` (`last_player_tile`, `last_player_blink_on`, `last_npc_tiles`).
- Minimap now skips texture writes when no visible minimap data changed.

### 2) HUD proximity scan throttling (`src/ui/hud.rs`)
- Added `InteractionPromptCache` resource keyed by:
  - current map,
  - player tile,
  - gift-capability context,
  - relevant key bindings.
- `update_interaction_prompt` now avoids full proximity rescans when cache is reusable.
- Cache invalidation includes changed transforms for interactables/NPCs/chests, NPC registry changes, and binding/gift-context changes.

### 3) Y-sort pass consolidation (`src/world/ysort.rs`)
- Replaced 3-query multi-pass update with one filtered pass:
  - `Or<(With<LogicalPosition>, With<YSorted>)>` query,
  - preserves rounded XY behavior for `LogicalPosition` entities,
  - preserves Y-based Z computation rules for `YSorted` entities.

### 4) Weather particle accounting cleanup (`src/world/weather_fx.rs`, `src/world/mod.rs`)
- Removed per-frame `iter().count()` pattern.
- Added `WeatherParticleCounts` resource and initialized in `WorldPlugin`.
- Particle totals now maintained on spawn/despawn/cleanup paths while preserving:
  - indoor-map suppression,
  - per-weather spawn cadence,
  - hard cap behavior.

### 5) Farming render incremental detection (`src/farming/mod.rs`, `src/farming/render.rs`)
- Added incremental early exits in render sync systems:
  - `sync_soil_sprites`: skip when `FarmState` and `FarmingAtlases` unchanged.
  - `sync_crop_sprites`: skip when `FarmState`, `CropRegistry`, and `FarmingAtlases` unchanged.
  - `sync_farm_objects_sprites`: skip when `FarmState` and object/furniture atlases unchanged.
- Scoped farming render sync systems to farm map only via `run_if(in_farm_map)`.

## Targeted Tests Added
- Added unit test in `src/world/ysort.rs`:
  - `consolidated_pass_preserves_rounding_and_z_rules`
  - Verifies behavior parity for:
    - y-sorted with logical position,
    - non-y-sorted with logical position,
    - y-sorted without logical position.

## Validation
- `cargo check` — passed
- `cargo test --test headless` — passed (88 passed, 0 failed, 2 ignored)
- `cargo test --lib consolidated_pass_preserves_rounding_and_z_rules` — passed
