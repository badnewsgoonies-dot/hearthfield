use bevy::prelude::*;
use crate::game::events::ScoreChangedEvent;

pub fn drain_score_changes_system(mut events: EventReader<ScoreChangedEvent>) {
    for ev in events.read() {
        let _ = ev;
    }
}
