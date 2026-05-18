//! Single-fn substrate port — pure/near-pure.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;



pub fn ward_corridor_index(bitmask: u8) -> usize {
    match bitmask {
        0b0000 => 0,
        0b0001 => 1,
        0b0010 => 2,
        0b0011 => 3,
        0b0100 => 4,
        0b0101 => 5,
        0b0110 => 6,
        0b0111 => 7,
        0b1000 => 8,
        0b1001 => 9,
        0b1010 => 10,
        0b1011 => 11,
        0b1100 => 12,
        0b1101 => 13,
        0b1110 => 14,
        0b1111 => 15,
        _ => 5,
    }
}


