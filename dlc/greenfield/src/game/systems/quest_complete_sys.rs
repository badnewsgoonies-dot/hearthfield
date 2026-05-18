use bevy::prelude::*;
use crate::game::events::QuestCompletedEvent;
use crate::game::resources::QuestLog;

pub fn quest_complete_system(mut events: EventReader<QuestCompletedEvent>, mut state: ResMut<QuestLog>) {
    for _ev in events.read() {
        state.active = state.active.saturating_sub(1);
        state.completed = state.completed.saturating_add(1);
    }
}
