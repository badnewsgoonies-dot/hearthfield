use bevy::prelude::*;

pub fn handle_config_system(_config: Res<crate::game::resources::GameConfig>) {
    // System handle_config_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*_config;
    if activity > 0 {
        // handle_config_system: tick had {activity} actionable events
    }
    let _ = activity;
}
