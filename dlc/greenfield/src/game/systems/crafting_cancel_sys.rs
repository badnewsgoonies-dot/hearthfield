use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Universal "cancel goes back to Playing" for overlay menus.
/// If a cutscene is active and we're in Dialogue, return to Cutscene instead.
pub fn crafting_cancel_system(
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


