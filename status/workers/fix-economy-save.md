# Worker Report: FIX-ECONOMY-SAVE

## Files Modified
- `src/economy/gold.rs` — added `serde::Serialize, serde::Deserialize` to `EconomyStats` derive
- `src/save/mod.rs` — 7 changes (see below)

## What Was Implemented
1. `EconomyStats` now derives `serde::Serialize, serde::Deserialize`
2. `FullSaveFile` has new `#[serde(default)] pub economy_stats: crate::economy::gold::EconomyStats`
3. `ExtendedResources` has `pub economy_stats: Res<'w, crate::economy::gold::EconomyStats>`
4. `ExtendedResourcesMut` has `pub economy_stats: ResMut<'w, crate::economy::gold::EconomyStats>`
5. `write_save` (native + wasm stub) signature extended with `economy_stats` param
6. `FullSaveFile` construction in `write_save` sets `economy_stats: economy_stats.clone()`
7. `handle_load_request` restores `*ext.economy_stats = file.economy_stats`
8. `handle_new_game` resets `*ext.economy_stats = crate::economy::gold::EconomyStats::default()`

## Quantitative Targets
- 0 errors, 0 warnings across all gates ✓

## Validation Results
- `cargo check` — PASS
- `cargo test --test headless` — PASS (88 passed, 0 failed)
- `cargo clippy -- -D warnings` — PASS

## Known Risks
- None. `#[serde(default)]` ensures backwards compatibility with old save files lacking the field.
