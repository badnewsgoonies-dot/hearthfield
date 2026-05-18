use bevy::prelude::*;
use crate::game::events::ItemConsumedEvent;

pub fn inventory_consume_system(mut events: EventReader<ItemConsumedEvent>) {
    let _drained = events.read().count();
}
