use crate::game::components::{Enemy, PlayerMarker};
use crate::game::events::{CombatInitiatedEvent, HeartbeatPulseEvent};
use bevy::prelude::*;

type MovingEnemies<'w, 's> =
    Query<'w, 's, (Entity, &'static mut Transform), (With<Enemy>, Without<PlayerMarker>)>;

pub fn heartbeat_pulse_system(
    mut writer: EventWriter<HeartbeatPulseEvent>,
    mut combat_writer: EventWriter<CombatInitiatedEvent>,
    time: Res<Time>,
    mut beat_clock: Local<f32>,
    mut combat_clock: Local<f32>,
    mut enemies: MovingEnemies,
    player: Query<&Transform, With<PlayerMarker>>,
) {
    writer.send(HeartbeatPulseEvent);
    let dt = time.delta_secs();
    *beat_clock += dt;
    *combat_clock += dt;
    // enemies drift toward player at 40 px/s
    let player_pos = match player.iter().next() {
        Some(t) => t.translation.truncate(),
        None => return,
    };
    let drift_speed = 40.0_f32 * dt;
    let mut combat_target = None;
    for (enemy, mut enemy_transform) in &mut enemies {
        combat_target.get_or_insert(enemy);
        let to_player = player_pos - enemy_transform.translation.truncate();
        let dist = to_player.length();
        if dist > 1.0 {
            let dir = to_player / dist;
            enemy_transform.translation.x += dir.x * drift_speed;
            enemy_transform.translation.y += dir.y * drift_speed;
        }
    }
    // every 2 seconds fire a combat tick against the identity selected here;
    // every downstream event carries this same target.
    if *combat_clock >= 2.0 {
        *combat_clock -= 2.0;
        if let Some(enemy) = combat_target {
            combat_writer.send(CombatInitiatedEvent { enemy });
        }
    }
}
