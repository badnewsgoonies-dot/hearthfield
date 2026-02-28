use bevy::prelude::*;
use crate::shared::*;

/// Reset MenuAction at frame start (before observers fire).
pub fn reset_menu_action(mut action: ResMut<MenuAction>) {
    *action = MenuAction::default();
}

/// Merge keyboard PlayerInput into MenuAction. Pointer observers already wrote
/// set_cursor and activate directly. This merges keyboard navigation on top.
pub fn merge_keyboard_to_menu_action(input: Res<PlayerInput>, mut action: ResMut<MenuAction>) {
    action.move_up = action.move_up || input.ui_up;
    action.move_down = action.move_down || input.ui_down;
    action.move_left = action.move_left || input.ui_left;
    action.move_right = action.move_right || input.ui_right;
    action.activate = action.activate || input.ui_confirm;
    action.cancel = action.cancel || input.ui_cancel || input.pause;
}

/// State transitions driven by PlayerInput (gameplay context).
/// Replaces the old global_input_handler in ui/input.rs.
pub fn gameplay_state_transitions(
    input: Res<PlayerInput>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    if input.pause {
        next.set(GameState::Paused);
    }
    if input.open_inventory {
        next.set(GameState::Inventory);
    }
}

/// Universal "cancel goes back to Playing" for overlay menus.
pub fn menu_cancel_transitions(
    action: Res<MenuAction>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    if !action.cancel {
        return;
    }
    match *state.get() {
        GameState::Inventory | GameState::Shop | GameState::Crafting | GameState::Dialogue => {
            next.set(GameState::Playing);
        }
        _ => {}
    }
}

/// Hotbar slot selection driven by PlayerInput.
/// Replaces hotbar_input_handler in ui/input.rs.
pub fn hotbar_input_handler(input: Res<PlayerInput>, mut inventory: ResMut<Inventory>) {
    // tool_slot maps 1-9 keys → Some(0..8)
    if let Some(slot) = input.tool_slot {
        inventory.selected_slot = slot as usize;
        return;
    }

    // Also support 0, -, = keys for slots 9-11 via tool_next/tool_prev as fallback
    // These are handled in the main input reader via digit keys already
}
