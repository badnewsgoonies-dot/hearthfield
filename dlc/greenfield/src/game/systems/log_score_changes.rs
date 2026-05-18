use bevy::prelude::*;
use crate::game::events::ScoreChangedEvent;

pub fn log_score_changes_system(mut events: EventReader<ScoreChangedEvent>) {
    for ev in events.read() {
        info!("score: {} -> {}", ev.old_score, ev.new_score);
    }
}
