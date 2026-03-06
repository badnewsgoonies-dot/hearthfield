use crate::shared::*;
use bevy::prelude::*;
use super::{
    calendar_screen::CalendarOverlayState, settings_screen::SettingsOverlayState,
    stats_screen::StatsOverlayState,
};

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
    calendar_overlay: Res<CalendarOverlayState>,
    stats_overlay: Res<StatsOverlayState>,
    settings_overlay: Res<SettingsOverlayState>,
) {
    if *state.get() != GameState::Playing {
        return;
    }
    if calendar_overlay.visible || stats_overlay.visible || settings_overlay.visible {
        return;
    }
    if input.pause {
        next.set(GameState::Paused);
    }
    if input.open_inventory {
        next.set(GameState::Inventory);
    }
    if input.open_journal {
        next.set(GameState::Journal);
    }
    if input.open_relationships {
        next.set(GameState::RelationshipsView);
    }
    if input.open_map {
        next.set(GameState::MapView);
    }
}

/// Universal "cancel goes back to Playing" for overlay menus.
/// If a cutscene is active and we're in Dialogue, return to Cutscene instead.
pub fn menu_cancel_transitions(
    action: Res<MenuAction>,
    input: Res<PlayerInput>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    cutscene_queue: Res<CutsceneQueue>,
) {
    // Toggle-close: pressing the same key that opened a menu closes it
    match *state.get() {
        GameState::Inventory if input.open_inventory => {
            next.set(GameState::Playing);
            return;
        }
        GameState::Crafting if input.open_crafting => {
            next.set(GameState::Playing);
            return;
        }
        GameState::Journal if input.open_journal => {
            next.set(GameState::Playing);
            return;
        }
        GameState::RelationshipsView if input.open_relationships => {
            next.set(GameState::Playing);
            return;
        }
        GameState::MapView if input.open_map => {
            next.set(GameState::Playing);
            return;
        }
        _ => {}
    }

    if !action.cancel {
        return;
    }
    match *state.get() {
        GameState::Dialogue if cutscene_queue.active => {
            next.set(GameState::Cutscene);
        }
        GameState::Inventory
        | GameState::Shop
        | GameState::Crafting
        | GameState::Dialogue
        | GameState::Journal
        | GameState::RelationshipsView
        | GameState::MapView => {
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
    }

    // Also support 0, -, = keys for slots 9-11 via tool_next/tool_prev as fallback
    // These are handled in the main input reader via digit keys already
}
