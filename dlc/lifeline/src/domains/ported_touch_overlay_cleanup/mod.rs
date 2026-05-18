//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct TouchControlsOverlay;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct TouchOverlayTimer;


/// Despawns the touch controls overlay (called on HUD exit).
pub fn despawn_touch_overlay(
    mut commands: Commands,
    query: Query<Entity, With<TouchControlsOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<TouchOverlayTimer>();
}

