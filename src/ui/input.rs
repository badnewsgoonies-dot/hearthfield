use bevy::prelude::*;
use crate::shared::*;

/// Legacy global input handler — no longer registered.
/// All input routing is handled by `src/input/mod.rs` (InputPlugin)
/// and `src/ui/menu_input.rs` (MenuAction).
pub fn global_input_handler(
    _player_input: Res<PlayerInput>,
    _current_state: Res<State<GameState>>,
    _next_state: ResMut<NextState<GameState>>,
) {
    // Intentionally empty — superseded by InputPlugin + menu_input.
}

/// Legacy hotbar input handler — no longer registered.
/// Hotbar slot selection is now driven by `PlayerInput::tool_slot`.
pub fn hotbar_input_handler(
    _player_input: Res<PlayerInput>,
    _inventory: ResMut<Inventory>,
) {
    // Intentionally empty — superseded by InputPlugin.
}
