//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Clamp a carried edge coordinate to the interior of the destination map so
/// corner transitions don't strand the player on the receiving border.
pub fn clamp_to_ward(value: i32, min: i32, max: i32) -> i32 {
    if max - min >= 2 {
        value.clamp(min + 1, max - 1)
    } else {
        value.clamp(min, max)
    }
}


