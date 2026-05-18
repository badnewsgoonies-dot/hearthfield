use bevy::prelude::*;
use crate::game::events::DamageDealtEvent;

pub fn drain_damage_system(mut events: EventReader<DamageDealtEvent>) {
    let _drained: u32 = events.read().map(|e| e.amount.max(0) as u32).sum();
}
