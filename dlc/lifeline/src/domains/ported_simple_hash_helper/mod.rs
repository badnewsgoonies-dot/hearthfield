//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Simple deterministic hash for per-NPC variation (avoids rand dependency).
pub fn simple_hash_helper(id: &str) -> u32 {
    let mut h: u32 = 5381;
    for byte in id.bytes() {
        h = h.wrapping_mul(33).wrapping_add(byte as u32);
    }
    h
}


