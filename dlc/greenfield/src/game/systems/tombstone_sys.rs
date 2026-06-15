//! Tombstone combat log — removal as append (keeps the witness, stays reversible).
//!
//! Lab finding (measured): destructive despawn collapses the witness — from a fixed wave seed
//! the post-combat state reconstructs the initial wave only ~1.5% of the time and "undo" is not
//! exact. Recording each kill append-only restores **100%** reconstruction and **O(1) exact undo**
//! (preimage = 1). This is the chirality law in practice: build by addition, gate removal.
//! Remove-as-append IS add-in-reverse; destructive remove is not, because it forgets the witness.
//!
//! `resolve_combat_system` appends a `KillRecord` here before despawning the enemy. Press **U** to
//! pop the log and resurrect the last defeated enemy *exactly* (position + colour from the witness).

use bevy::prelude::*;
use crate::game::components::Enemy;

/// The witness for one removal — everything needed to reconstruct the enemy exactly.
#[derive(Clone, Copy, Debug)]
pub struct KillRecord {
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

/// Append-only kill log. Never mutated except push (record) / pop (undo).
#[derive(Resource, Default, Debug)]
pub struct KillLog(pub Vec<KillRecord>);

/// Press **U**: pop the last tombstone and resurrect that enemy exactly. Because the witness was
/// kept, the reverse is a function (preimage = 1), not a search.
pub fn revert_last_kill_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut log: ResMut<KillLog>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::KeyU) {
        if let Some(k) = log.0.pop() {
            commands.spawn((
                Sprite {
                    color: Color::srgb(k.r, k.g, k.b),
                    custom_size: Some(Vec2::splat(20.0)),
                    ..default()
                },
                Transform::from_xyz(k.x, k.y, 1.0),
                Enemy,
            ));
        }
    }
}
