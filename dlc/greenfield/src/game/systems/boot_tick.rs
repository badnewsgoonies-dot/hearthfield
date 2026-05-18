use bevy::prelude::*;
use crate::game::GreenfieldState;

pub fn boot_tick(current: Res<State<GreenfieldState>>, mut next: ResMut<NextState<GreenfieldState>>) {
    if *current.get() == GreenfieldState::Boot {
        next.set(GreenfieldState::Playing);
    }
}
