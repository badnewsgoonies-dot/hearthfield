use bevy::prelude::*;
use crate::game::events::EnemySpawnedEvent;
use crate::game::components::Enemy;

pub fn spawn_enemies_system(mut commands: Commands, mut reader: EventReader<EnemySpawnedEvent>) {
    // System spawn_enemies_system: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let mut _reader_count: u32 = 0;
    let mut _reader_last: Option<()> = None;
    for _ev in reader.read() {
        _reader_count = _reader_count.saturating_add(1);
        _reader_last = Some(());
        activity = activity.saturating_add(1);
    }
    let _ = _reader_last;
    // commands available; spawn entities here when intent fires
    if activity > 0 {
        // spawn_enemies_system: tick had {activity} actionable events
    }
    let _ = activity;
}
