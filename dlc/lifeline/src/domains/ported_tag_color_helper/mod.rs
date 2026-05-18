//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



pub fn tag_color_helper(tag: &str) -> Color {
    match tag {
        "Today" => Color::srgb(1.0, 1.0, 0.8),
        "Tomorrow" => Color::srgb(0.8, 0.92, 1.0),
        "Yesterday" => Color::srgb(0.72, 0.76, 0.76),
        "Birthday" => Color::srgb(0.7, 0.85, 1.0),
        "Festival Soon" => Color::srgb(1.0, 0.7, 0.3),
        _ => Color::srgb(0.8, 0.8, 0.8),
    }
}


