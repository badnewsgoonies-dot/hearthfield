use bevy::prelude::*;
use crate::game::events::ScoreChangedEvent;

pub fn drain_score_changes_system(mut events: EventReader<ScoreChangedEvent>) {
    let _drained = events.read().count();
}
