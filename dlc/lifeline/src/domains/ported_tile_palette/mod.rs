//! Single-fn substrate port — pure/near-pure.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;



pub fn fallback_palette(floor: u8) -> (Color, Color, f32) {
    match floor {
        1..=5 => (
            Color::srgb(0.18, 0.16, 0.18),
            Color::srgb(0.11, 0.09, 0.11),
            0.035,
        ),
        6..=10 => (
            Color::srgb(0.14, 0.13, 0.17),
            Color::srgb(0.08, 0.08, 0.11),
            0.025,
        ),
        11..=15 => (
            Color::srgb(0.11, 0.12, 0.16),
            Color::srgb(0.07, 0.08, 0.12),
            0.015,
        ),
        _ => (
            Color::srgb(0.08, 0.10, 0.14),
            Color::srgb(0.05, 0.07, 0.11),
            0.008,
        ),
    }
}


