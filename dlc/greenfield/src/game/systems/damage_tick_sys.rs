use bevy::prelude::*;
use crate::game::events::{DamageDealtEvent, PlayerDamage};

pub fn damage_tick_system(mut reader: EventReader<DamageDealtEvent>, mut writer: EventWriter<PlayerDamage>) {
    for ev in reader.read() {
        let amt = ev.amount.max(0) as f32;
        if amt > 0.0 {
            writer.send(PlayerDamage { amount: amt });
        }
    }
}
