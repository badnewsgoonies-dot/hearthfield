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


/// Pixel offset from player center for the held tool, based on facing direction.
pub fn tool_hand_offset_helper(facing: &Facing) -> Vec2 {
    match facing {
        Facing::Down => Vec2::new(6.0, -4.0),
        Facing::Up => Vec2::new(6.0, 4.0),
        Facing::Left => Vec2::new(-7.0, 0.0),
        Facing::Right => Vec2::new(7.0, 0.0),
    }
}


