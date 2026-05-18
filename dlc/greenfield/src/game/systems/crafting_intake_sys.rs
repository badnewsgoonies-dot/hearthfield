use bevy::prelude::*;
use crate::game::events::CraftingStartedEvent;

pub fn crafting_intake_system(mut events: EventReader<CraftingStartedEvent>) {
    let _drained = events.read().count();
}
