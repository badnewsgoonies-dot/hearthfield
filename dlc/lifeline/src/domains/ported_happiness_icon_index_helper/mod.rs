//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Map animal happiness (0–255) to an icon index in icons_happiness.png (row 1, saturated).
/// Col 0 = lowest mood, col 5 = highest mood.
pub fn happiness_icon_index_helper(happiness: u8) -> usize {
    let col = match happiness {
        0..=49 => 0,
        50..=99 => 1,
        100..=149 => 2,
        150..=199 => 3,
        200..=229 => 4,
        230..=u8::MAX => 5,
    };
    6 + col // row 1 (saturated variants) = offset by 6 cols
}


