use bevy::prelude::*;
use crate::game::events::ItemStackedEvent;

pub fn inventory_stack_system(mut events: EventReader<ItemStackedEvent>) {
    let _drained = events.read().count();
}
