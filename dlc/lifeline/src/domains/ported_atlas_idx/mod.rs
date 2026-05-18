//! Single-fn substrate port — pure/near-pure.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;



pub fn supply_atlas_index(drop_item: &str) -> usize {
    match drop_item {
        "copper_ore" => 8,
        "iron_ore" => 9,
        "gold_ore" => 11,
        "iridium_ore" => 10,
        "diamond" => 22,
        "ruby" => 19,
        "emerald" => 20,
        "quartz" => 16,
        "amethyst" => 17,
        _ => 0,
    }
}


