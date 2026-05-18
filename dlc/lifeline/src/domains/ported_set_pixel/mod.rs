//! Single-fn substrate port — pure/near-pure.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;



pub fn set_chart_pixel(data: &mut [u8], w: usize, x: usize, y: usize, rgba: [u8; 4]) {
    if x < w && y < w {
        let i = (y * w + x) * 4;
        data[i] = rgba[0];
        data[i + 1] = rgba[1];
        data[i + 2] = rgba[2];
        data[i + 3] = rgba[3];
    }
}


