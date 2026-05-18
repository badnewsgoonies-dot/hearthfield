use bevy::prelude::*;
use crate::game::resources::GameScore;
use crate::game::events::ScoreChangedEvent;

pub fn increment_score_system(mut events: EventReader<ScoreChangedEvent>, mut state: ResMut<GameScore>) {
    for ev in events.read() {
        state.total = ev.new_score.max(0) as u32;
        if state.total > state.high {
            state.high = state.total;
        }
    }
}
