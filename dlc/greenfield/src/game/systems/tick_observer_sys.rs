use bevy::prelude::*;
use crate::game::events::TickAdvancedEvent;

pub fn tick_observer_system(mut reader: EventReader<TickAdvancedEvent>) {
    for _ev in reader.read() {
        info!("tick observed");
    }

}
