use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Remove all fireflies when dusk ends or the player enters an indoor map.
pub fn crafting_cleanup_system(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    mut swarm_state: ResMut<FireflySwarmState>,
    fireflies: Query<Entity, With<Firefly>>,
) {
    if fireflies_should_be_active(&calendar, player_state.current_map) {
        return;
    }

    for entity in &fireflies {
        commands.entity(entity).despawn();
    }
    swarm_state.target_count = None;
}


