# UI Visual Consistency Fixes

## Status: Complete

## Changes Made

### 1. Font Consistency (3 files)

**src/ui/shop_screen.rs**
- Added `use super::UiFontHandle;`
- Added `font_handle: Res<UiFontHandle>` param to `spawn_shop_screen`
- Added `font: font_handle.0.clone()` to all 6 TextFont instances: ShopTitle, ShopGoldDisplay, ShopModeText, ShopItemName, ShopItemPrice, ShopHintText

**src/ui/building_upgrade_menu.rs**
- Added `use super::UiFontHandle;`
- Added `font_handle: Res<UiFontHandle>` param to `spawn_building_upgrade_menu`
- Added `font: font_handle.0.clone()` to all 5 TextFont instances: title, BuildingUpgradeStatusText, BuildingRowText, BuildingRowCost, hint

**src/ui/crafting_screen.rs**
- Added `use super::UiFontHandle;`
- Added `font_handle: Res<UiFontHandle>` param to `spawn_crafting_screen`
- Added `font: font_handle.0.clone()` to all 5 TextFont instances: title, CraftingStatusText, CraftingRecipeName, CraftingRecipeMaterials, hint

### 2. Z-Index Fix

**src/ui/chest_screen.rs**
- Changed `GlobalZIndex(100)` → `GlobalZIndex(50)` on ChestScreenRoot to avoid collision with ScreenFadeOverlay

### 3. Portrait Fix

**src/ui/dialogue_box.rs**
- Changed hardcoded `index: 0` → `index: ui_state.as_ref().and_then(|s| s.portrait_index).unwrap_or(0) as usize` in the portrait TextureAtlas

## Verification

`cargo check` shows 4 pre-existing errors in unrelated files (evaluation.rs, quests.rs, shipping.rs). No new errors introduced by these changes.
