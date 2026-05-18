use bevy::prelude::*;
use crate::game::events::EnemyDefeatedEvent;

pub fn drop_loot_system(mut reader: EventReader<EnemyDefeatedEvent>) {
    // System drop_loot_system: substrate-expanded body
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
    if activity > 0 {
        // drop_loot_system: tick had {activity} actionable events
    }
    let _ = activity;
}
