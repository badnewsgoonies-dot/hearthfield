# impl_save_stats — Done

## Changes made

### `src/save/mod.rs`
All changes are additive (no existing code removed).

1. **`FullSaveFile` struct** — added two fields with `#[serde(default)]`:
   ```rust
   pub harvest_stats: crate::economy::stats::HarvestStats,
   pub animal_product_stats: crate::economy::stats::AnimalProductStats,
   ```

2. **`ExtendedResources<'w>`** — added read-only handles:
   ```rust
   pub harvest_stats: Res<'w, crate::economy::stats::HarvestStats>,
   pub animal_product_stats: Res<'w, crate::economy::stats::AnimalProductStats>,
   ```

3. **`ExtendedResourcesMut<'w>`** — added mutable handles:
   ```rust
   pub harvest_stats: ResMut<'w, crate::economy::stats::HarvestStats>,
   pub animal_product_stats: ResMut<'w, crate::economy::stats::AnimalProductStats>,
   ```

4. **`write_save` (native)** — added two parameters and matching `FullSaveFile` fields.

5. **`write_save` (WASM stub)** — added two `_`-prefixed parameters to keep signatures in sync.

6. **`handle_save_request`** — passes `&ext.harvest_stats` and `&ext.animal_product_stats` to `write_save`.

7. **`handle_load_request`** — restores both resources from the loaded file.

8. **`handle_new_game`** — resets both resources to `Default`.

## Verification
`cargo check` exits 0 with no errors.
