# Domain Spec: Input

## Scope
`src/input/` — `mod.rs`

## Responsibility
Read raw keyboard/mouse input, write to `PlayerInput` resource based on active `InputContext`. Reset `PlayerInput` each frame. Handle `InteractionClaimed` reset.

## Shared Contract Types (import from `crate::shared`)
- `PlayerInput` (Resource — all input fields)
- `InputContext` (Resource — Gameplay, Menu, Dialogue, Fishing, Cutscene, Disabled)
- `KeyBindings` (Resource — configurable key mappings)
- `InputBlocks` (Resource — check if input should be suppressed)
- `InteractionClaimed` (Resource — reset each frame)
- `MenuAction` (Resource — set_cursor, activate, cancel, move directions)
- `GameState`

## Quantitative Targets
- Key bindings: 17 mappable keys (WASD, F, Space, R, E, C, M, J, Esc, ] [, Enter, Tab, F5, F9)
- Input contexts: 5 (Gameplay, Menu, Dialogue, Fishing, Cutscene) + Disabled
- Frame-accurate: all `just_pressed` fields reflect single-frame presses only

## Key Systems
1. `reset_input` — clear all `PlayerInput` fields to defaults at frame start
2. `read_keyboard` — read Bevy `ButtonInput<KeyCode>`, map to `PlayerInput` based on `InputContext`
3. `read_mouse` — handle mouse clicks for tool use, menu interaction
4. `reset_interaction_claimed` — set `InteractionClaimed(false)` each frame
5. `update_menu_action` — translate keyboard navigation to `MenuAction` in Menu context

## Does NOT Handle
- Touch/gamepad input (future feature)
- Key rebinding UI (ui domain)
- Context switching (other domains set `InputContext` when entering states)
- Actual action execution (all domains consume `PlayerInput`)
