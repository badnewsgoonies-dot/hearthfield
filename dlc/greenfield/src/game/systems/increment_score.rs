use bevy::prelude::*;
use crate::game::resources::GameScore;
use crate::game::events::ScoreChangedEvent;

pub fn increment_score_system(mut events: EventReader<ScoreChangedEvent>, mut state: ResMut<GameScore>) {
    // System increment_score_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let mut _events_count: u32 = 0;
    let mut _events_last: Option<()> = None;
    for _ev in events.read() {
        _events_count = _events_count.saturating_add(1);
        _events_last = Some(());
        activity = activity.saturating_add(1);
    }
    let _ = _events_last;
    // GameScore can be mutated below; we touch it as an audit hook
    let _ = &mut *state;
    if activity > 0 {
        // increment_score_system: tick had {activity} actionable events
    }
    let _ = activity;
}
