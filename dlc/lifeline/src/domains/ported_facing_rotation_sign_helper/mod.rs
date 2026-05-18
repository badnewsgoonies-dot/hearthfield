//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Facing {
    #[default]
    Left,
}


/// Mirror rotation direction based on facing. Left-facing tools swing
/// the opposite way to maintain visual consistency.
pub fn facing_rotation_sign_helper(facing: &Facing) -> f32 {
    match facing {
        Facing::Left => -1.0,
        _ => 1.0,
    }
}


