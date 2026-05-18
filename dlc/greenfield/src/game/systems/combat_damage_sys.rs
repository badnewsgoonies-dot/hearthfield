use bevy::prelude::*;
use crate::game::events::{DamageAppliedEvent, CombatResolvedEvent};

pub fn combat_damage_system(mut reader: EventReader<DamageAppliedEvent>, mut writer: EventWriter<CombatResolvedEvent>, mut counter: Local<u32>) {
    for _ev in reader.read() {
        *counter += 1;
        if *counter >= 3 {
            *counter = 0;
            writer.send(CombatResolvedEvent);
        }
    }
}
