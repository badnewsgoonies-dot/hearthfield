//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Facing {
    #[default]
    Down,
    Left,
    Right,
    Up,
}


/// Get the facing-direction offset as a grid delta.
pub fn facing_offset_player(facing: &Facing) -> (i32, i32) {
    match facing {
        Facing::Up => (0, 1),
        Facing::Down => (0, -1),
        Facing::Left => (-1, 0),
        Facing::Right => (1, 0),
    }
}

