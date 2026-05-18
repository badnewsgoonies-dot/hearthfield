use bevy::prelude::*;

pub fn tick_observer_system(counter: Res<crate::game::resources::TickCounter>) {
    let _ = counter.value;
}
