use bevy::prelude::*;
use crate::game::events::MaterialConsumedEvent;

pub fn crafting_consume_system(mut events: EventReader<MaterialConsumedEvent>) {
    let _drained = events.read().count();
}
