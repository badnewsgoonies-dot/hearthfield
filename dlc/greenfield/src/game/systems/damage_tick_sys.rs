use bevy::prelude::*;
use crate::game::events::{DamageDealtEvent, PlayerDamage};

pub fn damage_tick_system(mut reader: EventReader<DamageDealtEvent>, mut writer: EventWriter<PlayerDamage>) {
    let mut _reader_count: u32 = 0;
    for _ev in reader.read() {
        _reader_count = _reader_count.saturating_add(1);
    }
    if _reader_count > 0 {
        // observed damage_tick_system activity this tick
    }
}
