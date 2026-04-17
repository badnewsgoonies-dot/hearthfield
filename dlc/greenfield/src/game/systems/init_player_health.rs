use bevy::prelude::*;
use crate::game::resources;
pub fn init_player_health(mut health: ResMut<resources::PlayerHealth>) {
    health.hp = 100.0;
    health.max_hp = 100.0;

}
