use bevy::prelude::*;
use crate::game::events::PlayerMovedEvent;

pub fn sim_tick(mut reader: EventReader<PlayerMovedEvent>) {
    for event in reader.read() {
        info!("player moved: dx={} dy={}", event.x, event.y);
    }
}
