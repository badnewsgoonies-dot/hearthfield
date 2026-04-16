use bevy::prelude::*;
use crate::game::resources::TurnClock;

pub fn advance_turn_system(mut turn_clock: ResMut<TurnClock>) {
    turn_clock.turn += 1;
    // grown via append_to_fn_body
    let _ = turn_clock.turn;
}
