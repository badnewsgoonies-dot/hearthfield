//! Input abstraction layer.
//!
//! Runs in `PreUpdate`, converts raw keyboard/mouse into `PlayerInput` resource.
//! All other systems read `PlayerInput` instead of `ButtonInput<KeyCode>` directly.

use bevy::prelude::*;
use crate::shared::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, reset_and_read_input)
            .add_systems(OnEnter(GameState::Playing), set_gameplay_context)
            .add_systems(OnEnter(GameState::Flying), set_cockpit_context);
    }
}

pub fn set_gameplay_context(mut input_state: ResMut<InputState>) {
    input_state.context = InputContext::Gameplay;
}

pub fn set_cockpit_context(mut input_state: ResMut<InputState>) {
    input_state.context = InputContext::Cockpit;
}

pub fn reset_and_read_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    mut input: ResMut<PlayerInput>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
    input_state: Res<InputState>,
) {
    interaction_claimed.0 = false;

    // Reset all fields
    *input = PlayerInput::default();

    match input_state.context {
        InputContext::Gameplay => {
            input.movement = Vec2::new(
                if keyboard.pressed(bindings.move_right) { 1.0 } else { 0.0 }
                    - if keyboard.pressed(bindings.move_left) { 1.0 } else { 0.0 },
                if keyboard.pressed(bindings.move_up) { 1.0 } else { 0.0 }
                    - if keyboard.pressed(bindings.move_down) { 1.0 } else { 0.0 },
            );
            input.interact = keyboard.just_pressed(bindings.interact);
            input.cancel = keyboard.just_pressed(bindings.cancel);
            input.pause = keyboard.just_pressed(bindings.pause);
            input.inventory = keyboard.just_pressed(bindings.inventory);
            input.map_view = keyboard.just_pressed(bindings.map_view);
            input.radio = keyboard.just_pressed(bindings.radio);
            input.sprint = keyboard.pressed(bindings.sprint);
        }
        InputContext::Cockpit => {
            input.throttle_up = keyboard.pressed(bindings.throttle_up);
            input.throttle_down = keyboard.pressed(bindings.throttle_down);
            input.yaw_left = keyboard.pressed(bindings.yaw_left);
            input.yaw_right = keyboard.pressed(bindings.yaw_right);
            input.flaps_toggle = keyboard.just_pressed(bindings.flaps);
            input.gear_toggle = keyboard.just_pressed(bindings.gear);
            input.interact = keyboard.just_pressed(bindings.interact);
            input.pause = keyboard.just_pressed(bindings.pause);
            input.radio = keyboard.just_pressed(bindings.radio);
            input.map_view = keyboard.just_pressed(bindings.map_view);
        }
        InputContext::Menu => {
            input.menu_up = keyboard.just_pressed(bindings.move_up);
            input.menu_down = keyboard.just_pressed(bindings.move_down);
            input.menu_left = keyboard.just_pressed(bindings.move_left);
            input.menu_right = keyboard.just_pressed(bindings.move_right);
            input.menu_confirm = keyboard.just_pressed(bindings.interact)
                || keyboard.just_pressed(KeyCode::Enter);
            input.menu_cancel = keyboard.just_pressed(bindings.cancel)
                || keyboard.just_pressed(bindings.pause);
        }
        InputContext::Dialogue => {
            input.confirm = keyboard.just_pressed(bindings.interact)
                || keyboard.just_pressed(KeyCode::Enter)
                || keyboard.just_pressed(KeyCode::Space);
            input.cancel = keyboard.just_pressed(bindings.cancel);
            input.menu_up = keyboard.just_pressed(bindings.move_up);
            input.menu_down = keyboard.just_pressed(bindings.move_down);
        }
        InputContext::Cutscene => {
            input.confirm = keyboard.just_pressed(KeyCode::Space)
                || keyboard.just_pressed(KeyCode::Enter);
            input.cancel = keyboard.just_pressed(bindings.cancel);
        }
        InputContext::Disabled => {}
    }

    // Always available
    input.debug_overlay = keyboard.just_pressed(bindings.debug_overlay);
}
