use bevy::prelude::*;
use crate::game::events::DamageDealtEvent;

pub fn drain_damage_system(mut events: EventReader<DamageDealtEvent>) {
    for ev in events.read() {
        let _ = ev;
    }
}
