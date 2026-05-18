use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// System: set overexposed white colour during flash, reset to white when done.
pub fn damage_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut rocks: Query<(Entity, &mut Sprite, &mut DamageFlash)>,
) {
    for (entity, mut sprite, mut flash) in rocks.iter_mut() {
        flash.timer.tick(time.delta());

        if flash.timer.finished() {
            sprite.color = Color::WHITE;
            commands.entity(entity).remove::<DamageFlash>();
        } else {
            // Overexposed white (HDR-style: values > 1.0 bloom in bright scenes)
            sprite.color = Color::srgb(2.0, 2.0, 2.0);
        }
    }
}


