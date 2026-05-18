use bevy::prelude::*;
use crate::game::events::ScoreChangedEvent;

pub fn log_score_changes_system(mut reader: EventReader<ScoreChangedEvent>) {
    for ev in reader.read() {
        info!("score: {} -> {}", ev.old_score, ev.new_score);
    }
}
