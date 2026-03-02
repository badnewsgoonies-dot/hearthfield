# save_shipping_log worker report

## Objective
Added `ShippingLog` to the save/load system in `src/save/mod.rs` following the same placement pattern as `BuildingLevels`.

## Changes made
1. Added `ShippingLog` import at top of file.
2. Added `shipping_log` to `ExtendedResources` after `building_levels`.
3. Added `shipping_log` to `ExtendedResourcesMut` after `building_levels`.
4. Added `shipping_log` to `FullSaveFile` after `building_levels` with `#[serde(default)]`.
5. Updated `write_save` (native + wasm signatures) to accept `ShippingLog` and serialize it into `FullSaveFile`.
6. Updated `handle_save_request` call site to pass `&ext.shipping_log` into `write_save`.
7. Updated load restore path to assign `*ext.shipping_log = file.shipping_log;` after building levels restore.
8. Updated new-game reset path to assign `*ext.shipping_log = ShippingLog::default();` after building levels reset.

## Validation
- `grep -n "ShippingLog\|shipping_log" src/save/mod.rs`
  - Found 10 matches covering import, both system param bundles, full save struct field, write params/construction/call, load restore, and new game reset.
- `cargo check`
  - Passed: `Finished dev profile [optimized + debuginfo] target(s)`.
