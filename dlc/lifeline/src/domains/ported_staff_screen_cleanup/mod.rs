//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct RelationshipsScreenRoot;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct RelationshipsUiState;


pub fn despawn_staff_screen(
    mut commands: Commands,
    query: Query<Entity, With<RelationshipsScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<RelationshipsUiState>();
}


