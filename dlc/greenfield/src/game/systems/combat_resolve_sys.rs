use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Resolve a `BuildingImage` variant to the corresponding loaded image handle.
pub fn combat_resolve_system(img: BuildingImage, atlases: &ObjectAtlases) -> Handle<Image> {
    match img {
        BuildingImage::Farmhouse => atlases.farmhouse_image.clone(),
        BuildingImage::Barn => atlases.barn_image.clone(),
        BuildingImage::ChickenHouse => atlases.chicken_house_image.clone(),
        BuildingImage::Well => atlases.well_image.clone(),
    }
}


