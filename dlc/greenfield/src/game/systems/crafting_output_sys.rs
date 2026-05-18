use bevy::prelude::*;
use crate::game::events::OutputProducedEvent;

pub fn crafting_output_system(mut events: EventReader<OutputProducedEvent>) {
    let _drained = events.read().count();
}
