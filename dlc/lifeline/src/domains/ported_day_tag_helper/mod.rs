//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



pub fn day_tag_helper(
    day: u8,
    current_day: u8,
    has_birthday: bool,
    upcoming_festival_day: Option<u8>,
) -> Option<&'static str> {
    if day == current_day {
        Some("Today")
    } else if day + 1 == current_day {
        Some("Yesterday")
    } else if day == current_day + 1 {
        Some("Tomorrow")
    } else if has_birthday {
        Some("Birthday")
    } else if upcoming_festival_day == Some(day) {
        Some("Festival Soon")
    } else {
        None
    }
}


