//! Map transition and day-end event handlers for the NPC domain.

use bevy::prelude::*;
use crate::shared::*;
use super::spawning::{SpawnedNpcs, NpcMapTag, spawn_npcs_for_map};

/// System: handle MapTransitionEvent — despawn old map NPCs, spawn new map NPCs.
pub fn handle_map_transition(
    mut commands: Commands,
    mut transition_reader: EventReader<MapTransitionEvent>,
    mut spawned: ResMut<SpawnedNpcs>,
    npc_map_tags: Query<(Entity, &NpcMapTag)>,
    calendar: Res<Calendar>,
    npc_registry: Res<NpcRegistry>,
) {
    for event in transition_reader.read() {
        // Despawn all currently loaded NPC entities
        let mut despawned_entities = Vec::new();
        for (entity, _tag) in npc_map_tags.iter() {
            commands.entity(entity).despawn_recursive();
            despawned_entities.push(entity);
        }
        // Clear the tracking map
        spawned.entities.retain(|_, e| !despawned_entities.contains(e));

        // Spawn NPCs that belong to the new map
        spawn_npcs_for_map(
            &mut commands,
            &calendar,
            event.to_map,
            &npc_registry,
            &mut spawned,
        );
    }
}

/// System: handle DayEndEvent — reset gifted_today for all NPCs.
pub fn handle_day_end(
    mut day_end_reader: EventReader<DayEndEvent>,
    mut relationships: ResMut<Relationships>,
) {
    for _event in day_end_reader.read() {
        relationships.gifted_today.clear();
    }
}
