use crate::game::components::Enemy;
use crate::game::events::{CombatInitiatedEvent, PlayerMovedEvent};
use bevy::prelude::*;

pub fn input_tick(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut move_writer: EventWriter<PlayerMovedEvent>,
    mut combat_writer: EventWriter<CombatInitiatedEvent>,
    enemies: Query<Entity, With<Enemy>>,
) {
    let speed = 100.0_f32 * time.delta_secs();
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;
    if keyboard.pressed(KeyCode::KeyW) {
        dy += speed;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        dy -= speed;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        dx -= speed;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        dx += speed;
    }
    if dx != 0.0 || dy != 0.0 {
        move_writer.send(PlayerMovedEvent {
            x: dx,
            y: dy,
            z: 0.0,
        });
    }
    if keyboard.just_pressed(KeyCode::Space) {
        if let Some(enemy) = enemies.iter().next() {
            combat_writer.send(CombatInitiatedEvent { enemy });
        }
    }
}
