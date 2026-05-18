//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FarmObject {
    #[default]
    Scarecrow,
    Sprinkler,
}


/// Map a FarmObject to an atlas index in furniture.png (9 cols × 6 rows).
/// Returns None for Fence (handled separately with fences atlas + autotile).
/// Retained for potential future atlas-based rendering of farm objects.
#[allow(dead_code)]
pub fn ward_object_atlas_index(obj: &FarmObject) -> Option<usize> {
    match obj {
        FarmObject::Sprinkler => Some(36), // row 4: machinery/device
        FarmObject::Scarecrow => Some(45), // row 5: tall object
        _ => None,
    }
}


