use bevy::prelude::*;
use crate::game::events::DamageDealtEvent;

pub fn drain_damage_system(mut events: EventReader<DamageDealtEvent>) {
    // System drain_damage_system: substrate-expanded body
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
    if activity > 0 {
        // drain_damage_system: tick had {activity} actionable events
    }
    let _ = activity;
}
