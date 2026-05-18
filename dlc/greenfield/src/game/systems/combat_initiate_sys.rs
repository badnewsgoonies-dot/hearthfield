use bevy::prelude::*;
use crate::game::events::CombatInitiatedEvent;

pub fn combat_initiate_system(mut events: EventReader<CombatInitiatedEvent>) {
    let _started = events.read().count();
}
