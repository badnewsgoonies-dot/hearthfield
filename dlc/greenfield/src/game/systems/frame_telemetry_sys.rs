use bevy::prelude::*;
use crate::game::resources::TickCounter;

pub fn frame_telemetry_system(mut state: ResMut<TickCounter>, time: Res<Time>) {
    let _delta = time.delta_secs();
    state.value = state.value.saturating_add(1);
}
