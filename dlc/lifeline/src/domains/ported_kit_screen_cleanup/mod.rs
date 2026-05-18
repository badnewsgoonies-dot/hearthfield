//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct InventoryScreenRoot;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct InventoryUiState;


pub fn despawn_kit_screen(
    mut commands: Commands,
    query: Query<Entity, With<InventoryScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<InventoryUiState>();
}


