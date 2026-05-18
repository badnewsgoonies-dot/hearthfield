use bevy::prelude::*;
use crate::game::events::EnemySpawnedEvent;
use crate::game::components::Enemy;

pub fn spawn_enemies_system(mut commands: Commands, mut reader: EventReader<EnemySpawnedEvent>) {
    for ev in reader.read() {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.85, 0.2, 0.2),
                custom_size: Some(Vec2::splat(24.0)),
                ..default()
            },
            Transform::from_xyz(ev.at_x, ev.at_y, 0.0),
            Enemy,
        ));
    }
}
