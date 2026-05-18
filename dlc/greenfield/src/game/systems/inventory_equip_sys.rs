use bevy::prelude::*;
use crate::game::events::ItemEquippedInvEvent;

pub fn inventory_equip_system(mut events: EventReader<ItemEquippedInvEvent>) {
    let _drained = events.read().count();
}
