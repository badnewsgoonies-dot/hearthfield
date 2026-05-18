//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GiftPreference {
    #[default]
    Disliked,
    Hated,
    Liked,
    Loved,
    Neutral,
}


/// Build the toast message shown to the player after giving a gift.
pub fn preference_toast_message_helper(npc_name: &str, preference: GiftPreference, points: i32) -> String {
    match preference {
        GiftPreference::Loved => format!(
            "{} loved your gift! \u{2665}\u{2665}\u{2665} (+{})",
            npc_name, points
        ),
        GiftPreference::Liked => format!(
            "{} liked your gift! \u{2665}\u{2665} (+{})",
            npc_name, points
        ),
        GiftPreference::Neutral => format!("{} accepted your gift. (+{})", npc_name, points),
        GiftPreference::Disliked => {
            format!("{} didn't seem to like that... ({})", npc_name, points)
        }
        GiftPreference::Hated => format!("{} hated that gift! ({})", npc_name, points),
    }
}


