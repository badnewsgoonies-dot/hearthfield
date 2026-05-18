//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



#[allow(dead_code)]
pub fn rare_case_sell_price(fish_id: &str) -> u32 {
    match fish_id {
        "legend_fish" => 5_000,
        "crimsonfish" => 1_500,
        "glacierfish" => 1_200,
        "frostfang" => 2_000,
        _ => 500,
    }
}


