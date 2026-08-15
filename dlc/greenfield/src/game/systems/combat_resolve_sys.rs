use crate::game::events::{CombatResolvedEvent, EnemyDefeatedEvent};
use bevy::prelude::*;

pub fn combat_resolve_system(
    mut reader: EventReader<CombatResolvedEvent>,
    mut writer: EventWriter<EnemyDefeatedEvent>,
) {
    for _ev in reader.read() {
        writer.send(EnemyDefeatedEvent { enemy: _ev.enemy });
    }
}
