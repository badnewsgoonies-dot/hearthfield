use bevy::prelude::*;
use crate::game::resources::QuestLog;
use crate::game::events::QuestAcceptedEvent;

pub fn quest_accept_system(mut log: ResMut<QuestLog>, mut reader: EventReader<QuestAcceptedEvent>) {
    for _ev in reader.read() {
        log.active = log.active.saturating_add(1);
    }

}
