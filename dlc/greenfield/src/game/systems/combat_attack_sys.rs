use bevy::prelude::*;
use crate::game::events::AttackStartedEvent;

pub fn combat_attack_system(mut events: EventReader<AttackStartedEvent>) {
    let _attacks = events.read().count();
}
