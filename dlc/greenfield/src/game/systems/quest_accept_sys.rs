use bevy::prelude::*;
use crate::game::events::QuestAcceptedEvent;
use crate::game::resources::QuestLog;

pub fn quest_accept_system(mut events: EventReader<QuestAcceptedEvent>, mut state: ResMut<QuestLog>) {
    for _ev in events.read() {
        state.active = state.active.saturating_add(1);
    }
}
