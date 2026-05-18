use bevy::prelude::*;
use crate::game::events::EnemyDefeatedEvent;

pub fn drop_loot_system(mut events: EventReader<EnemyDefeatedEvent>) {
    let _drained = events.read().count();
}
