use bevy::prelude::*;
use crate::game::resources;
use crate::game::events;
pub fn apply_damage(mut health: ResMut<resources::PlayerHealth>, mut damage_events: EventReader<events::PlayerDamage>) {
    for event in damage_events.read() {
        health.hp = (health.hp - event.amount).max(0.0);
    }

}
