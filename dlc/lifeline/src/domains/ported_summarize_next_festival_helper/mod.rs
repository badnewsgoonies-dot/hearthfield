//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



pub fn summarize_next_festival_helper(current_day: u8, festival: Option<(u8, &'static str)>) -> String {
    match festival {
        Some((festival_day, festival_name)) if festival_day == current_day => {
            format!("Today • {festival_name}")
        }
        Some((festival_day, festival_name)) if festival_day == current_day + 1 => {
            format!("Tomorrow • {festival_name}")
        }
        Some((festival_day, festival_name)) => format!("Day {festival_day} • {festival_name}"),
        None => "Passed this season".to_string(),
    }
}


