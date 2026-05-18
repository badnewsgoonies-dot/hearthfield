use bevy::prelude::*;
use crate::game::events::ItemDroppedInvEvent;

pub fn inventory_drop_system(mut events: EventReader<ItemDroppedInvEvent>) {
    let _drained = events.read().count();
}
