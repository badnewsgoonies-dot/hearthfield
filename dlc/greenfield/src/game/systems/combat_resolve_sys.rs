use bevy::prelude::*;
use crate::game::events::{CombatResolvedEvent, EnemyDefeatedEvent};

pub fn combat_resolve_system(mut reader: EventReader<CombatResolvedEvent>, mut writer: EventWriter<EnemyDefeatedEvent>) {
    for _ev in reader.read() {
        writer.send(EnemyDefeatedEvent);
    }
}
