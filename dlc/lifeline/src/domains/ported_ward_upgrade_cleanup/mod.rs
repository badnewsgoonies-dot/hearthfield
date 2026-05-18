//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct BuildingUpgradeMenuRoot;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct BuildingUpgradeMenuState;


pub fn despawn_ward_upgrade_menu(
    mut commands: Commands,
    query: Query<Entity, With<BuildingUpgradeMenuRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<BuildingUpgradeMenuState>();
}


