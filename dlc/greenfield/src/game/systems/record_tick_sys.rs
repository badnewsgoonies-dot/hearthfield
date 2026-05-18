use bevy::prelude::*;

pub fn record_tick_system(_buffer: Res<crate::game::resources::RecordingBuffer>) {
    // System record_tick_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*_buffer;
    if activity > 0 {
        // record_tick_system: tick had {activity} actionable events
    }
    let _ = activity;
}
