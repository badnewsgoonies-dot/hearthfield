use bevy::prelude::*;
use crate::game::events::CombatResolvedEvent;

pub fn combat_resolve_system(mut events: EventReader<CombatResolvedEvent>) {
    let _resolved = events.read().count();
}
