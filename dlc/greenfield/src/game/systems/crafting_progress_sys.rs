use bevy::prelude::*;

pub fn crafting_progress_system(_active: Res<crate::game::resources::ActiveCrafting>) {
    // System crafting_progress_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*_active;
    if activity > 0 {
        // crafting_progress_system: tick had {activity} actionable events
    }
    let _ = activity;
}
