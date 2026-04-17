use bevy::prelude::*;
use crate::game::resources::QuestLog;
use crate::game::events::QuestCompletedEvent;

pub fn quest_complete_system(mut log: ResMut<QuestLog>, mut reader: EventReader<QuestCompletedEvent>) {
    for _ev in reader.read() {
        log.active = log.active.saturating_sub(1);
        log.completed = log.completed.saturating_add(1);
    }

}
