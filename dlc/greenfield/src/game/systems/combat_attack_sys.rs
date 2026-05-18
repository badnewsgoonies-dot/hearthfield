use bevy::prelude::*;
use crate::game::events::{AttackStartedEvent, DamageAppliedEvent};

pub fn combat_attack_system(mut reader: EventReader<AttackStartedEvent>, mut writer: EventWriter<DamageAppliedEvent>) {
    for _ev in reader.read() {
        writer.send(DamageAppliedEvent);
    }
}
