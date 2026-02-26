use bevy::prelude::*;
use crate::shared::*;

/// Handles global input for state transitions.
/// - Escape: Playing -> Paused; any overlay state -> Playing
/// - Tab or I: Playing <-> Inventory
pub fn global_input_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let state = *current_state.get();

    match state {
        GameState::Playing => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Paused);
            }
            if keyboard.just_pressed(KeyCode::Tab) || keyboard.just_pressed(KeyCode::KeyI) {
                next_state.set(GameState::Inventory);
            }
        }
        GameState::Inventory => {
            if keyboard.just_pressed(KeyCode::Escape)
                || keyboard.just_pressed(KeyCode::Tab)
                || keyboard.just_pressed(KeyCode::KeyI)
            {
                next_state.set(GameState::Playing);
            }
        }
        GameState::Shop => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Playing);
            }
        }
        GameState::Crafting => {
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Playing);
            }
        }
        GameState::Dialogue => {
            // Dialogue handles its own closing via Space at end
            // But also allow Escape to force-close
            if keyboard.just_pressed(KeyCode::Escape) {
                next_state.set(GameState::Playing);
            }
        }
        // Paused and MainMenu handle their own escape in their own systems
        _ => {}
    }
}

/// Handles hotbar slot selection with number keys 1-9, 0, -, =
/// Maps to slots 0-11
pub fn hotbar_input_handler(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut inventory: ResMut<Inventory>,
) {
    let key_map: &[(KeyCode, usize)] = &[
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
        (KeyCode::Digit0, 9),
        (KeyCode::Minus, 10),
        (KeyCode::Equal, 11),
    ];

    for (key, slot) in key_map {
        if keyboard.just_pressed(*key) {
            inventory.selected_slot = *slot;
            return;
        }
    }

    // Scroll wheel for tool cycling could be added here
    // but would need mouse wheel input
}
