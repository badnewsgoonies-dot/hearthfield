use bevy::prelude::*;
use crate::game::events::PlayerMovedEvent;
use crate::game::components::PlayerMarker;

pub fn sim_tick(mut reader: EventReader<PlayerMovedEvent>, mut query: Query<&mut Transform, With<PlayerMarker>>) {
    for event in reader.read() {
        for mut transform in &mut query {
            transform.translation.x += event.x;
            transform.translation.y += event.y;
        }
    }
}
