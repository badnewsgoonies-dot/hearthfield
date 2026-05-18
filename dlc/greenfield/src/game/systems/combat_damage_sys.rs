use bevy::prelude::*;
use crate::game::events::DamageAppliedEvent;

pub fn combat_damage_system(mut events: EventReader<DamageAppliedEvent>) {
    let _hits = events.read().count();
}
