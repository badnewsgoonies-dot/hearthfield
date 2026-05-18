//! Single-fn substrate port — pure/despawn-shaped.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct FloatingLedgerText;


/// Despawns all floating gold text entities (called on HUD exit).
pub fn despawn_floating_ledger_text(
    mut commands: Commands,
    query: Query<Entity, With<FloatingLedgerText>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}


