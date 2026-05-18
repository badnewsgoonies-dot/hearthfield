use bevy::prelude::*;
use crate::game::components::{PlayerMarker, HudRoot, HudTimer};
use crate::game::resources::{GameScore, LevelProgress, PlayerHealth};

pub fn update_hud_system(mut commands: Commands, mut spawned: Local<bool>, score: Res<GameScore>, level: Res<LevelProgress>, health: Res<PlayerHealth>, mut hud_query: Query<&mut Text, With<HudTimer>>) {
    // System update_hud_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let _ = &*score;
    let _ = &*level;
    let _ = &*health;
    let mut _hud_query_visited: u32 = 0;
    for _entity in hud_query.iter() {
        _hud_query_visited = _hud_query_visited.saturating_add(1);
        activity = activity.saturating_add(1);
    }
    // commands available; spawn entities here when intent fires
    // local state preserved across ticks; useful for accumulators
    if activity > 0 {
        // update_hud_system: tick had {activity} actionable events
    }
    let _ = activity;
}
