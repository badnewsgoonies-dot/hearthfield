# Pilot Sprite Wiring Report

## Files modified
- `dlc/pilot/src/crew/spawning.rs`
- `dlc/pilot/src/crew/mod.rs`

## What changed
- Rewired `crew_sprite_file()` to use DLC asset path `pilot/sprites/crew_sheet.png` (assets are resolved from `../../assets` via `AssetPlugin` in `dlc/pilot/src/main.rs`).
- Updated crew sprite atlas layout to `TextureAtlasLayout::from_grid(UVec2::new(16, 16), 10, 1, None, None)`.
- Added `crew_sprite_index()` and now map `sprite_index` to atlas frame `0..9` using modulo 10.
- Added `CrewPortraitData` resource in `spawning.rs` and initialized it in `CrewPlugin` (`mod.rs`).
- Added portrait helper `crew_portrait_file()` returning `pilot/sprites/crew_portraits.png` and wired portrait atlas/image loading (`16x16`, `10x1`) during spawn setup.

## Validation
Attempted:
- `cd dlc/pilot && cargo check`

Result in this environment: **failed due toolchain mismatch**, not code errors:
- Cargo 1.75 cannot parse lockfile v4 and cannot build dependencies requiring `edition2024`.
- Also confirmed with `cargo -Znext-lockfile-bump check -q`, which fails on dependency/toolchain compatibility.
