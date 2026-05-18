use bevy::prelude::*;
use crate::game::events::{CombatInitiatedEvent, AttackStartedEvent};

pub fn combat_initiate_system(mut reader: EventReader<CombatInitiatedEvent>, mut writer: EventWriter<AttackStartedEvent>) {
    for _ev in reader.read() {
        writer.send(AttackStartedEvent);
    }
}
