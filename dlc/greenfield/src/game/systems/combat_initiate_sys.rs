use bevy::prelude::*;
use crate::game::events::{CombatInitiatedEvent, AttackStartedEvent};

pub fn combat_initiate_system(mut reader: EventReader<CombatInitiatedEvent>, mut writer: EventWriter<AttackStartedEvent>) {
    let mut _reader_count: u32 = 0;
    for _ev in reader.read() {
        _reader_count = _reader_count.saturating_add(1);
    }
    if _reader_count > 0 {
        // observed combat_initiate_system activity this tick
    }
}
