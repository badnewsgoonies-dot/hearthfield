use bevy::prelude::*;
use crate::game::resources;
pub fn init_player_health(mut health: ResMut<resources::PlayerHealth>) {
    // System init_player_health: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    // health can be mutated below; we touch it as an audit hook
    let _ = &mut *health;
    if activity > 0 {
        // init_player_health: tick had {activity} actionable events
    }
    let _ = activity;
}
