# Worker: Fix Hotbar Icon Sizing + Derive Prompt Strings from KeyBindings

## Context
Two related UI issues in src/ui/hud.rs:

1. **Hotbar icon sizing:** Lines 405-406 use `Val::Px(28.0)` for icons sourced from 16×16 sprite frames. 28/16 = 1.75x which is fractional — nearest-neighbour sampling produces uneven pixel widths. Should be 32.0 (2x) for crisp rendering.

2. **Hardcoded key strings:** Lines 905, 913, 933, 935 hardcode `[F]` and `[R]` instead of reading from the `KeyBindings` resource. After the R key collision fix, `open_relationships` moved to `KeyL`, but `tool_secondary` is still `KeyR`. The prompt at line 933 says `[R] Give Gift` — this should derive from `bindings.tool_secondary`, and the interact key `[F]` should derive from `bindings.interact`.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/ui/`
All out-of-scope edits will be reverted.

## Required reading
1. src/ui/hud.rs — the full file, especially:
   - Lines 405-406 (hotbar icon sizing)
   - Lines 879-940 (interaction prompt system `update_interaction_prompt`)
2. src/shared/mod.rs — READ ONLY. Find the `KeyBindings` struct to see field names and the `key_name` helper if one exists. Import KeyBindings from shared.

## Task 1: Fix icon sizing
Find:
```rust
width: Val::Px(28.0),
height: Val::Px(28.0),
```
Replace with:
```rust
width: Val::Px(32.0),
height: Val::Px(32.0),
```

## Task 2: Derive prompt strings from KeyBindings
In the `update_interaction_prompt` system:

1. Add `bindings: Res<KeyBindings>` to the system parameters (import KeyBindings from shared if needed).

2. Create a helper to format a KeyCode as a short display string:
```rust
fn key_display(code: KeyCode) -> String {
    match code {
        KeyCode::KeyA => "A".into(),
        KeyCode::KeyB => "B".into(),
        KeyCode::KeyC => "C".into(),
        KeyCode::KeyD => "D".into(),
        KeyCode::KeyE => "E".into(),
        KeyCode::KeyF => "F".into(),
        KeyCode::KeyG => "G".into(),
        KeyCode::KeyH => "H".into(),
        KeyCode::KeyI => "I".into(),
        KeyCode::KeyJ => "J".into(),
        KeyCode::KeyK => "K".into(),
        KeyCode::KeyL => "L".into(),
        KeyCode::KeyM => "M".into(),
        KeyCode::KeyN => "N".into(),
        KeyCode::KeyO => "O".into(),
        KeyCode::KeyP => "P".into(),
        KeyCode::KeyQ => "Q".into(),
        KeyCode::KeyR => "R".into(),
        KeyCode::KeyS => "S".into(),
        KeyCode::KeyT => "T".into(),
        KeyCode::KeyU => "U".into(),
        KeyCode::KeyV => "V".into(),
        KeyCode::KeyW => "W".into(),
        KeyCode::KeyX => "X".into(),
        KeyCode::KeyY => "Y".into(),
        KeyCode::KeyZ => "Z".into(),
        KeyCode::Space => "Space".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Escape => "Esc".into(),
        other => format!("{:?}", other),
    }
}
```

3. Replace hardcoded strings:
   - `"[F] {}"` → `format!("[{}] {}", key_display(bindings.interact), ...)`
   - `"[F] Storage"` → `format!("[{}] Storage", key_display(bindings.interact))`
   - `"[F] Talk to {} | [R] Give Gift"` → `format!("[{}] Talk to {} | [{}] Give Gift", key_display(bindings.interact), name, key_display(bindings.tool_secondary))`
   - `"[F] Talk to {}"` → `format!("[{}] Talk to {}", key_display(bindings.interact), name)`

## Do NOT
- Modify src/shared/mod.rs
- Modify any files outside src/ui/
- Change game logic or behavior
- Add new UI screens or features

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write a brief report to status/workers/fix-ui-prompts-icons.md listing:
- Lines changed in hud.rs
- Old vs new icon size
- Which prompt strings were updated
