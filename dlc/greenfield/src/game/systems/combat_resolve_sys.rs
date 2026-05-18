use bevy::prelude::*;
use crate::game::events::{CombatResolvedEvent, EnemyDefeatedEvent};

pub fn combat_resolve_system(mut reader: EventReader<CombatResolvedEvent>, mut writer: EventWriter<EnemyDefeatedEvent>) {
    let mut _reader_count: u32 = 0;
    for _ev in reader.read() {
        _reader_count = _reader_count.saturating_add(1);
    }
    if _reader_count > 0 {
        // observed combat_resolve_system activity this tick
    }
}
