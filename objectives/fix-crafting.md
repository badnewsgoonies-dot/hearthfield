# Worker: FIX-CRAFTING

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/crafting/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. src/crafting/cooking.rs (read the FULL file — both bugs live here)
2. src/crafting/mod.rs (read the FULL file — understand plugin structure)
3. src/shared/mod.rs (the type contract — import from here, do not redefine)
4. Look at HouseState struct in src/shared/mod.rs (search for "HouseState")

## Bug 1: Cooking Restores Stamina Twice (Double-Restore)

### Root Cause
`handle_cook_item` in `src/crafting/cooking.rs` at lines ~136-146 directly restores stamina when cooking:
```rust
if item_def.edible && item_def.energy_restore > 0.0 {
    let restore = item_def.energy_restore;
    let new_stamina = (player_state.stamina + restore).min(player_state.max_stamina);
    player_state.stamina = new_stamina;
}
```
The cooked food then goes into the player's inventory. When the player later eats it (via EatFoodEvent), stamina is restored AGAIN. This means every cooked food item restores stamina twice: once at cook-time and once at eat-time.

### Fix Required
Remove the direct stamina restoration block (lines ~136-146) entirely. The food should ONLY restore stamina when eaten via EatFoodEvent. Keep the `StaminaDrainEvent { amount: 2.0 }` at line ~162 — that's the cooking effort cost, which is correct.

Specifically, delete or comment out this block:
```rust
// Apply stamina restoration if the result item is edible
if let Some(item_def) = item_registry.get(&recipe.result) {
    if item_def.edible && item_def.energy_restore > 0.0 {
        let restore = item_def.energy_restore;
        let new_stamina = (player_state.stamina + restore).min(player_state.max_stamina);
        info!(
            "Cooking '{}' restored {:.0} stamina (from {:.0} to {:.0})",
            recipe.name, restore, player_state.stamina, new_stamina
        );
        player_state.stamina = new_stamina;
    }
}
```

After removal, if `player_state` is no longer used (was only used for the stamina restore), remove the `mut player_state: ResMut<PlayerState>` parameter to avoid clippy warnings. But check first — it may be used elsewhere in the function.

## Bug 2: Kitchen Accessible Without House Upgrade

### Root Cause
`handle_cook_item` does not check `HouseState.has_kitchen`. The cooking UI should only be available when the player has upgraded their house to include a kitchen. Currently, cooking works from day 1.

### Fix Required
At the top of the `for event in events.read()` loop in `handle_cook_item`, BEFORE the cooking mode check, add a kitchen gate:
```rust
// Add HouseState to the function parameters:
house_state: Res<HouseState>,

// Then at the top of the loop body:
if !house_state.has_kitchen {
    ui_state.set_feedback("You need a kitchen upgrade first!".to_string());
    continue;
}
```

Actually — place the kitchen check AFTER the `if !ui_state.is_cooking_mode { continue; }` check (line ~40-42), so it only fires when the player is actually trying to cook.

### Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

## When done
Write completion report to status/workers/fix-crafting.md containing:
- Files modified (with line counts)
- What was changed and why
- Validation results (pass/fail)
