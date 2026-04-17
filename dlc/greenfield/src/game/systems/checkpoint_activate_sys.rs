use bevy::prelude::*;
use crate::game::resources::CheckpointState;
use crate::game::events::CheckpointActivatedEvent;

pub fn checkpoint_activate_system(mut state: ResMut<CheckpointState>, mut reader: EventReader<CheckpointActivatedEvent>) {
    for _ev in reader.read() {
        state.last_id = state.last_id.saturating_add(1);
        state.visits = state.visits.saturating_add(1);
    }

}
