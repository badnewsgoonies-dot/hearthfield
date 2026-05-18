use bevy::prelude::*;
use crate::game::events::CraftingCompletedEvent;

pub fn crafting_complete_system(mut events: EventReader<CraftingCompletedEvent>) {
    let _drained = events.read().count();
}
