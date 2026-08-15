use crate::game::components::Enemy;
use crate::game::events::EnemySpawnedEvent;
use crate::game::systems::tombstone_sys::EnemyKeyAllocator;
use bevy::prelude::*;

pub fn spawn_enemies_system(
    mut commands: Commands,
    mut reader: EventReader<EnemySpawnedEvent>,
    mut enemy_keys: ResMut<EnemyKeyAllocator>,
) {
    for ev in reader.read() {
        let Some(enemy_key) = enemy_keys.allocate() else {
            error!("enemy key field exhausted; external spawn refused");
            continue;
        };
        commands.spawn((
            Sprite {
                color: Color::srgb(0.85, 0.2, 0.2),
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            Transform::from_xyz(ev.at_x, ev.at_y, 0.0),
            Enemy,
            enemy_key,
        ));
    }
}
