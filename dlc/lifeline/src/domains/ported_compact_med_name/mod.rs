//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



pub fn compact_med_name(name: &str) -> String {
    if name.chars().count() <= 12 {
        name.to_string()
    } else {
        let truncated: String = name.chars().take(11).collect();
        format!("{truncated}…")
    }
}


