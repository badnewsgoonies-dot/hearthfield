use bevy::prelude::*;
use crate::shared::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (reset_and_read_input, manage_input_context).chain(),
        );
    }
}

/// The single point where hardware input becomes game actions.
fn reset_and_read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    context: Res<InputContext>,
    mut input: ResMut<PlayerInput>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
) {
    *input = PlayerInput::default();
    interaction_claimed.0 = false;

    input.any_key =
        keys.get_just_pressed().next().is_some() || mouse.get_just_pressed().next().is_some();

    match *context {
        InputContext::Disabled => return,

        InputContext::Gameplay => {
            let mut axis = Vec2::ZERO;
            if keys.pressed(bindings.move_up) || keys.pressed(KeyCode::ArrowUp) {
                axis.y += 1.0;
            }
            if keys.pressed(bindings.move_down) || keys.pressed(KeyCode::ArrowDown) {
                axis.y -= 1.0;
            }
            if keys.pressed(bindings.move_left) || keys.pressed(KeyCode::ArrowLeft) {
                axis.x -= 1.0;
            }
            if keys.pressed(bindings.move_right) || keys.pressed(KeyCode::ArrowRight) {
                axis.x += 1.0;
            }
            input.move_axis = if axis != Vec2::ZERO {
                axis.normalize()
            } else {
                Vec2::ZERO
            };

            input.interact = keys.just_pressed(bindings.interact);
            input.tool_use =
                keys.just_pressed(bindings.tool_use) || mouse.just_pressed(MouseButton::Left);
            input.tool_secondary =
                keys.just_pressed(bindings.tool_secondary) || mouse.just_pressed(MouseButton::Right);
            input.attack = input.tool_use;

            input.open_inventory = keys.just_pressed(bindings.open_inventory);
            input.open_crafting = keys.just_pressed(bindings.open_crafting);
            input.open_map = keys.just_pressed(bindings.open_map);
            input.open_journal = keys.just_pressed(bindings.open_journal);
            input.pause = keys.just_pressed(bindings.pause);

            input.tool_next = keys.just_pressed(bindings.tool_next);
            input.tool_prev = keys.just_pressed(bindings.tool_prev);
            for (i, key) in [
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
                KeyCode::Digit6,
                KeyCode::Digit7,
                KeyCode::Digit8,
                KeyCode::Digit9,
            ]
            .iter()
            .enumerate()
            {
                if keys.just_pressed(*key) {
                    input.tool_slot = Some(i as u8);
                    break;
                }
            }

            // Quicksave / quickload
            input.quicksave = keys.just_pressed(KeyCode::F5);
            input.quickload = keys.just_pressed(KeyCode::F9);

            // UI navigation for in-game overlays (chest panel, elevator, etc.)
            // WASD + Arrows for consistency with Menu context.
            input.ui_up = keys.just_pressed(KeyCode::ArrowUp)
                || keys.just_pressed(bindings.move_up);
            input.ui_down = keys.just_pressed(KeyCode::ArrowDown)
                || keys.just_pressed(bindings.move_down);
            input.ui_left = keys.just_pressed(KeyCode::ArrowLeft)
                || keys.just_pressed(bindings.move_left);
            input.ui_right = keys.just_pressed(KeyCode::ArrowRight)
                || keys.just_pressed(bindings.move_right);
            input.tab_pressed = keys.just_pressed(KeyCode::Tab);
            input.ui_confirm = keys.just_pressed(bindings.ui_confirm);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);
        }

        InputContext::Menu => {
            input.ui_up =
                keys.just_pressed(bindings.move_up) || keys.just_pressed(KeyCode::ArrowUp);
            input.ui_down =
                keys.just_pressed(bindings.move_down) || keys.just_pressed(KeyCode::ArrowDown);
            input.ui_left =
                keys.just_pressed(bindings.move_left) || keys.just_pressed(KeyCode::ArrowLeft);
            input.ui_right =
                keys.just_pressed(bindings.move_right) || keys.just_pressed(KeyCode::ArrowRight);
            input.ui_confirm =
                keys.just_pressed(bindings.ui_confirm) || keys.just_pressed(bindings.interact);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);
            input.pause = keys.just_pressed(bindings.pause);
            input.tab_pressed = keys.just_pressed(KeyCode::Tab);

            // Quicksave / quickload available from pause menu
            input.quicksave = keys.just_pressed(KeyCode::F5);
            input.quickload = keys.just_pressed(KeyCode::F9);
        }

        InputContext::Dialogue => {
            input.interact = keys.just_pressed(bindings.interact)
                || keys.just_pressed(bindings.ui_confirm)
                || keys.just_pressed(KeyCode::Space);
            input.skip_cutscene = keys.just_pressed(bindings.skip_cutscene);
            input.ui_up =
                keys.just_pressed(bindings.move_up) || keys.just_pressed(KeyCode::ArrowUp);
            input.ui_down =
                keys.just_pressed(bindings.move_down) || keys.just_pressed(KeyCode::ArrowDown);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);
        }

        InputContext::Fishing => {
            input.fishing_reel =
                keys.pressed(bindings.tool_use) || mouse.pressed(MouseButton::Left);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);
            input.tool_use = keys.just_pressed(bindings.tool_use);
        }

        InputContext::Cutscene => {
            input.skip_cutscene = keys.just_pressed(bindings.skip_cutscene);
        }
    }
}

/// Derives InputContext from GameState. ONE system, replaces all per-domain guards.
fn manage_input_context(
    game_state: Res<State<GameState>>,
    mut context: ResMut<InputContext>,
) {
    *context = match *game_state.get() {
        GameState::MainMenu => InputContext::Menu,
        GameState::Playing => InputContext::Gameplay,
        GameState::Paused => InputContext::Menu,
        GameState::Inventory => InputContext::Menu,
        GameState::Shop => InputContext::Menu,
        GameState::Crafting => InputContext::Menu,
        GameState::BuildingUpgrade => InputContext::Menu,
        GameState::Dialogue => InputContext::Dialogue,
        GameState::Fishing => InputContext::Fishing,
        GameState::Cutscene => InputContext::Cutscene,
        GameState::Loading => InputContext::Disabled,
        _ => InputContext::Gameplay,
    };
}
