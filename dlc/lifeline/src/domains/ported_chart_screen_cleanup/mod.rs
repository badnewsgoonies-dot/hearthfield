//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct JournalScreenRoot;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct JournalUiState;


pub fn despawn_chart_screen(
    mut commands: Commands,
    query: Query<Entity, With<JournalScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<JournalUiState>();
}


