//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FarmObject {
    #[default]
    Fence,
    Scarecrow,
    Sprinkler,
}


/// Fallback placeholder colour for farm objects when no atlas is available.
pub fn ward_object_color(obj: &FarmObject) -> Color {
    match obj {
        FarmObject::Sprinkler => Color::srgb(0.5, 0.5, 0.7),
        FarmObject::Scarecrow => Color::srgb(0.6, 0.4, 0.2),
        FarmObject::Fence => Color::srgb(0.6, 0.4, 0.2),
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}


