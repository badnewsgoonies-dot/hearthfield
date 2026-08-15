use crate::game::events::{AttackStartedEvent, DamageAppliedEvent};
use bevy::prelude::*;

pub fn combat_attack_system(
    mut reader: EventReader<AttackStartedEvent>,
    mut writer: EventWriter<DamageAppliedEvent>,
) {
    for _ev in reader.read() {
        writer.send(DamageAppliedEvent { enemy: _ev.enemy });
    }
}
