use bevy::prelude::*;
use crate::game::events::{PlayerMovedEvent, DamageDealtEvent};
use crate::game::components::{PlayerMarker, Enemy};

pub fn sim_tick(mut reader: EventReader<PlayerMovedEvent>, mut player_query: Query<&mut Transform, (With<PlayerMarker>, Without<Enemy>)>, enemies: Query<&Transform, (With<Enemy>, Without<PlayerMarker>)>, mut damage_writer: EventWriter<DamageDealtEvent>, mut hit_cooldown: Local<f32>, time: Res<Time>) {
    for event in reader.read() {
        for mut transform in &mut player_query {
            transform.translation.x += event.x;
            transform.translation.y += event.y;
        }
    }
    *hit_cooldown = (*hit_cooldown - time.delta_secs()).max(0.0);
    // collision: any enemy within 28px of player triggers 5 dmg/tick
    let player_pos = match player_query.iter().next() {
        Some(t) => t.translation.truncate(),
        None => return,
    };
    if *hit_cooldown > 0.0 {
        return;
    }
    for enemy_transform in &enemies {
        let to_enemy = enemy_transform.translation.truncate() - player_pos;
        if to_enemy.length() < 28.0 {
            damage_writer.send(DamageDealtEvent { amount: 5 });
            *hit_cooldown = 0.5;
            break;
        }
    }
}
