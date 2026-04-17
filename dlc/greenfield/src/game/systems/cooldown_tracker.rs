use bevy::prelude::*;
use crate::game::resources::CooldownClock;
pub fn cooldown_tick_system(time: Res<Time>, mut clock: ResMut<CooldownClock>) {
    clock.remaining -= time.delta_secs();

}
