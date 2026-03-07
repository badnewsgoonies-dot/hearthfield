# UI Functional Fixes

## Changes Made

### 1. Crafting Key (`src/ui/menu_input.rs`)
- Added `input.open_crafting` handler in `gameplay_state_transitions` to transition to `GameState::Crafting` from Playing.
- `menu_cancel_transitions` already handled `GameState::Crafting` (both toggle-close via `input.open_crafting` and cancel via Escape) — left untouched.

### 2. Chest Scroll (`src/ui/chest_screen.rs`)
- Increased `VISIBLE_SLOTS` from `12` to `36` (matching `TOTAL_INVENTORY_SLOTS = HOTBAR_SLOTS(12) + BACKPACK_SLOTS(24)`).
- The cursor clamp `ui_state.cursor < VISIBLE_SLOTS - 1` in `handle_chest_input` automatically picks up the new value (now clamps at 35 instead of 11).
- Both inventory and chest panels now spawn 36 slot rows, showing all inventory slots without scrolling.

## Verification
- `cargo check` passes for both changed files; 4 pre-existing errors in other files are unrelated.
