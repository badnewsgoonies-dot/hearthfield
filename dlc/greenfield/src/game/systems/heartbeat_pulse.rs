use bevy::prelude::*;
use crate::game::events::{HeartbeatPulseEvent, CombatInitiatedEvent};
use crate::game::components::{Enemy, PlayerMarker};

pub fn heartbeat_pulse_system(mut writer: EventWriter<HeartbeatPulseEvent>, mut combat_writer: EventWriter<CombatInitiatedEvent>, time: Res<Time>, mut beat_clock: Local<f32>, mut combat_clock: Local<f32>, mut enemies: Query<&mut Transform, (With<Enemy>, Without<PlayerMarker>)>, player: Query<&Transform, With<PlayerMarker>>) {
    writer.send(HeartbeatPulseEvent);
    let dt = time.delta_secs();
    *beat_clock += dt;
    *combat_clock += dt;
    // every 2 seconds fire a combat tick so the chain visibly progresses
    if *combat_clock >= 2.0 {
        *combat_clock -= 2.0;
        combat_writer.send(CombatInitiatedEvent);
    }
    // enemies drift toward player at 40 px/s
    let player_pos = match player.iter().next() {
        Some(t) => t.translation.truncate(),
        None => return,
    };
    let drift_speed = 40.0_f32 * dt;
    for mut enemy_transform in &mut enemies {
        let to_player = player_pos - enemy_transform.translation.truncate();
        let dist = to_player.length();
        if dist > 1.0 {
            let dir = to_player / dist;
            enemy_transform.translation.x += dir.x * drift_speed;
            enemy_transform.translation.y += dir.y * drift_speed;
        }
    }
}
