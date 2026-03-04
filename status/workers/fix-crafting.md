# Worker Report: fix-crafting

## Files Modified
- `src/crafting/cooking.rs` (236 lines, down from 242)

## Changes Made

### Bug 1: Double Stamina Restoration (Fixed)
**Problem:** `handle_cook_item` directly restored stamina when cooking a food item (lines 136-147). The cooked food then went into inventory, and when eaten via `EatFoodEvent`, stamina was restored again -- resulting in a double-restore exploit.

**Fix:** Removed the entire stamina restoration block (the `if let Some(item_def) = item_registry.get(...)` block that set `player_state.stamina`). Food now only restores stamina when explicitly eaten via `EatFoodEvent`. The `StaminaDrainEvent { amount: 2.0 }` for cooking effort was preserved as intended.

Also removed the `mut player_state: ResMut<PlayerState>` parameter from `handle_cook_item` since it was only used by the removed stamina restoration block.

### Bug 2: Kitchen Accessible Without House Upgrade (Fixed)
**Problem:** `handle_cook_item` did not check `HouseState.has_kitchen`. Players could cook from day 1 without the kitchen house upgrade.

**Fix:** Added `house_state: Res<HouseState>` parameter to `handle_cook_item` and inserted a gate check immediately after the cooking mode check:
```rust
if !house_state.has_kitchen {
    ui_state.set_feedback("You need a kitchen upgrade first!".to_string());
    continue;
}
```

## Shared Type Imports Used
- `HouseState` (added -- `Res<HouseState>`)
- `PlayerState` (removed -- no longer needed)
- `StaminaDrainEvent` (retained, used for cooking effort cost)
- `Inventory`, `RecipeRegistry`, `ItemRegistry`, `UnlockedRecipes`, `ItemPickupEvent`, `PlaySfxEvent` (unchanged)

## Validation Results
- `cargo check`: PASS (zero errors, zero warnings in crafting scope)
- `cargo clippy -- -D warnings`: PASS (zero errors, zero warnings)

## Known Risks
- None. Both changes are isolated to `src/crafting/cooking.rs` and do not affect any other domain.
