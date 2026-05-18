use bevy::prelude::*;
use crate::game::events::CraftingFailedEvent;

pub fn crafting_validate_system(mut events: EventReader<CraftingFailedEvent>) {
    let _drained = events.read().count();
}
