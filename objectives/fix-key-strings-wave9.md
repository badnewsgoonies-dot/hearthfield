# Worker: Fix Remaining Hardcoded Key Strings + Add Keybinding Duplicate Test

## Context
We already fixed hardcoded key strings in `src/ui/hud.rs` (Wave 7), but several other files still have wrong or hardcoded key references:

1. `src/calendar/festivals.rs` line 103: `"Press E to participate"` — WRONG. Interact key is F (KeyCode::KeyF). Players will press E and nothing happens.
2. `src/ui/relationships_screen.rs` line 96: `"R/Esc: Close"` — WRONG. Relationships key was moved to L (KeyCode::KeyL).

Additionally, the R key collision happened because there was no test preventing duplicate bindings. We need a regression guard.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/calendar/`, `src/ui/`, and `tests/`
All out-of-scope edits will be reverted.

## Required reading
1. src/shared/mod.rs — READ ONLY. Find `KeyBindings` struct and its Default impl to see current key assignments.
2. src/calendar/festivals.rs — the toast at line ~103
3. src/ui/relationships_screen.rs — the close hint at line ~96

## Task 1: Fix festival toast (src/calendar/festivals.rs)

Find line ~103:
```rust
message: format!("Today is the {}! Press E to participate.", name),
```

Change to:
```rust
message: format!("Today is the {}! Press F to participate.", name),
```

Note: Ideally this would use KeyBindings dynamically, but the system may not have access to Res<KeyBindings>. If it does, use `key_display(bindings.interact)` like hud.rs does. If not, just hardcode F (which matches the actual default).

## Task 2: Fix relationships close hint (src/ui/relationships_screen.rs)

Find line ~96:
```rust
Text::new("W/S or Arrows: Navigate | R/Esc: Close"),
```

Change to:
```rust
Text::new("W/S or Arrows: Navigate | L/Esc: Close"),
```

## Task 3: Add keybinding duplicate test

Create a new test file `tests/keybinding_duplicates.rs` (or add to an existing test file in `tests/`):

```rust
//! Regression test: no unintended duplicate keybindings.
use hearthfield::shared::KeyBindings;
use bevy::prelude::KeyCode;
use std::collections::HashMap;

#[test]
fn default_keybindings_have_no_unintended_duplicates() {
    let b = KeyBindings::default();
    let mut map: HashMap<KeyCode, Vec<&'static str>> = HashMap::new();

    macro_rules! add {
        ($key:expr, $name:expr) => {
            map.entry($key).or_default().push($name);
        };
    }

    add!(b.move_up, "move_up");
    add!(b.move_down, "move_down");
    add!(b.move_left, "move_left");
    add!(b.move_right, "move_right");
    add!(b.interact, "interact");
    add!(b.tool_use, "tool_use");
    add!(b.tool_secondary, "tool_secondary");
    add!(b.open_inventory, "open_inventory");
    add!(b.open_crafting, "open_crafting");
    add!(b.open_map, "open_map");
    add!(b.open_journal, "open_journal");
    add!(b.open_relationships, "open_relationships");
    add!(b.pause, "pause");
    add!(b.tool_next, "tool_next");
    add!(b.tool_prev, "tool_prev");
    add!(b.ui_confirm, "ui_confirm");
    add!(b.ui_cancel, "ui_cancel");
    add!(b.skip_cutscene, "skip_cutscene");

    // These are intentionally shared across input contexts:
    let allow = [KeyCode::Escape, KeyCode::Space];

    for (k, fields) in &map {
        if fields.len() > 1 && !allow.contains(k) {
            panic!(
                "Duplicate keybinding: {:?} is used by {:?}",
                k, fields
            );
        }
    }
}
```

NOTE: This test needs to import from the hearthfield crate. Check what's publicly exported. If `KeyBindings` isn't accessible as `hearthfield::shared::KeyBindings`, check how other tests import it (look at existing test files in `tests/`). Adjust the import path accordingly.

If `tests/` directory doesn't exist or the crate structure doesn't support integration tests easily, add it as a unit test inside `src/shared/mod.rs` instead:
```rust
#[cfg(test)]
mod keybinding_tests {
    use super::*;
    // ... same test body ...
}
```

## Do NOT
- Modify src/shared/mod.rs
- Modify src/player/ or src/npcs/
- Change any game logic or behavior
- Add new features

## Validation
```
cargo check
cargo test
```
Must pass with zero errors.

## When done
Write completion report to status/workers/fix-key-strings-wave9.md
