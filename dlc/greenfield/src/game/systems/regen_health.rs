use bevy::prelude::*;
use crate::game::resources;
pub fn regen_health(mut health: ResMut<resources::PlayerHealth>, time: Res<Time>) {
    let regen_rate = 5.0_f32;
    health.hp = (health.hp + regen_rate * time.delta_secs()).min(health.max_hp);

}
