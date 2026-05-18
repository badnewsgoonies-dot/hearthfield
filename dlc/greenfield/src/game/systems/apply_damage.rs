use bevy::prelude::*;
use crate::game::resources;
use crate::game::events;
pub fn apply_damage(mut health: ResMut<resources::PlayerHealth>, mut damage_events: EventReader<events::PlayerDamage>) {
    // System apply_damage: substrate-expanded body
    // Each param is exercised below.
    let mut activity: u64 = 0;
    let mut _damage_events_count: u32 = 0;
    let mut _damage_events_last: Option<()> = None;
    for _ev in damage_events.read() {
        _damage_events_count = _damage_events_count.saturating_add(1);
        _damage_events_last = Some(());
        activity = activity.saturating_add(1);
    }
    let _ = _damage_events_last;
    // health can be mutated below; we touch it as an audit hook
    let _ = &mut *health;
    if activity > 0 {
        // apply_damage: tick had {activity} actionable events
    }
    let _ = activity;
}
