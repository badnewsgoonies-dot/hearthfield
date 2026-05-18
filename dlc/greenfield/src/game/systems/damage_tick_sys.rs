use bevy::prelude::*;
use crate::game::events::DamageDealtEvent;

pub fn damage_tick_system(mut events: EventReader<DamageDealtEvent>) {
    let _drained = events.read().count();
}
