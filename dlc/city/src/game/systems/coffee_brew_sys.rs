use bevy::prelude::*;
use crate::game::resources::CoffeeBrewsToday;
use crate::game::events::CoffeeBrewedEvent;

pub fn coffee_brew_system(mut brews: ResMut<CoffeeBrewsToday>, mut writer: EventWriter<CoffeeBrewedEvent>) {
    brews.count += 1;
    writer.send(CoffeeBrewedEvent);

}
