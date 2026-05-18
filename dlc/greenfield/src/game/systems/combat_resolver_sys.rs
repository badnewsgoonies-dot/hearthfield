use bevy::prelude::*;
use crate::game::events::EnemyDefeatedEvent;

pub fn resolve_combat_system(mut events: EventReader<EnemyDefeatedEvent>) {
    let _defeated = events.read().count();
}
