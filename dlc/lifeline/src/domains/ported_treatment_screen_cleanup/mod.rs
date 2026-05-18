//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct CraftingScreenRoot;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct CraftingUiState;


pub fn despawn_treatment_screen(
    mut commands: Commands,
    query: Query<Entity, With<CraftingScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<CraftingUiState>();
}


