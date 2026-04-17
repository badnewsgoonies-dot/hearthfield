use bevy::prelude::*;
use crate::game::resources::TickCounter;
use crate::game::events::TickAdvancedEvent;

pub fn tick_advance_system(mut counter: ResMut<TickCounter>, mut writer: EventWriter<TickAdvancedEvent>) {
    counter.value += 1;
    writer.send(TickAdvancedEvent);

}
