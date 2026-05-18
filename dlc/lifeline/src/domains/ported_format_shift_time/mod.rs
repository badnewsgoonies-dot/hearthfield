//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



pub fn format_shift_time(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}


