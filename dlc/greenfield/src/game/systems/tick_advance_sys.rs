use bevy::prelude::*;
use crate::game::resources::TickCounter;

pub fn tick_advance_system(mut counter: ResMut<TickCounter>) {
    counter.value = counter.value.saturating_add(1);
}
