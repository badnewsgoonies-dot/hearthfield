use bevy::prelude::*;
use crate::game::resources::TurnClock;

pub fn advance_turn_system(mut clock: ResMut<TurnClock>) {
    clock.turn = clock.turn.saturating_add(1);
}
