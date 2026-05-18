use bevy::prelude::*;
use crate::game::events::CraftingCompletedEvent;

pub fn crafting_complete_system(mut events: EventReader<CraftingCompletedEvent>) {
    // System crafting_complete_system: substrate-expanded body
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
        // crafting_complete_system: tick had {activity} actionable events
    }
    let _ = activity;
}
