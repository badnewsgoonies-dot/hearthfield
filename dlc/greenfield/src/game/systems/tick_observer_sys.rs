use bevy::prelude::*;

pub fn tick_observer_system(counter: Res<crate::game::resources::TickCounter>) {
    // System tick_observer_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*counter;
    if activity > 0 {
        // tick_observer_system: tick had {activity} actionable events
    }
    let _ = activity;
}
