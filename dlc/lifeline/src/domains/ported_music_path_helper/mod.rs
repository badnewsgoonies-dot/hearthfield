//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;



/// Maps music track IDs to actual audio file paths.
pub fn music_path_helper(track_id: &str) -> Option<&'static str> {
    match track_id {
        "farm" | "spring" => Some("audio/music/pixel_1.ogg"),
        "summer" => Some("audio/music/pixel_2.ogg"),
        "fall" => Some("audio/music/pixel_3.ogg"),
        "winter" => Some("audio/music/pixel_4.ogg"),
        "town" => Some("audio/music/pixel_5.ogg"),
        "mine" | "mine_ambient" => Some("audio/music/pixel_6.ogg"),
        "forest" => Some("audio/music/pixel_7.ogg"),
        "indoor" => Some("audio/music/pixel_1.ogg"),
        "beach" => Some("audio/music/pixel_8.ogg"),
        "menu" => Some("audio/music/pixel_9.ogg"),
        "night" => Some("audio/music/pixel_10.ogg"),
        "festival" => Some("audio/music/pixel_11.ogg"),
        "credits" => Some("audio/music/pixel_12.ogg"),
        _ => None,
    }
}


