# Blacksmith Cleanup — Implementation Notes

## Changes made to `src/economy/blacksmith.rs` only

### 1. Toast message fix (issue 1)
**Before:** `"Your {:?} upgrade is ready! Pick it up at the Blacksmith."`  
**After:** `"Your {:?} upgrade is complete — ready to use!"`

The old text implied a physical pickup step that doesn't exist; `tick_upgrade_queue` applies
the upgrade directly to `PlayerState::tools` at day-end, so the corrected message reflects
what actually happens.

### 2. Removed `UpgradeEntry` type alias (issue 2)
Lines 238–248 defined `pub type UpgradeEntry = (ToolKind, ToolTier, ToolTier, u32, &'static str, u8, bool, bool, bool)`.  
A grep across the entire `src/` tree confirmed it was never imported or referenced outside this
file. Removed in full.

### 3. Replaced local `required_bars_for_tier` with shared helpers (issue 3)
`src/shared/mod.rs` already provides two methods on `ToolTier`:
- `upgrade_bar_item(&self) -> Option<&'static str>` — bar needed to upgrade FROM this tier
- `upgrade_bars_needed(&self) -> u8` — quantity (always 5 for non-Iridium)

`handle_upgrade_request` was calling the local `required_bars_for_tier(target_tier)`.  
Replaced with `current_tier.upgrade_bar_item()` / `current_tier.upgrade_bars_needed()`,
which produce identical values.  The local function was then removed.

`cargo check` passes with zero errors.
