//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Read D-pad as a Vec2 for movement (digital, -1/0/+1 per axis).
pub fn read_dpad_axis_helper(gamepad: &Gamepad) -> Vec2 {
    let mut axis = Vec2::ZERO;
    if gamepad.pressed(GamepadButton::DPadUp) {
        axis.y += 1.0;
    }
    if gamepad.pressed(GamepadButton::DPadDown) {
        axis.y -= 1.0;
    }
    if gamepad.pressed(GamepadButton::DPadLeft) {
        axis.x -= 1.0;
    }
    if gamepad.pressed(GamepadButton::DPadRight) {
        axis.x += 1.0;
    }
    axis
}


