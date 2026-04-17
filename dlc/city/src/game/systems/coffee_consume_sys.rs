use bevy::prelude::*;
use crate::game::events::{CoffeeDrunkEvent, EnergyBoostedEvent};

pub fn coffee_consume_system(mut reader: EventReader<CoffeeDrunkEvent>, mut writer: EventWriter<EnergyBoostedEvent>) {
    for _ev in reader.read() {
        writer.send(EnergyBoostedEvent);
    }

}
