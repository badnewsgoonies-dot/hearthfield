use bevy::prelude::*;
use crate::game::events::PlayerMovedEvent;

pub fn input_tick(keyboard: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut writer: EventWriter<PlayerMovedEvent>) {
    let speed = 100.0_f32 * time.delta_secs();
    let mut dx = 0.0_f32;
    let mut dy = 0.0_f32;
    if keyboard.pressed(KeyCode::KeyW) { dy += speed; }
    if keyboard.pressed(KeyCode::KeyS) { dy -= speed; }
    if keyboard.pressed(KeyCode::KeyA) { dx -= speed; }
    if keyboard.pressed(KeyCode::KeyD) { dx += speed; }
    if dx != 0.0 || dy != 0.0 {
        writer.send(PlayerMovedEvent { x: dx, y: dy, z: 0.0 });
    }
}
