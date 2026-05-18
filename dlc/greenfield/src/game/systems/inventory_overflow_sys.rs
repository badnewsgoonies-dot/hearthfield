use bevy::prelude::*;
use crate::game::events::InventoryOverflowEvent;

pub fn inventory_overflow_system(mut events: EventReader<InventoryOverflowEvent>) {
    let _drained = events.read().count();
}
