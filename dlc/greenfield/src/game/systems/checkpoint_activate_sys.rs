use bevy::prelude::*;
use crate::game::events::CheckpointActivatedEvent;
use crate::game::resources::CheckpointState;

pub fn checkpoint_activate_system(mut events: EventReader<CheckpointActivatedEvent>, mut state: ResMut<CheckpointState>) {
    for _ev in events.read() {
        state.visits = state.visits.saturating_add(1);
        state.last_id = state.last_id.saturating_add(1);
    }
}
