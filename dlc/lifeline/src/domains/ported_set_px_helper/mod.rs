//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Set a pixel at (px, py) in a flat RGBA buffer.
#[allow(clippy::too_many_arguments)]
pub fn set_px_helper(data: &mut [u8], w: usize, px: usize, py: usize, r: u8, g: u8, b: u8, a: u8) {
    let i = (py * w + px) * 4;
    data[i] = r;
    data[i + 1] = g;
    data[i + 2] = b;
    data[i + 3] = a;
}


