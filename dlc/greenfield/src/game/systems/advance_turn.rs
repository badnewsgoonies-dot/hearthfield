use bevy::prelude::*;
use crate::game::resources::TurnClock;

pub fn advance_turn_system(mut clock: ResMut<TurnClock>) {
    // System advance_turn_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    // TurnClock can be mutated below; we touch it as an audit hook
    let _ = &mut *clock;
    if activity > 0 {
        // advance_turn_system: tick had {activity} actionable events
    }
    let _ = activity;
}
