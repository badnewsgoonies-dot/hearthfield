//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct ShiftOverlay;


/// Despawn the overlay when leaving Playing state.
pub fn despawn_shift_overlay(
    mut commands: Commands,
    query: Query<Entity, With<ShiftOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}


