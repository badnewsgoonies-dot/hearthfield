# Worker Report: Fix Hotbar Icon Sizing + Derive Prompt Strings from KeyBindings

## Files Modified
- `src/ui/hud.rs`

## Changes Made

### Task 1: Icon Sizing (Lines ~405-406)
- **Old:** `Val::Px(28.0)` for both width and height
- **New:** `Val::Px(32.0)` for both width and height
- 2x scale of 16×16 sprites → crisp nearest-neighbour rendering

### Task 2: Prompt Strings Derived from KeyBindings (Lines ~900-950)

Added `bindings: Res<KeyBindings>` parameter to `update_interaction_prompt`.

Added `key_display(code: KeyCode) -> String` helper function (before the system).

Updated 4 prompt strings:
| Old | New |
|-----|-----|
| `format!("[F] {}", inter.label)` | `format!("[{}] {}", key_display(bindings.interact), inter.label)` |
| `"[F] Storage".to_string()` | `format!("[{}] Storage", key_display(bindings.interact))` |
| `format!("[F] Talk to {} \| [R] Give Gift", name)` | `format!("[{}] Talk to {} \| [{}] Give Gift", key_display(bindings.interact), name, key_display(bindings.tool_secondary))` |
| `format!("[F] Talk to {}", name)` | `format!("[{}] Talk to {}", key_display(bindings.interact), name)` |

## Validation
`cargo check` — **passed** with zero errors (2 pre-existing unrelated warnings in `src/player/interact_dispatch.rs`).
