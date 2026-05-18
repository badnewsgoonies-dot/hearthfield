//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Read D-pad as just_pressed for UI navigation.
pub fn read_dpad_just_pressed_helper(gamepad: &Gamepad) -> (bool, bool, bool, bool) {
    (
        gamepad.just_pressed(GamepadButton::DPadUp),
        gamepad.just_pressed(GamepadButton::DPadDown),
        gamepad.just_pressed(GamepadButton::DPadLeft),
        gamepad.just_pressed(GamepadButton::DPadRight),
    )
}


