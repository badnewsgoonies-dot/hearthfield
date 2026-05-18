//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



pub fn format_credits(gold: u32, materials: &[(&str, u8)]) -> String {
    let mut parts = vec![format!("{}g", gold)];
    for &(mat, qty) in materials {
        parts.push(format!("{} {}", qty, mat));
    }
    parts.join(" + ")
}


