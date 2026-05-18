use bevy::prelude::*;
use crate::game::resources::TurnClock;

pub fn tick_clock_system(mut clock: ResMut<TurnClock>, time: Res<Time>) {
    clock.elapsed_secs += time.delta_secs();
    if clock.elapsed_secs >= 1.0 {
        clock.elapsed_secs -= 1.0;
        clock.turn = clock.turn.saturating_add(1);
    }
}
