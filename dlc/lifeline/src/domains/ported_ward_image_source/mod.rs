//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BuildingImage {
    #[default]
    Barn,
    ChickenHouse,
    Farmhouse,
    Well,
}


/// Source pixel dimensions for each composite building sprite.
pub fn ward_image_source(img: BuildingImage) -> Vec2 {
    match img {
        BuildingImage::Farmhouse => Vec2::new(128.0, 160.0),
        BuildingImage::Barn => Vec2::new(128.0, 160.0),
        BuildingImage::ChickenHouse => Vec2::new(48.0, 48.0),
        BuildingImage::Well => Vec2::new(48.0, 32.0),
    }
}


