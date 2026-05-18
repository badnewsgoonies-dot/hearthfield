use bevy::prelude::*;
use crate::game::events::EnemyDefeatedEvent;

pub fn drop_loot_system(mut reader: EventReader<EnemyDefeatedEvent>) {
    for _ev in reader.read() {
        info!("loot dropped: enemy defeated");
    }
}
