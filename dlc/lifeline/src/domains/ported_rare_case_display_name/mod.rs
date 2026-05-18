//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



#[allow(dead_code)]
pub fn rare_case_display_name(fish_id: &str) -> &'static str {
    match fish_id {
        "legend_fish" => "Legend",
        "crimsonfish" => "Crimsonfish",
        "glacierfish" => "Glacierfish",
        "frostfang" => "Frostfang",
        _ => "Legendary Fish",
    }
}


