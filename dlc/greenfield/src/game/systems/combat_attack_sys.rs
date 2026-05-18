use bevy::prelude::*;
use crate::game::events::{AttackStartedEvent, DamageAppliedEvent};

pub fn combat_attack_system(mut reader: EventReader<AttackStartedEvent>, mut writer: EventWriter<DamageAppliedEvent>) {
    let mut _reader_count: u32 = 0;
    for _ev in reader.read() {
        _reader_count = _reader_count.saturating_add(1);
    }
    if _reader_count > 0 {
        // observed combat_attack_system activity this tick
    }
}
