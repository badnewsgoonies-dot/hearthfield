//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Helper: write an RGBA pixel into the data buffer at (x, y) for a 16-wide image.
#[allow(dead_code)]
pub fn put_pixel_helper(data: &mut [u8], x: usize, y: usize, r: u8, g: u8, b: u8, a: u8) {
    let i = (y * 16 + x) * 4;
    if i + 3 < data.len() {
        data[i] = r;
        data[i + 1] = g;
        data[i + 2] = b;
        data[i + 3] = a;
    }
}


