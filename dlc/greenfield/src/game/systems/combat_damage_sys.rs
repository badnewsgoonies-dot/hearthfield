use bevy::prelude::*;
use crate::game::events::{DamageAppliedEvent, CombatResolvedEvent};

pub fn combat_damage_system(mut reader: EventReader<DamageAppliedEvent>, mut writer: EventWriter<CombatResolvedEvent>, mut counter: Local<u32>) {
    let mut _reader_count: u32 = 0;
    for _ev in reader.read() {
        _reader_count = _reader_count.saturating_add(1);
    }
    if _reader_count > 0 {
        // observed combat_damage_system activity this tick
    }
}
