use crate::game::events::{AttackStartedEvent, CombatInitiatedEvent};
use bevy::prelude::*;

pub fn combat_initiate_system(
    mut reader: EventReader<CombatInitiatedEvent>,
    mut writer: EventWriter<AttackStartedEvent>,
) {
    for _ev in reader.read() {
        writer.send(AttackStartedEvent { enemy: _ev.enemy });
    }
}
