# Shipping Implementation Report

**File modified:** `src/economy/shipping.rs` only  
**Date:** 2026-03-02

---

## Issue 1: `ShippingLog.shipped_items` Never Populated — FIXED

**Root cause:** `process_shipping_bin_on_day_end` did not take `ShippingLog` as a parameter and never wrote to it.

**Fix:** Added `mut shipping_log: ResMut<ShippingLog>` to the system parameters. Inside the per-slot loop, after computing `slot_value`:

```rust
*shipping_log.shipped_items.entry(slot.item_id.clone()).or_insert(0) +=
    slot.quantity as u32;
```

`ShippingLog.shipped_items` is a `HashMap<ItemId, u32>` (unique items → total quantity shipped). This accumulates across days (the map is never cleared here, matching its intent as a lifetime collection tracker). The year-end evaluation's `shipped_items.len() >= 30` check will now work correctly.

---

## Issue 2: Shipping Bin Ignores Item Quality — NOT FIXED (future improvement)

**Finding:** `ShippingBin.items` is `Vec<InventorySlot>`. `InventorySlot` has no quality field. `QualityStack` does have quality, but the bin does not use it.

**Action:** A `TODO` comment was already present in `shipping.rs` (lines 116–120) explaining this limitation and the migration path. It has been preserved unchanged:

```rust
// TODO: Apply ItemQuality::sell_multiplier() here once ShippingBin tracks quality.
// InventorySlot (used by ShippingBin) has no quality field; QualityStack does.
// To fix: replace ShippingBin.items: Vec<InventorySlot> with Vec<QualityStack>,
// update place_in_shipping_bin to preserve quality from the harvested item,
// then multiply: (sell_price as f32 * quality.sell_multiplier()) as u32.
```

Applying quality multipliers (Normal 1.0×, Silver 1.25×, Gold 1.5×, Iridium 2.0×) requires changing `ShippingBin`'s data structure in `src/shared/mod.rs`, which is outside the allowed file scope.

---

## Validation

- `ShippingLog` is `pub` in `src/shared/mod.rs` (line 1389), imported via `use crate::shared::*` — no additional imports needed.
- `cargo check` could not run in this environment (missing ALSA system library), but the change is a straightforward system parameter addition + HashMap entry update with no unsafe code or API misuse.
