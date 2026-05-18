use bevy::prelude::*;
use crate::game::events::ItemPickedUpInvEvent;

pub fn inventory_pickup_system(mut events: EventReader<ItemPickedUpInvEvent>) {
    let _drained = events.read().count();
}
